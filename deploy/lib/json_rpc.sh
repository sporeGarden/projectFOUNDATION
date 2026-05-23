#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# json_rpc.sh — Typed JSON-RPC response parsing helpers
#
# Sourced by foundation_validate.sh and primal_ipc.sh consumers.
# Replaces grep '"result"' with deterministic Python JSON parsing.

# Check if a JSON-RPC response contains a successful result.
# Returns 0 (true) if result field exists and is non-null.
# Returns 1 (false) on error, empty, or malformed response.
rpc_has_result() {
    local response="$1"
    [[ -z "$response" ]] && return 1
    python3 -c "
import sys, json
try:
    r = json.loads(sys.argv[1])
    if 'result' in r and r['result'] is not None:
        sys.exit(0)
    sys.exit(1)
except Exception:
    sys.exit(1)
" "$response" 2>/dev/null
}

# Extract a field from a JSON-RPC result.
# Usage: value=$(rpc_extract_field "$response" "session_id")
rpc_extract_field() {
    local response="$1" field="$2"
    python3 -c "
import sys, json
try:
    r = json.loads(sys.argv[1])
    result = r.get('result', {})
    if isinstance(result, dict):
        v = result.get(sys.argv[2], '')
        print(v if v is not None else '')
    else:
        print(result if result is not None else '')
except Exception:
    print('')
" "$response" "$field" 2>/dev/null
}

# Check if a JSON-RPC response contains an error.
# Returns 0 (true) if error field exists.
rpc_has_error() {
    local response="$1"
    [[ -z "$response" ]] && return 0
    python3 -c "
import sys, json
try:
    r = json.loads(sys.argv[1])
    sys.exit(0 if 'error' in r else 1)
except Exception:
    sys.exit(0)
" "$response" 2>/dev/null
}

# Extract error message from JSON-RPC error response.
rpc_error_message() {
    local response="$1"
    python3 -c "
import sys, json
try:
    r = json.loads(sys.argv[1])
    err = r.get('error', {})
    if isinstance(err, dict):
        print(err.get('message', 'unknown error'))
    else:
        print(str(err))
except Exception:
    print('malformed response')
" "$response" 2>/dev/null
}

# Parse workload stdout for structured results.
# Looks for JSON lines with {target_id, status, value} first,
# then falls back to counting [OK]/[FAIL]/[SKIP] tags.
# Outputs: ok_count fail_count skip_count
parse_workload_results() {
    local output_file="$1"
    python3 -c "
import sys, json, re

ok = fail = skip = 0
json_found = False

with open(sys.argv[1]) as f:
    for line in f:
        line = line.strip()
        # Try structured JSON lines first
        if line.startswith('{'):
            try:
                obj = json.loads(line)
                status = obj.get('status', '').upper()
                if status in ('OK', 'PASS'):
                    ok += 1; json_found = True
                elif status == 'FAIL':
                    fail += 1; json_found = True
                elif status == 'SKIP':
                    skip += 1; json_found = True
                continue
            except json.JSONDecodeError:
                pass
        # Fall back to tag counting
        if not json_found:
            ok += line.count('[OK]')
            fail += line.count('[FAIL]')
            skip += line.count('[SKIP]')

print(f'{ok} {fail} {skip}')
" "$output_file" 2>/dev/null || echo "0 0 0"
}

# Compute trio provenance state from session/spine/braid availability.
# Single source of truth — call once, reuse in report/toml/json.
compute_trio_state() {
    local session="$1" spine="$2" braid="$3"
    if [[ -n "$braid" && "$braid" != "unknown" ]]; then
        echo "full"
    elif [[ -n "$spine" && "$spine" != "unknown" ]]; then
        echo "dag_spine"
    elif [[ -n "$session" && "$session" != "unknown" ]]; then
        echo "dag_only"
    else
        echo "standalone"
    fi
}

# Human-readable trio state label.
trio_state_label() {
    case "$1" in
        full)       echo "Full (DAG+spine+braid)" ;;
        dag_spine)  echo "Partial (DAG+spine)" ;;
        dag_only)   echo "Partial (DAG only)" ;;
        *)          echo "Standalone (no trio)" ;;
    esac
}
