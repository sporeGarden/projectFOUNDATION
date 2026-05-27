#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# primal_ipc.sh — Primal discovery and IPC helpers
#
# Sourced by foundation_validate.sh and other deploy scripts.
# Provides: discover_socket, discover_port, rpc_*, blake3_hash, hash_to_byte_array
#
# Transport resolution (UDS-first, TCP fallback):
#   1. Environment: ${PRIMAL}_SOCKET (UDS) or ${PRIMAL}_PORT (TCP)
#   2. XDG discovery socket: capability.resolve → result.socket / result.port
#   3. Config file: [sockets] (UDS) then [bootstrap_tcp] (TCP, dev-only)
#
# VPS deployments (--uds-only) should resolve at step 1 or 2 and never
# reach TCP bootstrap. Desktop/dev may use TCP during bringup.

DISCOVERY_FALLBACK_COUNT=0
DISCOVERY_UDS_COUNT=0
DISCOVERY_DEFAULTS_TOML="${DISCOVERY_DEFAULTS_TOML:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/discovery_defaults.toml}"

_resolve_config_value() {
    local section="$1" key="$2"
    if [[ -f "$DISCOVERY_DEFAULTS_TOML" ]]; then
        python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib
with open('$DISCOVERY_DEFAULTS_TOML', 'rb') as f:
    data = tomllib.load(f)
print(data.get('$section', {}).get('$key', ''))
" 2>/dev/null
    fi
}

# Resolve a primal's UDS socket path.
# Returns empty string if no socket is available.
discover_socket() {
    local name="$1"

    # 1. Explicit env: ${PRIMAL}_SOCKET
    local env_var="${name^^}_SOCKET"
    local env_val="${!env_var:-}"
    if [[ -n "$env_val" && -S "$env_val" ]]; then echo "$env_val"; return; fi

    # 2. Discovery socket: capability.resolve → result.socket
    local discovery_sock="${XDG_RUNTIME_DIR:-/tmp}/ecoPrimals/discovery.sock"
    if [[ -S "$discovery_sock" ]]; then
        local disc_resp sock_path
        disc_resp=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"capability.resolve\",\"params\":{\"primal\":\"$name\"},\"id\":1}" \
            | nc -w 2 -U "$discovery_sock" 2>/dev/null) || disc_resp=""
        sock_path=$(python3 -c "
import sys, json
try:
    r = json.loads(sys.argv[1])
    print(r.get('result',{}).get('socket',''))
except Exception:
    print('')
" "$disc_resp" 2>/dev/null)
        if [[ -n "$sock_path" && -S "$sock_path" ]]; then echo "$sock_path"; return; fi
    fi

    # 3. Config: [sockets] section with XDG expansion
    local config_path
    config_path=$(_resolve_config_value "sockets" "$name")
    if [[ -n "$config_path" ]]; then
        # Expand ${XDG_RUNTIME_DIR} in the path
        local expanded
        expanded=$(echo "$config_path" | sed "s|\${XDG_RUNTIME_DIR}|${XDG_RUNTIME_DIR:-/tmp}|g")
        if [[ -S "$expanded" ]]; then echo "$expanded"; return; fi
    fi

    # No UDS available
    echo ""
}

# Resolve a primal's TCP port (fallback when no UDS socket exists).
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
        default=$(_resolve_config_value "bootstrap_tcp" "$name")
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

PRIMAL_HOST="${PRIMAL_HOST:-127.0.0.1}"

# Generic UDS JSON-RPC call. Returns response or empty string on failure.
_rpc_uds() {
    local sock="$1" payload="$2"
    python3 -c "
import socket, sys, json
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.settimeout(10)
s.connect(sys.argv[1])
s.sendall((sys.argv[2] + '\n').encode())
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
" "$sock" "$payload" 2>/dev/null
}

rpc_nestgate() {
    local sock
    sock=$(discover_socket "nestgate")
    if [[ -n "$sock" ]]; then
        DISCOVERY_UDS_COUNT=$((DISCOVERY_UDS_COUNT + 1))
        _rpc_uds "$sock" "$1"
    else
        printf '%s\n' "$1" | nc -w 5 "$PRIMAL_HOST" "$NESTGATE_PORT" 2>/dev/null
    fi
}

rpc_rhizocrypt() {
    # rhizoCrypt prefers family-specific socket, then generic, then TCP
    local sock="${XDG_RUNTIME_DIR:-/tmp}/ecoPrimals/rhizocrypt-${FAMILY_ID:-}.sock"
    if [[ ! -S "$sock" ]]; then
        sock=$(discover_socket "rhizocrypt")
    fi
    if [[ -n "$sock" && -S "$sock" ]]; then
        DISCOVERY_UDS_COUNT=$((DISCOVERY_UDS_COUNT + 1))
        _rpc_uds "$sock" "$1"
    else
        printf '%s\n' "$1" | nc -w 5 "$PRIMAL_HOST" "$RHIZOCRYPT_PORT" 2>/dev/null
    fi
}

rpc_loamspine() {
    local sock
    sock=$(discover_socket "loamspine")
    if [[ -n "$sock" ]]; then
        DISCOVERY_UDS_COUNT=$((DISCOVERY_UDS_COUNT + 1))
        _rpc_uds "$sock" "$1"
    else
        curl -s -X POST "http://${PRIMAL_HOST}:${LOAMSPINE_PORT}" \
            -H 'Content-Type: application/json' -d "$1" 2>/dev/null
    fi
}

rpc_sweetgrass() {
    local sock
    sock=$(discover_socket "sweetgrass")
    if [[ -n "$sock" ]]; then
        DISCOVERY_UDS_COUNT=$((DISCOVERY_UDS_COUNT + 1))
        _rpc_uds "$sock" "$1"
    else
        curl -s -X POST "http://${PRIMAL_HOST}:${SWEETGRASS_PORT}/jsonrpc" \
            -H 'Content-Type: application/json' -d "$1" 2>/dev/null
    fi
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
