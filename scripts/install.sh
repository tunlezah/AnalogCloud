#!/usr/bin/env bash
#
# Analog Cloud installer.
#
# Builds and configures Analog Cloud on the current host. Designed to:
#
#   1. Handle port conflicts gracefully. The backend defaults to 7777
#      and the frontend dev server to 5173; if either is in use the user
#      is shown the owning process and offered a free port.
#   2. Detect and respect existing installs. A marker file at
#      $XDG_CONFIG_HOME/analog-cloud/installer.env records prior runs.
#      Build artifacts and a stale runtime config also count as "this
#      machine has been touched before" — we surface what we found and
#      let the user choose update / reinstall / abort.
#   3. Assume nothing. Every prerequisite is probed against the live
#      system; nothing is inferred from "we installed it last time".
#
# Flags:
#   --reinstall     wipe build artifacts before rebuilding
#   --force         skip confirmations (implies non-interactive)
#   --skip-system   don't try to install OS-level audio deps
#   --skip-build    just configure ports + write .env, don't build
#   --backend-port  set backend port non-interactively (default: 7777)
#   --frontend-port set frontend dev port (default: 5173)
#   --release       build the backend with cargo --release
#   --help          show usage

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
. "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=lib/ports.sh
. "$SCRIPT_DIR/lib/ports.sh"
# shellcheck source=lib/deps.sh
. "$SCRIPT_DIR/lib/deps.sh"
# shellcheck source=lib/state.sh
. "$SCRIPT_DIR/lib/state.sh"

DEFAULT_BACKEND_PORT=7777
DEFAULT_FRONTEND_PORT=5173

OPT_REINSTALL=0
OPT_FORCE=0
OPT_SKIP_SYSTEM=0
OPT_SKIP_BUILD=0
OPT_RELEASE=0
OPT_BACKEND_PORT=""
OPT_FRONTEND_PORT=""

usage() {
  awk '
    NR==1 { next }            # skip shebang
    /^[^#]/ { exit }          # stop at first non-comment line
    { sub(/^# ?/, ""); print }
  ' "$0"
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --reinstall)      OPT_REINSTALL=1 ;;
      --force)          OPT_FORCE=1; ANALOG_CLOUD_NONINTERACTIVE=1 ;;
      --skip-system)    OPT_SKIP_SYSTEM=1 ;;
      --skip-build)     OPT_SKIP_BUILD=1 ;;
      --release)        OPT_RELEASE=1 ;;
      --backend-port)   shift; OPT_BACKEND_PORT="${1:-}" ;;
      --backend-port=*) OPT_BACKEND_PORT="${1#*=}" ;;
      --frontend-port)  shift; OPT_FRONTEND_PORT="${1:-}" ;;
      --frontend-port=*) OPT_FRONTEND_PORT="${1#*=}" ;;
      -h|--help)        usage; exit 0 ;;
      *) die "Unknown argument: $1 (try --help)" ;;
    esac
    shift
  done
  export ANALOG_CLOUD_NONINTERACTIVE
}

# ------------------------------------------------------------------------

print_banner() {
  printf '\n%s%s═══════════════════════════════════════%s\n' "$C_BOLD" "$C_CYAN" "$C_RESET"
  printf '%s     Analog Cloud — installer        %s\n' "$C_BOLD" "$C_RESET"
  printf '%s%s═══════════════════════════════════════%s\n\n' "$C_BOLD" "$C_CYAN" "$C_RESET"
}

# ------------------------------------------------------------------------

handle_existing_install() {
  log_step "Checking for an existing installation"

  load_install_state || true
  local rc
  set +e
  detect_existing_install
  rc=$?
  set -e

  case "$rc" in
    0)
      log_info "Previous installer marker found."
      for r in "${DETECT_REASONS[@]}"; do log_dim "- $r"; done
      log_info "Last build:  backend=${STATE_BACKEND_BUILT:-?}  frontend=${STATE_FRONTEND_BUILT:-?}"
      log_info "Last ports:  backend=${STATE_BACKEND_PORT:-?}  frontend=${STATE_FRONTEND_PORT:-?}"

      if (( OPT_REINSTALL == 1 )); then
        log_warn "--reinstall: wiping build artifacts."
        wipe_build_artifacts
        return 0
      fi
      if confirm "Wipe build artifacts and reinstall from scratch?" "N"; then
        wipe_build_artifacts
      else
        log_info "Keeping existing build cache — will incrementally rebuild."
      fi
      ;;
    1)
      log_warn "No installer marker, but build artifacts exist on disk:"
      for r in "${DETECT_REASONS[@]}"; do log_dim "- $r"; done
      log_warn "This usually means a prior manual build or a partial install."
      if (( OPT_REINSTALL == 1 )) || confirm "Wipe them and start clean?" "N"; then
        wipe_build_artifacts
      else
        log_info "Keeping existing artifacts — will incrementally rebuild."
      fi
      ;;
    2)
      log_ok "Clean machine. Performing a fresh install."
      ;;
  esac
}

wipe_build_artifacts() {
  local root; root=$(repo_root)
  [[ -d "$root/target" ]]               && run rm -rf "$root/target"
  [[ -d "$root/frontend/node_modules" ]] && run rm -rf "$root/frontend/node_modules"
  [[ -d "$root/frontend/build" ]]        && run rm -rf "$root/frontend/build"
  [[ -d "$root/frontend/.svelte-kit" ]]  && run rm -rf "$root/frontend/.svelte-kit"
  log_ok "Build artifacts removed."
}

# ------------------------------------------------------------------------

check_dependencies() {
  log_step "Probing toolchain"

  local os
  os=$(detect_os)
  log_info "Detected OS: $os"

  check_rust  || true
  check_node  || true
  check_pnpm  || true
  if (( OPT_SKIP_SYSTEM == 0 )); then
    check_system_audio "$os" || true
  fi

  if [[ ${#DEPS_MISSING[@]} -eq 0 ]]; then
    log_ok "All required dependencies present."
  else
    log_warn "Missing required tools: ${DEPS_MISSING[*]}"
    if confirm "Try to install them now?" "Y"; then
      for dep in "${DEPS_MISSING[@]}"; do
        case "$dep" in
          rust) install_rust ;;
          node) install_node_corepack "$os" ;;
          pnpm) install_pnpm ;;
        esac
      done
    else
      die "Required dependencies missing. Install them and re-run."
    fi
  fi

  if [[ ${#DEPS_OPTIONAL_MISSING[@]} -gt 0 && $OPT_SKIP_SYSTEM -eq 0 ]]; then
    log_warn "Optional system audio stack is incomplete: ${DEPS_OPTIONAL_MISSING[*]}"
    if confirm "Install the audio stack now? (Required for real-hardware playback.)" "Y"; then
      install_system_audio "$os"
      STATE_AUDIO_INSTALLED=1
    else
      log_info "Skipping. The backend will run in mock mode."
      STATE_AUDIO_INSTALLED=0
    fi
  else
    STATE_AUDIO_INSTALLED=1
  fi
}

# ------------------------------------------------------------------------

choose_ports() {
  log_step "Choosing service ports"

  local backend_default="${STATE_BACKEND_PORT:-$DEFAULT_BACKEND_PORT}"
  local frontend_default="${STATE_FRONTEND_PORT:-$DEFAULT_FRONTEND_PORT}"

  local backend_port frontend_port

  if [[ -n "$OPT_BACKEND_PORT" ]]; then
    valid_port "$OPT_BACKEND_PORT" || die "--backend-port: not a valid port: $OPT_BACKEND_PORT"
    if port_in_use "$OPT_BACKEND_PORT"; then
      die "--backend-port $OPT_BACKEND_PORT is in use ($(port_owner "$OPT_BACKEND_PORT"))."
    fi
    backend_port="$OPT_BACKEND_PORT"
  else
    backend_port=$(resolve_port "backend (analog-cloud-server)" "$backend_default")
  fi

  if [[ -n "$OPT_FRONTEND_PORT" ]]; then
    valid_port "$OPT_FRONTEND_PORT" || die "--frontend-port: not a valid port: $OPT_FRONTEND_PORT"
    if port_in_use "$OPT_FRONTEND_PORT"; then
      die "--frontend-port $OPT_FRONTEND_PORT is in use ($(port_owner "$OPT_FRONTEND_PORT"))."
    fi
    frontend_port="$OPT_FRONTEND_PORT"
  else
    frontend_port=$(resolve_port "frontend (Vite dev server)" "$frontend_default")
  fi

  if [[ "$backend_port" = "$frontend_port" ]]; then
    die "Backend and frontend can't share the same port ($backend_port). Re-run and pick distinct ones."
  fi

  STATE_BACKEND_PORT="$backend_port"
  STATE_FRONTEND_PORT="$frontend_port"

  log_ok "Backend port:  $backend_port"
  log_ok "Frontend port: $frontend_port"
}

# ------------------------------------------------------------------------

write_env_file() {
  log_step "Writing repo-local .env"
  local root; root=$(repo_root)
  local file="$root/.env"
  local bind="127.0.0.1:${STATE_BACKEND_PORT}"

  # Preserve unrelated env vars if the user has added any.
  if [[ -f "$file" ]]; then
    log_dim "Updating existing .env at $file"
    # strip any keys we own; keep everything else.
    grep -Ev '^(ANALOG_CLOUD_BIND|ANALOG_CLOUD_BACKEND_PORT|ANALOG_CLOUD_FRONTEND_PORT|VITE_BACKEND_URL|VITE_FRONTEND_PORT)=' "$file" \
      > "$file.tmp" || true
    mv "$file.tmp" "$file"
  fi

  {
    echo "# Analog Cloud — generated by scripts/install.sh."
    echo "# Hand-edits to other keys are preserved on re-run."
    echo "ANALOG_CLOUD_BIND=${bind}"
    echo "ANALOG_CLOUD_BACKEND_PORT=${STATE_BACKEND_PORT}"
    echo "ANALOG_CLOUD_FRONTEND_PORT=${STATE_FRONTEND_PORT}"
    echo "VITE_BACKEND_URL=http://${bind}"
    echo "VITE_FRONTEND_PORT=${STATE_FRONTEND_PORT}"
  } >> "$file"

  log_ok "Wrote $file"
}

# ------------------------------------------------------------------------

build_backend() {
  if (( OPT_SKIP_BUILD == 1 )); then
    log_info "--skip-build: not building backend."
    return 0
  fi
  log_step "Building backend (cargo)"
  local root; root=$(repo_root)
  ( cd "$root" && \
    if (( OPT_RELEASE == 1 )); then
      run cargo build --release -p analog-cloud-server
    else
      run cargo build -p analog-cloud-server
    fi
  )
  STATE_BACKEND_BUILT=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  log_ok "Backend built."
}

build_frontend() {
  if (( OPT_SKIP_BUILD == 1 )); then
    log_info "--skip-build: not building frontend."
    return 0
  fi
  log_step "Installing + building frontend (pnpm)"
  local root; root=$(repo_root)
  ( cd "$root/frontend" && \
    run pnpm install --frozen-lockfile && \
    run pnpm build
  )
  STATE_FRONTEND_BUILT=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  log_ok "Frontend built."
}

# ------------------------------------------------------------------------

print_summary() {
  local root; root=$(repo_root)
  printf '\n%s%s──────── Install complete ────────%s\n' "$C_BOLD" "$C_GREEN" "$C_RESET"
  printf '  backend port    : %s\n' "$STATE_BACKEND_PORT"
  printf '  frontend port   : %s\n' "$STATE_FRONTEND_PORT"
  printf '  repo root       : %s\n' "$root"
  printf '  env file        : %s\n' "$(env_file)"
  printf '  install marker  : %s\n' "$(install_state_file)"
  printf '  audio installed : %s\n' "${STATE_AUDIO_INSTALLED:-0}"
  printf '\n'
  printf '  %sStart the backend:%s\n' "$C_BOLD" "$C_RESET"
  if (( OPT_RELEASE == 1 )); then
    printf '    set -a; . ./.env; set +a\n'
    printf '    ./target/release/analog-cloud-server\n'
  else
    printf '    set -a; . ./.env; set +a\n'
    printf '    cargo run -p analog-cloud-server\n'
  fi
  printf '\n'
  printf '  %sStart the frontend (dev):%s\n' "$C_BOLD" "$C_RESET"
  printf '    cd frontend && pnpm dev\n'
  printf '\n'
  printf '  %sOr both, in one shot:%s\n' "$C_BOLD" "$C_RESET"
  printf '    ./scripts/run.sh\n'
  printf '\n'
  printf '  Re-run %sscripts/install.sh --reinstall%s to wipe + rebuild.\n' "$C_BOLD" "$C_RESET"
  printf '  Run    %sscripts/uninstall.sh%s            to undo this install.\n' "$C_BOLD" "$C_RESET"
}

# ------------------------------------------------------------------------

main() {
  parse_args "$@"
  print_banner

  STATE_REPO_ROOT=$(repo_root)
  cd "$STATE_REPO_ROOT"

  handle_existing_install
  check_dependencies
  choose_ports
  write_env_file
  build_backend
  build_frontend

  save_install_state
  print_summary
}

main "$@"
