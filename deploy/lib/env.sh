#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# env.sh — Centralized environment variable bootstrap for deploy scripts.
#
# Sourced by foundation_validate.sh, fetch_sources.sh, backfill_hashes.sh.
# Resolves ecosystem paths and primal identity from environment or discovery.
#
# Callers must set FOUNDATION_ROOT before sourcing this file.

: "${FOUNDATION_ROOT:?FOUNDATION_ROOT must be set before sourcing env.sh}"

ECOPRIMALS_ROOT="${ECOPRIMALS_ROOT:-$(cd "$FOUNDATION_ROOT/../.." 2>/dev/null && pwd || echo "$FOUNDATION_ROOT/../..")}"
SPRINGS_ROOT="${SPRINGS_ROOT:-${ECOPRIMALS_ROOT}/springs}"
NUCLEUS_ROOT="${NUCLEUS_ROOT:-$(cd "$FOUNDATION_ROOT/../projectNUCLEUS" 2>/dev/null && pwd || echo "$FOUNDATION_ROOT/../projectNUCLEUS")}"
PLASMIDBIN_DIR="${PLASMIDBIN_DIR:-${ECOPRIMALS_ROOT}/infra/plasmidBin}"

# Primal identity — resolved from env or discovery socket.
# Empty FAMILY_ID is valid (pre-bootstrap, no discovery).
FAMILY_ID="${FAMILY_ID:-}"
if [[ -z "$FAMILY_ID" ]]; then
    local_discovery="${XDG_RUNTIME_DIR:-/tmp}/ecoPrimals/discovery.sock"
    if [[ -S "$local_discovery" ]]; then
        FAMILY_ID=$(echo '{"jsonrpc":"2.0","method":"family.id","params":{},"id":1}' \
            | nc -w 2 -U "$local_discovery" 2>/dev/null \
            | python3 -c "
import sys, json
try:
    r = json.loads(sys.stdin.read())
    print(r.get('result', {}).get('family_id', ''))
except Exception:
    print('')
" 2>/dev/null) || FAMILY_ID=""
    fi
fi

export ECOPRIMALS_ROOT SPRINGS_ROOT NUCLEUS_ROOT PLASMIDBIN_DIR FAMILY_ID
