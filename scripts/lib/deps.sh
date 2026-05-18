#!/usr/bin/env bash
# Dependency probing and (optional) installation.
#
# Each `check_*` function prints a one-line status and sets a global
# DEPS_MISSING array so the caller can decide whether to bail, prompt, or
# auto-install.

[[ -n "${__ANALOG_CLOUD_DEPS_SH:-}" ]] && return 0
__ANALOG_CLOUD_DEPS_SH=1

# shellcheck source=common.sh
. "$(dirname "${BASH_SOURCE[0]}")/common.sh"

DEPS_MISSING=()
DEPS_OPTIONAL_MISSING=()

_record_missing() { DEPS_MISSING+=("$1"); }
_record_optional() { DEPS_OPTIONAL_MISSING+=("$1"); }

check_rust() {
  if has_cmd cargo && has_cmd rustc; then
    log_ok "rust:    $(rustc --version 2>/dev/null)"
    return 0
  fi
  log_warn "rust:    not found (cargo + rustc required)"
  _record_missing "rust"
  return 1
}

check_node() {
  if has_cmd node; then
    local v
    v=$(node --version 2>/dev/null)
    local major="${v#v}"; major="${major%%.*}"
    if [[ -n "$major" ]] && (( major >= 18 )); then
      log_ok "node:    $v"
      return 0
    fi
    log_warn "node:    $v (need >= 18)"
  else
    log_warn "node:    not found"
  fi
  _record_missing "node"
  return 1
}

check_pnpm() {
  if has_cmd pnpm; then
    log_ok "pnpm:    $(pnpm --version 2>/dev/null)"
    return 0
  fi
  log_warn "pnpm:    not found"
  _record_missing "pnpm"
  return 1
}

# Audio/media system libraries. These are real prerequisites at *runtime*
# on the target host. They are listed as optional during install because
# the workspace builds in mock mode without them — we only nag if the
# user is targeting a real deploy.
check_system_audio() {
  local os="$1"
  local missing=()
  local probe=(pipewire wireplumber gst-launch-1.0 ffmpeg pkg-config)
  for tool in "${probe[@]}"; do
    if ! has_cmd "$tool"; then missing+=("$tool"); fi
  done
  if [[ ${#missing[@]} -eq 0 ]]; then
    log_ok "audio:   pipewire/wireplumber/gst/ffmpeg/pkg-config present"
    return 0
  fi
  log_warn "audio:   missing ${missing[*]} (required for real-hardware builds)"
  _record_optional "system-audio:${missing[*]}"
  return 1
}

# ---- installers ---------------------------------------------------------
#
# Each installer is a *best-effort* convenience for the common case. If
# the host is exotic (NixOS, immutable distros, etc.) we never push past
# our knowledge — we explain what we'd run and bail out, so the user can
# do it their own way.

install_rust() {
  log_step "Installing Rust via rustup"
  if ! has_cmd curl; then
    die "curl is required to install rustup. Install curl and re-run, or install Rust manually."
  fi
  if confirm "Run the official rustup installer (https://sh.rustup.rs)?" "Y"; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    # shellcheck disable=SC1091
    [[ -f "$HOME/.cargo/env" ]] && . "$HOME/.cargo/env"
    has_cmd cargo || die "rustup completed but cargo still not on PATH. Open a new shell and re-run."
    log_ok "rust installed: $(rustc --version)"
  else
    die "Rust is required. Install it from https://rustup.rs and re-run."
  fi
}

install_node_corepack() {
  local os="$1"
  log_step "Installing Node.js"
  case "$os" in
    linux-ubuntu|linux-debian)
      if confirm "Install Node.js 20.x via NodeSource APT repository (needs sudo)?" "Y"; then
        run curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
        run sudo apt-get install -y nodejs
      else
        die "Node.js is required. Install >= 18 manually and re-run."
      fi
      ;;
    linux-fedora)
      if confirm "Install Node.js 20.x via dnf (needs sudo)?" "Y"; then
        run sudo dnf install -y nodejs npm
      else
        die "Node.js is required. Install >= 18 manually and re-run."
      fi
      ;;
    linux-arch)
      if confirm "Install Node.js + pnpm via pacman (needs sudo)?" "Y"; then
        run sudo pacman -S --noconfirm nodejs npm
      else
        die "Node.js is required. Install >= 18 manually and re-run."
      fi
      ;;
    macos)
      if has_cmd brew; then
        if confirm "Install Node.js via Homebrew?" "Y"; then
          run brew install node
        else
          die "Node.js is required. Install >= 18 manually and re-run."
        fi
      else
        die "Homebrew not found. Install Node.js >= 18 manually (https://nodejs.org) and re-run."
      fi
      ;;
    *)
      die "Don't know how to install Node.js on '$os'. Install >= 18 manually and re-run."
      ;;
  esac
  has_cmd node || die "Node still not on PATH after install."
  log_ok "node installed: $(node --version)"
}

install_pnpm() {
  log_step "Activating pnpm"
  if has_cmd corepack; then
    run corepack enable
    run corepack prepare pnpm@latest --activate
  elif has_cmd npm; then
    run npm install -g pnpm
  else
    die "Neither corepack nor npm available. Install pnpm manually and re-run."
  fi
  has_cmd pnpm || die "pnpm still not on PATH after install."
  log_ok "pnpm installed: $(pnpm --version)"
}

install_system_audio() {
  local os="$1"
  log_step "Installing system audio stack"
  case "$os" in
    linux-ubuntu|linux-debian)
      if confirm "Install PipeWire + GStreamer + FFmpeg + dev headers via apt (needs sudo)?" "Y"; then
        run sudo apt-get update
        run sudo apt-get install -y \
          pipewire pipewire-pulse pipewire-audio-client-libraries wireplumber \
          libpipewire-0.3-dev libspa-0.2-dev \
          gstreamer1.0-tools \
          gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
          gstreamer1.0-plugins-bad  gstreamer1.0-plugins-ugly \
          gstreamer1.0-libav libgstreamer1.0-dev \
          ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswresample-dev \
          pkg-config libssl-dev build-essential
      else
        log_warn "Skipping system audio install — the backend will fall back to mock mode."
      fi
      ;;
    linux-fedora)
      if confirm "Install audio stack via dnf (needs sudo)?" "Y"; then
        run sudo dnf install -y \
          pipewire pipewire-pulseaudio wireplumber \
          pipewire-devel \
          gstreamer1 gstreamer1-plugins-base gstreamer1-plugins-good \
          gstreamer1-plugins-bad-free gstreamer1-plugins-ugly-free \
          gstreamer1-devel ffmpeg ffmpeg-devel \
          openssl-devel pkgconf-pkg-config gcc
      else
        log_warn "Skipping system audio install."
      fi
      ;;
    linux-arch)
      if confirm "Install audio stack via pacman (needs sudo)?" "Y"; then
        run sudo pacman -S --noconfirm \
          pipewire pipewire-pulse wireplumber \
          gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly \
          gst-libav ffmpeg pkgconf base-devel openssl
      else
        log_warn "Skipping system audio install."
      fi
      ;;
    macos)
      log_warn "macOS doesn't ship PipeWire. The backend will run in mock mode."
      log_warn "Real-hardware audio paths target Linux/PipeWire only."
      ;;
    *)
      log_warn "Don't know how to install audio deps on '$os' — install manually if you need real audio."
      ;;
  esac
}
