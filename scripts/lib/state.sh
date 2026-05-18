#!/usr/bin/env bash
# Detect and persist installation state.
#
# We treat *every* run as "unknown environment" by default — we never
# assume something is installed just because we ran the installer once
# before. Instead, we read the marker file (if it exists) as a *hint*,
# then re-verify everything against the live filesystem.

[[ -n "${__ANALOG_CLOUD_STATE_SH:-}" ]] && return 0
__ANALOG_CLOUD_STATE_SH=1

# shellcheck source=common.sh
. "$(dirname "${BASH_SOURCE[0]}")/common.sh"

INSTALL_VERSION=1

# Marker fields (set by load_install_state, written by save_install_state)
STATE_VERSION=""
STATE_REPO_ROOT=""
STATE_BACKEND_PORT=""
STATE_FRONTEND_PORT=""
STATE_INSTALLED_AT=""
STATE_BACKEND_BUILT=""
STATE_FRONTEND_BUILT=""
STATE_AUDIO_INSTALLED=""
STATE_RUST_MANAGED=""
STATE_NODE_MANAGED=""

# load_install_state
#
# Reads $(install_state_file) into the STATE_* vars. Missing file is
# *not* an error; it just means "first run". The marker is the only
# source of truth for "have we run before" — we never infer it from the
# presence of build artifacts, because target/ and node_modules/ can be
# in any state for any number of reasons.
load_install_state() {
  local file
  file="$(install_state_file)"
  [[ -r "$file" ]] || return 1
  # shellcheck disable=SC1090
  . "$file"
  STATE_VERSION="${INSTALL_VERSION_MARKER:-}"
  STATE_REPO_ROOT="${INSTALL_REPO_ROOT:-}"
  STATE_BACKEND_PORT="${INSTALL_BACKEND_PORT:-}"
  STATE_FRONTEND_PORT="${INSTALL_FRONTEND_PORT:-}"
  STATE_INSTALLED_AT="${INSTALL_INSTALLED_AT:-}"
  STATE_BACKEND_BUILT="${INSTALL_BACKEND_BUILT:-}"
  STATE_FRONTEND_BUILT="${INSTALL_FRONTEND_BUILT:-}"
  STATE_AUDIO_INSTALLED="${INSTALL_AUDIO_INSTALLED:-}"
  STATE_RUST_MANAGED="${INSTALL_RUST_MANAGED:-}"
  STATE_NODE_MANAGED="${INSTALL_NODE_MANAGED:-}"
  return 0
}

save_install_state() {
  local file
  file="$(install_state_file)"
  mkdir -p "$(dirname "$file")"
  umask 077
  cat >"$file" <<EOF
# Analog Cloud installer state. Managed by scripts/install.sh.
# Safe to delete — re-running the installer will rebuild it.
INSTALL_VERSION_MARKER=$INSTALL_VERSION
INSTALL_REPO_ROOT='${STATE_REPO_ROOT}'
INSTALL_BACKEND_PORT='${STATE_BACKEND_PORT}'
INSTALL_FRONTEND_PORT='${STATE_FRONTEND_PORT}'
INSTALL_INSTALLED_AT='${STATE_INSTALLED_AT:-$(date -u +%Y-%m-%dT%H:%M:%SZ)}'
INSTALL_BACKEND_BUILT='${STATE_BACKEND_BUILT}'
INSTALL_FRONTEND_BUILT='${STATE_FRONTEND_BUILT}'
INSTALL_AUDIO_INSTALLED='${STATE_AUDIO_INSTALLED}'
INSTALL_RUST_MANAGED='${STATE_RUST_MANAGED}'
INSTALL_NODE_MANAGED='${STATE_NODE_MANAGED}'
EOF
  log_dim "state -> $file"
}

# detect_existing_install
#
# Returns:
#   0 if a previous install is detected (marker present)
#   1 if no marker but artifacts exist (partial / manual)
#   2 if nothing detected (fresh)
#
# Sets DETECT_REASONS array describing what was found.
DETECT_REASONS=()
detect_existing_install() {
  DETECT_REASONS=()
  local root marker config
  root=$(repo_root)
  marker="$(install_state_file)"
  config="$(xdg_config_home)/analog-cloud/config.toml"

  local found_marker=0
  if [[ -r "$marker" ]]; then
    DETECT_REASONS+=("installer marker: $marker")
    found_marker=1
  fi
  [[ -d "$root/target" ]]               && DETECT_REASONS+=("backend build cache: $root/target")
  [[ -f "$root/target/debug/analog-cloud-server" ]] && DETECT_REASONS+=("backend binary: target/debug/analog-cloud-server")
  [[ -f "$root/target/release/analog-cloud-server" ]] && DETECT_REASONS+=("backend binary: target/release/analog-cloud-server")
  [[ -d "$root/frontend/node_modules" ]] && DETECT_REASONS+=("frontend deps: frontend/node_modules")
  [[ -d "$root/frontend/build" ]]        && DETECT_REASONS+=("frontend build: frontend/build")
  [[ -f "$root/.env" ]]                  && DETECT_REASONS+=("env file: .env")
  [[ -f "$config" ]]                     && DETECT_REASONS+=("runtime config: $config")

  if (( found_marker == 1 )); then
    return 0
  fi
  if [[ ${#DETECT_REASONS[@]} -gt 0 ]]; then
    return 1
  fi
  return 2
}
