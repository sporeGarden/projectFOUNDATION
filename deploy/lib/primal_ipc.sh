#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# primal_ipc.sh — Primal discovery and IPC helpers
#
# Sourced by foundation_validate.sh and other deploy scripts.
# Provides: discover_port, rpc_*, blake3_hash, hash_to_byte_array

# Capability-based primal discovery: each primal's port is resolved from
# environment (explicit config), then XDG runtime discovery socket, then
# well-known defaults.
#
# The fallback defaults exist only for bootstrap/dev environments where the
# discovery socket isn't running yet. In production, all resolution should
# go through the discovery socket or environment variables.
DISCOVERY_FALLBACK_COUNT=0
DISCOVERY_DEFAULTS_TOML="${DISCOVERY_DEFAULTS_TOML:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/discovery_defaults.toml}"

_resolve_default_port() {
    local name="$1"
    if [[ -f "$DISCOVERY_DEFAULTS_TOML" ]]; then
        python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib
with open('$DISCOVERY_DEFAULTS_TOML', 'rb') as f:
    data = tomllib.load(f)
print(data.get('defaults', {}).get('$name', ''))
" 2>/dev/null
    fi
}

discover_port() {
    local name="$1" default="${2:-}"
    local env_var="${name^^}_PORT"
    local env_val="${!env_var:-}"
    if [[ -n "$env_val" ]]; then echo "$env_val"; return; fi

    local discovery_sock="${XDG_RUNTIME_DIR:-/tmp}/ecoPrimals/discovery.sock"
    if [[ -S "$discovery_sock" ]]; then
        local disc_resp port
        disc_resp=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"capability.resolve\",\"params\":{\"primal\":\"$name\"},\"id\":1}" \
            | nc -w 2 -U "$discovery_sock" 2>/dev/null) || disc_resp=""
        port=$(python3 -c "
import sys, json
try:
    r = json.loads(sys.argv[1])
    print(r.get('result',{}).get('port',''))
except Exception:
    print('')
" "$disc_resp" 2>/dev/null)
        if [[ -n "$port" ]]; then echo "$port"; return; fi
    fi

    if [[ -z "$default" ]]; then
        default=$(_resolve_default_port "$name")
    fi

    DISCOVERY_FALLBACK_COUNT=$((DISCOVERY_FALLBACK_COUNT + 1))
    echo "$default"
}

blake3_hash() {
    if command -v b3sum >/dev/null 2>&1; then
        b3sum "$1" | cut -d' ' -f1
    else
        python3 -c "
import sys
try:
    import blake3
    print(blake3.blake3(open(sys.argv[1], 'rb').read()).hexdigest())
except ImportError:
    print('no-blake3-tool', file=sys.stderr)
    sys.exit(1)
" "$1" 2>/dev/null || echo "no-hash"
    fi
}

# RPC host resolved at runtime — never assume localhost.
PRIMAL_HOST="${PRIMAL_HOST:-127.0.0.1}"

rpc_nestgate() {
    printf '%s\n' "$1" | nc -w 5 "$PRIMAL_HOST" "$NESTGATE_PORT" 2>/dev/null
}

rpc_rhizocrypt() {
    local sock="${XDG_RUNTIME_DIR:-/tmp}/ecoPrimals/rhizocrypt-${FAMILY_ID:-}.sock"
    if [[ -S "$sock" ]]; then
        python3 -c "
import socket, sys, json
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.settimeout(10)
s.connect('$sock')
s.sendall((sys.argv[1] + '\n').encode())
data = b''
while True:
    try:
        chunk = s.recv(65536)
        if not chunk: break
        data += chunk
        try:
            json.loads(data)
            break
        except Exception: pass
    except socket.timeout: break
s.close()
print(data.decode().strip())
" "$1" 2>/dev/null
    else
        printf '%s\n' "$1" | nc -w 5 "$PRIMAL_HOST" "$RHIZOCRYPT_PORT" 2>/dev/null
    fi
}

rpc_loamspine() {
    curl -s -X POST "http://${PRIMAL_HOST}:${LOAMSPINE_PORT}" \
        -H 'Content-Type: application/json' -d "$1" 2>/dev/null
}

rpc_sweetgrass() {
    curl -s -X POST "http://${PRIMAL_HOST}:${SWEETGRASS_PORT}/jsonrpc" \
        -H 'Content-Type: application/json' -d "$1" 2>/dev/null
}

hash_to_byte_array() {
    local hex="$1"
    local arr="["
    for i in $(seq 0 2 62); do
        local byte=$((16#${hex:$i:2}))
        [ "$i" -gt 0 ] && arr+=","
        arr+="$byte"
    done
    arr+="]"
    echo "$arr"
}
