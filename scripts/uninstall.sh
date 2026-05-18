#!/usr/bin/env bash
#
# Reverse what `install.sh` did. Conservative by default: removes build
# artifacts and the installer's state file, but leaves user config
# ($XDG_CONFIG_HOME/analog-cloud/config.toml) and user data
# ($XDG_DATA_HOME/analog-cloud/) alone unless --purge is passed.
#
# Flags:
#   --purge   also remove user config + data (destructive — recordings go)
#   --force   skip confirmation prompts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
. "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=lib/state.sh
. "$SCRIPT_DIR/lib/state.sh"

OPT_PURGE=0
OPT_FORCE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --purge)  OPT_PURGE=1 ;;
    --force)  OPT_FORCE=1; ANALOG_CLOUD_NONINTERACTIVE=1 ;;
    -h|--help)
      awk '
        NR==1 { next }
        /^[^#]/ { exit }
        { sub(/^# ?/, ""); print }
      ' "$0"
      exit 0
      ;;
    *) die "Unknown argument: $1 (try --help)" ;;
  esac
  shift
done
export ANALOG_CLOUD_NONINTERACTIVE

log_step "Uninstalling Analog Cloud"

root=$(repo_root)
marker=$(install_state_file)
config_dir="$(xdg_config_home)/analog-cloud"
data_dir="$(xdg_data_home)/analog-cloud"

echo "  repo root      : $root"
echo "  install marker : $marker"
echo "  config dir     : $config_dir"
echo "  data dir       : $data_dir"
echo "  purge user data: $([[ $OPT_PURGE = 1 ]] && echo yes || echo no)"

if ! confirm "Proceed?" "N"; then
  log_info "Aborted."
  exit 0
fi

# build artifacts
[[ -d "$root/target" ]]               && run rm -rf "$root/target"
[[ -d "$root/frontend/node_modules" ]] && run rm -rf "$root/frontend/node_modules"
[[ -d "$root/frontend/build" ]]        && run rm -rf "$root/frontend/build"
[[ -d "$root/frontend/.svelte-kit" ]]  && run rm -rf "$root/frontend/.svelte-kit"
[[ -f "$root/.env" ]]                  && run rm -f  "$root/.env"

# installer marker
[[ -f "$marker" ]] && run rm -f "$marker"

if (( OPT_PURGE == 1 )); then
  log_warn "Purging user config + data (recordings, settings)."
  [[ -d "$config_dir" ]] && run rm -rf "$config_dir"
  [[ -d "$data_dir"   ]] && run rm -rf "$data_dir"
else
  log_info "User config + data preserved. Pass --purge to remove them."
fi

log_ok "Uninstall complete."
