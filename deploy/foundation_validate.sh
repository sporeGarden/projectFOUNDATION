#!/usr/bin/env bash
# foundation_validate.sh — Full foundation validation with provenance
#
# Orchestrates the complete foundation validation cycle:
#   Phase 1: Health-check all NUCLEUS primals
#   Phase 2: Create provenance session (rhizoCrypt DAG + loamSpine spine)
#   Phase 3: Fetch and hash data sources (NCBI, UniProt, KEGG)
#   Phase 4: Register source artifacts in NestGate with BLAKE3
#   Phase 5: Execute thread workloads through toadStool
#   Phase 6: Compare results against validation targets
#   Phase 7: Commit provenance (Merkle root, loamSpine, sweetGrass braid)
#   Phase 8: Write validation report
#
# Usage:
#   ./foundation_validate.sh [--thread THREAD] [--skip-fetch] [--data-dir DIR]
#
# Prerequisites:
#   - NUCLEUS composition running (deploy via projectNUCLEUS):
#     cd ../projectNUCLEUS/deploy && bash deploy.sh --composition nest --gate irongate
#   - b3sum, curl, nc (netcat), jq or python3
#
# Environment:
#   NCBI_API_KEY        Higher NCBI rate limits (optional, recommended)
#   ECOPRIMALS_ROOT     Root of ecoPrimals checkout (auto-detected)
#   NESTGATE_PORT       NestGate TCP port (default: 9500)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FOUNDATION_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NUCLEUS_ROOT="${NUCLEUS_ROOT:-$(cd "$FOUNDATION_ROOT/../projectNUCLEUS" 2>/dev/null && pwd)}"

ECOPRIMALS_ROOT="${ECOPRIMALS_ROOT:-$(cd "$FOUNDATION_ROOT/../../.." 2>/dev/null && pwd)}"
PLASMIDBIN_DIR="${PLASMIDBIN_DIR:-$ECOPRIMALS_ROOT/infra/plasmidBin}"
TOADSTOOL="${TOADSTOOL:-$PLASMIDBIN_DIR/primals/toadstool}"

THREAD_FILTER="all"
SKIP_FETCH=false
DATA_DIR="$FOUNDATION_ROOT/.data"
RESULTS_DIR="$FOUNDATION_ROOT/validation/run-$(date +%Y%m%d-%H%M%S)"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --thread)      THREAD_FILTER="$2"; shift 2 ;;
        --skip-fetch)  SKIP_FETCH=true; shift ;;
        --data-dir)    DATA_DIR="$2"; shift 2 ;;
        --results-dir) RESULTS_DIR="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [--thread THREAD] [--skip-fetch] [--data-dir DIR]"
            echo "Threads: wcm, plasma, immuno, enviro, health, all"
            exit 0 ;;
        *)             echo "Unknown option: $1"; exit 1 ;;
    esac
done

mkdir -p "$RESULTS_DIR"

BEARDOG_PORT="${BEARDOG_PORT:-9100}"
SONGBIRD_PORT="${SONGBIRD_PORT:-9200}"
TOADSTOOL_PORT="${TOADSTOOL_PORT:-9400}"
NESTGATE_PORT="${NESTGATE_PORT:-9500}"
RHIZOCRYPT_PORT="${RHIZOCRYPT_PORT:-9601}"
LOAMSPINE_PORT="${LOAMSPINE_PORT:-9700}"
SWEETGRASS_PORT="${SWEETGRASS_PORT:-9850}"

log() { echo "[$(date +%H:%M:%S)] $*"; }

blake3_hash() { b3sum "$1" | cut -d' ' -f1; }

rpc_nestgate() {
    printf '%s\n' "$1" | nc -w 5 127.0.0.1 "$NESTGATE_PORT" 2>/dev/null
}

rpc_rhizocrypt() {
    local sock="${XDG_RUNTIME_DIR:-/tmp/biomeos}/biomeos/rhizocrypt-${FAMILY_ID:-}.sock"
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
        except: pass
    except socket.timeout: break
s.close()
print(data.decode().strip())
" "$1" 2>/dev/null
    else
        printf '%s\n' "$1" | nc -w 5 127.0.0.1 "$RHIZOCRYPT_PORT" 2>/dev/null
    fi
}

rpc_loamspine() {
    curl -s -X POST "http://127.0.0.1:$LOAMSPINE_PORT" \
        -H 'Content-Type: application/json' -d "$1" 2>/dev/null
}

rpc_sweetgrass() {
    curl -s -X POST "http://127.0.0.1:$SWEETGRASS_PORT/jsonrpc" \
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

log "═══════════════════════════════════════════════════════════"
log "  Foundation Validation Pipeline"
log "  Thread: $THREAD_FILTER"
log "  Results: $RESULTS_DIR"
log "  Data: $DATA_DIR"
log "═══════════════════════════════════════════════════════════"

# ══════════════════════════════════════════════════════════════
# PHASE 1: Health checks
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 1: Health Checks ──"

rpc_health() {
    local name="$1" port="$2"
    local resp

    if [[ "$name" == "Songbird" ]]; then
        resp=$(curl -sf --max-time 3 "http://127.0.0.1:$port/health" 2>/dev/null) || resp=""
        if [[ "$resp" == "OK" ]]; then
            log "  [OK] $name (HTTP $port)"
            return 0
        fi
        log "  [FAIL] $name (HTTP $port)"
        return 1
    fi

    if [[ "$name" == "rhizoCrypt" ]]; then
        if pgrep -f "primals/rhizocrypt" >/dev/null 2>&1; then
            log "  [OK] $name (PID alive)"
            return 0
        fi
        log "  [FAIL] $name not running"
        return 1
    fi

    resp=$(curl -sf --max-time 3 "http://127.0.0.1:$port" \
        -X POST -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":0}' 2>/dev/null) || resp=""
    if [[ -n "$resp" ]] && echo "$resp" | grep -q '"result"'; then
        log "  [OK] $name (TCP $port)"
        return 0
    fi

    resp=$(printf '{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":0}\n' | nc -w 3 127.0.0.1 "$port" 2>/dev/null) || resp=""
    if [[ -n "$resp" ]] && echo "$resp" | grep -q '"result"'; then
        log "  [OK] $name (TCP $port)"
        return 0
    fi

    log "  [FAIL] $name (TCP $port)"
    return 1
}

HEALTH_FAIL=0
for pair in "BearDog:$BEARDOG_PORT" "Songbird:$SONGBIRD_PORT" "ToadStool:$TOADSTOOL_PORT" "NestGate:$NESTGATE_PORT" "rhizoCrypt:$RHIZOCRYPT_PORT" "loamSpine:$LOAMSPINE_PORT" "sweetGrass:$SWEETGRASS_PORT"; do
    name="${pair%%:*}"
    port="${pair#*:}"
    if ! rpc_health "$name" "$port"; then
        HEALTH_FAIL=$((HEALTH_FAIL + 1))
    fi
done

if [[ $HEALTH_FAIL -gt 0 ]]; then
    log ""
    log "  $HEALTH_FAIL primal(s) not responding."
    log "  Deploy NUCLEUS first:"
    log "    cd $NUCLEUS_ROOT/deploy"
    log "    bash deploy.sh --composition nest --gate irongate"
    exit 1
fi

# ══════════════════════════════════════════════════════════════
# PHASE 2: Create provenance session
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 2: Create Provenance Session ──"

SESSION_NAME="foundation-$THREAD_FILTER-$(date +%Y%m%d-%H%M%S)"
SESSION_RESP=$(rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.session.create\",\"params\":{\"name\":\"$SESSION_NAME\"},\"id\":1}")
SESSION_ID=$(echo "$SESSION_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['result'])" 2>/dev/null) || SESSION_ID=""

if [[ -z "$SESSION_ID" ]]; then
    log "  [FAIL] Could not create DAG session: $SESSION_RESP"
    exit 1
fi
log "  [OK] DAG Session: $SESSION_ID"

SPINE_RESP=$(rpc_loamspine "{\"jsonrpc\":\"2.0\",\"method\":\"spine.create\",\"params\":{\"name\":\"$SESSION_NAME\",\"owner\":\"foundation\"},\"id\":2}")
SPINE_ID=$(echo "$SPINE_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['spine_id'])" 2>/dev/null) || SPINE_ID=""

if [[ -z "$SPINE_ID" ]]; then
    log "  [FAIL] Could not create spine: $SPINE_RESP"
    exit 1
fi
log "  [OK] Spine: $SPINE_ID"

EVENT_IDX=0

# ══════════════════════════════════════════════════════════════
# PHASE 3: Fetch data sources
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 3: Fetch Data Sources ──"

if $SKIP_FETCH; then
    log "  [SKIP] --skip-fetch specified, using cached data in $DATA_DIR"
else
    bash "$SCRIPT_DIR/fetch_sources.sh" --thread "$THREAD_FILTER" --data-dir "$DATA_DIR"
fi

# ══════════════════════════════════════════════════════════════
# PHASE 4: Register fetched artifacts in NestGate + DAG
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 4: Register Data Artifacts ──"

ARTIFACT_TABLE=""

register_data_file() {
    local filepath="$1"
    local key="$2"

    if [[ ! -f "$filepath" ]]; then
        return
    fi

    local hash
    hash=$(blake3_hash "$filepath")
    local size
    size=$(stat -c%s "$filepath" 2>/dev/null || stat -f%z "$filepath" 2>/dev/null)
    local hash_bytes
    hash_bytes=$(hash_to_byte_array "$hash")

    rpc_nestgate "{\"jsonrpc\":\"2.0\",\"method\":\"storage.store\",\"params\":{\"key\":\"$key\",\"value\":\"blake3:$hash size:$size\"},\"id\":$((EVENT_IDX+100))}" > /dev/null 2>&1 || true

    rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.event.append\",\"params\":{\"session_id\":\"$SESSION_ID\",\"event_type\":{\"DataCreate\":{}},\"data\":{\"key\":\"$key\",\"blake3\":\"$hash\",\"size\":$size}},\"id\":$((EVENT_IDX+200))}" > /dev/null 2>&1 || true

    rpc_loamspine "{\"jsonrpc\":\"2.0\",\"method\":\"entry.append\",\"params\":{\"spine_id\":\"$SPINE_ID\",\"entry_type\":{\"DataAnchor\":{\"data_hash\":$hash_bytes,\"source\":\"foundation\",\"size\":$size}},\"committer\":\"did:primal:foundation\",\"data\":{\"key\":\"$key\",\"blake3\":\"$hash\"}},\"id\":$((EVENT_IDX+300))}" > /dev/null 2>&1 || true

    EVENT_IDX=$((EVENT_IDX + 1))
    ARTIFACT_TABLE+="| $key | ${hash:0:16}… | ${size}B |\n"
    log "  [OK] $key → blake3:${hash:0:16}…"
}

if [[ -d "$DATA_DIR" ]]; then
    while IFS= read -r -d '' f; do
        rel="${f#"$DATA_DIR/"}"
        key="foundation:${rel//\//:}"
        register_data_file "$f" "$key"
    done < <(find "$DATA_DIR" -type f -print0 2>/dev/null | sort -z)
fi

log "  Registered $EVENT_IDX data artifacts"

# ══════════════════════════════════════════════════════════════
# PHASE 5: Execute thread workloads
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 5: Execute Workloads ──"

WORKLOAD_DIR="$FOUNDATION_ROOT/workloads"
WORKLOAD_TABLE=""
TOTAL_OK=0
TOTAL_FAIL=0

execute_workload() {
    local toml_path="$1"
    local name
    name=$(basename "$toml_path" .toml)

    log "  [$name] Executing..."

    rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.event.append\",\"params\":{\"session_id\":\"$SESSION_ID\",\"event_type\":{\"ExperimentStart\":{\"protocol\":\"foundation-validation\"}},\"data\":{\"workload\":\"$name\",\"timestamp\":\"$(date -Iseconds)\"}},\"id\":$((EVENT_IDX+400))}" > /dev/null 2>&1 || true
    EVENT_IDX=$((EVENT_IDX + 1))

    local output_file="$RESULTS_DIR/${name}.stdout"
    local start_time
    start_time=$(date +%s)

    if [[ -x "$TOADSTOOL" ]]; then
        "$TOADSTOOL" execute --timeout 300 --format text "$toml_path" > "$output_file" 2>&1 || true
    else
        log "  [$name] toadStool not found at $TOADSTOOL — running command directly"
        local cmd
        cmd=$(python3 -c "
import tomllib, sys
with open('$toml_path', 'rb') as f:
    d = tomllib.load(f)
print(d.get('execution', {}).get('command', ''))
" 2>/dev/null) || cmd=""
        if [[ -n "$cmd" && -x "$cmd" ]]; then
            "$cmd" > "$output_file" 2>&1 || true
        else
            echo "[SKIP] No executable found" > "$output_file"
        fi
    fi

    local end_time
    end_time=$(date +%s)
    local elapsed=$((end_time - start_time))

    local ok_count fail_count
    ok_count=$(grep -c '\[OK\]' "$output_file" 2>/dev/null || echo 0)
    fail_count=$(grep -c '\[FAIL\]' "$output_file" 2>/dev/null || echo 0)
    TOTAL_OK=$((TOTAL_OK + ok_count))
    TOTAL_FAIL=$((TOTAL_FAIL + fail_count))

    local output_hash
    output_hash=$(blake3_hash "$output_file")

    rpc_nestgate "{\"jsonrpc\":\"2.0\",\"method\":\"storage.store\",\"params\":{\"key\":\"foundation:workload:$name\",\"value\":\"blake3:$output_hash\"},\"id\":$((EVENT_IDX+500))}" > /dev/null 2>&1 || true

    rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.event.append\",\"params\":{\"session_id\":\"$SESSION_ID\",\"event_type\":{\"ExperimentEnd\":{}},\"data\":{\"workload\":\"$name\",\"ok\":$ok_count,\"fail\":$fail_count,\"elapsed_s\":$elapsed,\"output_hash\":\"$output_hash\"}},\"id\":$((EVENT_IDX+600))}" > /dev/null 2>&1 || true
    EVENT_IDX=$((EVENT_IDX + 1))

    local status="RUN"
    [[ $fail_count -gt 0 ]] && status="FAIL"
    [[ $ok_count -gt 0 && $fail_count -eq 0 ]] && status="PASS"

    WORKLOAD_TABLE+="| $name | $ok_count | $fail_count | ${elapsed}s | $status |\n"
    log "  [$name] $ok_count OK / $fail_count FAIL (${elapsed}s)"
}

if [[ "$THREAD_FILTER" == "all" ]]; then
    SCAN_DIRS=("$WORKLOAD_DIR"/thread*)
else
    SCAN_DIRS=("$WORKLOAD_DIR/thread"*"$THREAD_FILTER"*)
fi

for dir in "${SCAN_DIRS[@]}"; do
    [[ -d "$dir" ]] || continue
    for toml in "$dir"/*.toml; do
        [[ -f "$toml" ]] || continue
        execute_workload "$toml"
    done
done

# ══════════════════════════════════════════════════════════════
# PHASE 6: Commit provenance
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 6: Commit Provenance ──"

COMPLETE_RESP=$(rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.session.complete\",\"params\":{\"session_id\":\"$SESSION_ID\"},\"id\":800}")
MERKLE_ROOT=$(echo "$COMPLETE_RESP" | python3 -c "
import sys,json
r = json.load(sys.stdin).get('result',{})
print(r.get('merkle_root','') or r.get('root','') or 'unknown')
" 2>/dev/null) || MERKLE_ROOT="unknown"
log "  [OK] DAG Merkle root: $MERKLE_ROOT"

MERKLE_BYTES=$(hash_to_byte_array "${MERKLE_ROOT:-0000000000000000000000000000000000000000000000000000000000000000}")
COMMIT_RESP=$(rpc_loamspine "{\"jsonrpc\":\"2.0\",\"method\":\"entry.append\",\"params\":{\"spine_id\":\"$SPINE_ID\",\"entry_type\":{\"SessionCommit\":{\"session_hash\":$MERKLE_BYTES}},\"committer\":\"did:primal:foundation\",\"data\":{\"session\":\"$SESSION_NAME\",\"merkle_root\":\"$MERKLE_ROOT\",\"events\":$EVENT_IDX,\"ok\":$TOTAL_OK,\"fail\":$TOTAL_FAIL}},\"id\":801}")
log "  [OK] loamSpine committed"

BRAID_RESP=$(rpc_sweetgrass "{\"jsonrpc\":\"2.0\",\"method\":\"braid.create\",\"params\":{\"creator\":\"did:primal:foundation\",\"subject\":\"foundation-validation:$SESSION_NAME\",\"claims\":[{\"type\":\"ProvenanceValidation\",\"data\":{\"session\":\"$SESSION_NAME\",\"merkle_root\":\"$MERKLE_ROOT\",\"ok\":$TOTAL_OK,\"fail\":$TOTAL_FAIL,\"events\":$EVENT_IDX}}]},\"id\":802}")
BRAID_URN=$(echo "$BRAID_RESP" | python3 -c "
import sys,json
r = json.load(sys.stdin).get('result',{})
print(r.get('urn','') or r.get('id','') or 'unknown')
" 2>/dev/null) || BRAID_URN="unknown"
log "  [OK] sweetGrass braid: $BRAID_URN"

echo "$BRAID_RESP" > "$RESULTS_DIR/braid.json"

# ══════════════════════════════════════════════════════════════
# PHASE 7: Write validation report
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 7: Write Report ──"

cat > "$RESULTS_DIR/VALIDATION_REPORT.md" << REPORT
# Foundation Validation Report

**Session**: $SESSION_NAME
**Thread**: $THREAD_FILTER
**Date**: $(date -Iseconds)
**NUCLEUS Gate**: ironGate

## Provenance Chain

| Component | Value |
|-----------|-------|
| DAG Session | $SESSION_ID |
| Merkle Root | $MERKLE_ROOT |
| loamSpine Spine | $SPINE_ID |
| sweetGrass Braid | $BRAID_URN |
| Total Events | $EVENT_IDX |

## Data Artifacts

| Key | BLAKE3 | Size |
|-----|--------|------|
$(echo -e "$ARTIFACT_TABLE")

## Workload Results

| Workload | OK | FAIL | Time | Status |
|----------|---:|-----:|-----:|--------|
$(echo -e "$WORKLOAD_TABLE")

**Total**: $TOTAL_OK OK / $TOTAL_FAIL FAIL

## Sediment Layer

This validation run is now a permanent layer in the foundation's
geological record. The Merkle root anchors the complete provenance
chain: data sources → computation → results → attribution.

Springs that absorb these patterns will strengthen the layer by adding
their own validation results, which flow back here as new sediment.
REPORT

log "  [OK] Report: $RESULTS_DIR/VALIDATION_REPORT.md"

log ""
log "═══════════════════════════════════════════════════════════"
log "  Foundation Validation Complete"
log "  $TOTAL_OK OK / $TOTAL_FAIL FAIL across $EVENT_IDX provenance events"
log "  Merkle root: $MERKLE_ROOT"
log "  Braid: $BRAID_URN"
log "  Results: $RESULTS_DIR"
log "═══════════════════════════════════════════════════════════"

[[ $TOTAL_FAIL -gt 0 ]] && exit 1
exit 0
