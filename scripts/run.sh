#!/usr/bin/env bash
#
# Convenience launcher for development. Loads the repo-local .env,
# starts the backend, then the frontend dev server, and forwards
# Ctrl-C to both.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
. "$SCRIPT_DIR/lib/common.sh"

root=$(repo_root)
env_path="$root/.env"

if [[ ! -f "$env_path" ]]; then
  die "$env_path not found. Run scripts/install.sh first."
fi

set -a
# shellcheck disable=SC1090
. "$env_path"
set +a

log_info "Backend  -> ${ANALOG_CLOUD_BIND:-127.0.0.1:7777}"
log_info "Frontend -> http://127.0.0.1:${ANALOG_CLOUD_FRONTEND_PORT:-5173}"

pids=()
cleanup() {
  log_info "Shutting down…"
  for pid in "${pids[@]:-}"; do
    kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
}
trap cleanup INT TERM EXIT

( cd "$root" && cargo run -p analog-cloud-server ) &
pids+=($!)

( cd "$root/frontend" && pnpm dev ) &
pids+=($!)

wait
