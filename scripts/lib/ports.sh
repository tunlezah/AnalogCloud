#!/usr/bin/env bash
# Port discovery and conflict resolution.
#
# Strategy: try `ss` first (always present on modern Linux), fall back to
# `lsof`, then to /proc/net/tcp. On macOS, `lsof` is the canonical tool.
# We must never *assume* a probe tool exists — that's exactly the kind of
# unchecked assumption this installer is trying to avoid.

[[ -n "${__ANALOG_CLOUD_PORTS_SH:-}" ]] && return 0
__ANALOG_CLOUD_PORTS_SH=1

# shellcheck source=common.sh
. "$(dirname "${BASH_SOURCE[0]}")/common.sh"

# port_in_use <port>
#
# Returns 0 if something is listening on the port (IPv4 or IPv6, any
# interface), 1 if it appears free, 2 if we couldn't determine.
port_in_use() {
  local port="$1"

  if has_cmd ss; then
    # -H suppresses the header; -t TCP; -l listening; -n numeric.
    if ss -Hltn "( sport = :$port )" 2>/dev/null | grep -q .; then
      return 0
    fi
    return 1
  fi

  if has_cmd lsof; then
    if lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
      return 0
    fi
    return 1
  fi

  if has_cmd netstat; then
    if netstat -lnt 2>/dev/null | awk '{print $4}' | grep -Eq "[:.]$port\$"; then
      return 0
    fi
    return 1
  fi

  # Best-effort fallback. /proc/net/tcp lists local ports in hex.
  if [[ -r /proc/net/tcp ]]; then
    local hex
    hex=$(printf '%04X' "$port")
    if awk 'NR>1 && $4=="0A"' /proc/net/tcp 2>/dev/null \
        | awk -F'[ :]+' '{print $4}' \
        | grep -qx "$hex"; then
      return 0
    fi
    return 1
  fi

  return 2
}

# port_owner <port>
#
# Best-effort: identify the listening process(es). Echoes a short string
# like "1234 nginx" or "<unknown>" — never fails. Useful for telling the
# user *why* a port is taken.
port_owner() {
  local port="$1"
  if has_cmd ss; then
    local out
    out=$(ss -Hltnp "( sport = :$port )" 2>/dev/null | head -n1)
    if [[ -n "$out" ]]; then
      # ss prints users:(("name",pid=NNN,fd=N))
      local pid name
      pid=$(printf '%s' "$out" | grep -oE 'pid=[0-9]+' | head -n1 | cut -d= -f2)
      name=$(printf '%s' "$out" | grep -oE '"[^"]+"' | head -n1 | tr -d '"')
      if [[ -n "$pid" || -n "$name" ]]; then
        printf '%s %s\n' "${pid:-?}" "${name:-?}"
        return 0
      fi
    fi
  fi
  if has_cmd lsof; then
    local row
    row=$(lsof -nP -iTCP:"$port" -sTCP:LISTEN 2>/dev/null | awk 'NR==2 {print $2, $1}')
    if [[ -n "$row" ]]; then
      printf '%s\n' "$row"
      return 0
    fi
  fi
  printf '<unknown>\n'
}

# valid_port <value>
valid_port() {
  local p="$1"
  [[ "$p" =~ ^[0-9]+$ ]] || return 1
  (( p >= 1 && p <= 65535 ))
}

# find_free_port <start>
#
# Walks upward from $start looking for a free port, skipping a small set
# of well-known services so we don't recommend something inappropriate
# (e.g. landing on 8000 next to a dev server, etc.). Returns the first
# free port, or 1 if no candidate found in the window.
find_free_port() {
  local start="${1:-7777}"
  local end=$((start + 200))
  local p
  for (( p=start; p<=end; p++ )); do
    # avoid recommending obviously-conflicting common ports
    case "$p" in
      8080|8000|3000|5000|7000|9000|9090) continue ;;
    esac
    if port_in_use "$p"; then continue; fi
    printf '%d\n' "$p"
    return 0
  done
  return 1
}

# resolve_port <label> <desired_port>
#
# Interactive workflow: probe the desired port, and if taken, surface the
# owner and offer the user a free suggestion. Echoes the final chosen
# port to stdout. Logs go to stderr so the caller can `port=$(resolve_port …)`.
resolve_port() {
  local label="$1" desired="$2"
  local chosen="$desired"

  while :; do
    if ! valid_port "$chosen"; then
      log_warn "Not a valid port: $chosen" >&2
      chosen=$(prompt "Pick a port for $label" "$desired")
      continue
    fi

    if port_in_use "$chosen"; then
      local owner
      owner=$(port_owner "$chosen")
      log_warn "Port $chosen is in use ($owner) — needed for $label" >&2
      local suggestion
      if suggestion=$(find_free_port $((chosen + 1))); then
        log_info "Suggested free port: $suggestion" >&2
      else
        suggestion=""
      fi
      chosen=$(prompt "Pick a different port for $label" "${suggestion:-$desired}")
      continue
    fi

    # passed all checks
    printf '%d\n' "$chosen"
    return 0
  done
}
