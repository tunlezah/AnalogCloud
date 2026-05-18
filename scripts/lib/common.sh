#!/usr/bin/env bash
# Common helpers: logging, prompts, fs, OS detection. Sourced by other
# scripts. Safe to source multiple times.

[[ -n "${__ANALOG_CLOUD_COMMON_SH:-}" ]] && return 0
__ANALOG_CLOUD_COMMON_SH=1

# ---- terminal / colors --------------------------------------------------

if [[ -t 1 ]] && [[ "${NO_COLOR:-}" = "" ]]; then
  C_RESET=$'\033[0m'
  C_DIM=$'\033[2m'
  C_BOLD=$'\033[1m'
  C_RED=$'\033[31m'
  C_GREEN=$'\033[32m'
  C_YELLOW=$'\033[33m'
  C_BLUE=$'\033[34m'
  C_CYAN=$'\033[36m'
else
  C_RESET="" C_DIM="" C_BOLD="" C_RED="" C_GREEN="" C_YELLOW="" C_BLUE="" C_CYAN=""
fi

log_info()  { printf '%s[i]%s %s\n' "$C_BLUE"   "$C_RESET" "$*"; }
log_ok()    { printf '%s[+]%s %s\n' "$C_GREEN"  "$C_RESET" "$*"; }
log_warn()  { printf '%s[!]%s %s\n' "$C_YELLOW" "$C_RESET" "$*" >&2; }
log_err()   { printf '%s[x]%s %s\n' "$C_RED"    "$C_RESET" "$*" >&2; }
log_step()  { printf '\n%s==>%s %s%s%s\n' "$C_CYAN" "$C_RESET" "$C_BOLD" "$*" "$C_RESET"; }
log_dim()   { printf '%s    %s%s\n' "$C_DIM" "$*" "$C_RESET"; }

die() { log_err "$*"; exit 1; }

# ---- prompts ------------------------------------------------------------
#
# All prompts honor $ANALOG_CLOUD_NONINTERACTIVE=1 by returning the
# supplied default, so the installer is usable from CI / cloud setup
# hooks without a TTY.

prompt() {
  # prompt "Question" "default"
  local question="$1" default="${2-}"
  if [[ "${ANALOG_CLOUD_NONINTERACTIVE:-0}" = "1" || ! -t 0 ]]; then
    printf '%s\n' "$default"
    return 0
  fi
  local hint=""
  [[ -n "$default" ]] && hint=" [$default]"
  local reply
  read -r -p "${C_BOLD}?${C_RESET} ${question}${hint}: " reply </dev/tty
  [[ -z "$reply" ]] && reply="$default"
  printf '%s\n' "$reply"
}

confirm() {
  # confirm "Question" "Y" -> 0 on yes, 1 on no
  local question="$1" default="${2:-Y}"
  local hint="[Y/n]"; [[ "$default" = "N" || "$default" = "n" ]] && hint="[y/N]"
  if [[ "${ANALOG_CLOUD_NONINTERACTIVE:-0}" = "1" || ! -t 0 ]]; then
    [[ "$default" = "Y" || "$default" = "y" ]] && return 0 || return 1
  fi
  local reply
  read -r -p "${C_BOLD}?${C_RESET} ${question} ${hint} " reply </dev/tty
  [[ -z "$reply" ]] && reply="$default"
  case "$reply" in
    [Yy]|[Yy][Ee][Ss]) return 0 ;;
    *) return 1 ;;
  esac
}

# ---- OS detection -------------------------------------------------------

detect_os() {
  # Echoes one of: linux-ubuntu, linux-debian, linux-fedora, linux-arch,
  # linux-unknown, macos, unknown.
  case "$(uname -s)" in
    Darwin) printf 'macos\n'; return 0 ;;
    Linux)  ;;
    *)      printf 'unknown\n'; return 0 ;;
  esac
  if [[ -r /etc/os-release ]]; then
    # shellcheck disable=SC1091
    . /etc/os-release
    case "${ID:-}" in
      ubuntu) printf 'linux-ubuntu\n' ;;
      debian) printf 'linux-debian\n' ;;
      fedora) printf 'linux-fedora\n' ;;
      arch|manjaro) printf 'linux-arch\n' ;;
      *)
        case " ${ID_LIKE:-} " in
          *debian*) printf 'linux-debian\n' ;;
          *fedora*) printf 'linux-fedora\n' ;;
          *arch*)   printf 'linux-arch\n'   ;;
          *)        printf 'linux-unknown\n' ;;
        esac
        ;;
    esac
  else
    printf 'linux-unknown\n'
  fi
}

# ---- paths --------------------------------------------------------------

xdg_config_home() {
  printf '%s\n' "${XDG_CONFIG_HOME:-$HOME/.config}"
}

xdg_data_home() {
  printf '%s\n' "${XDG_DATA_HOME:-$HOME/.local/share}"
}

# Repo root resolves relative to this file's location, so the installer
# works regardless of the user's CWD.
repo_root() {
  local here
  here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  printf '%s\n' "$here"
}

install_state_file() {
  printf '%s/analog-cloud/installer.env\n' "$(xdg_config_home)"
}

env_file() {
  printf '%s/.env\n' "$(repo_root)"
}

# ---- misc ---------------------------------------------------------------

has_cmd() { command -v "$1" >/dev/null 2>&1; }

# Run a command and stream its output prefixed for clarity. Returns the
# command's exit code.
run() {
  log_dim "\$ $*"
  "$@"
}
