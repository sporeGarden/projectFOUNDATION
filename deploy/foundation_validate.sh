#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
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
#   - b3sum, curl, nc (netcat), python3
#
# Environment:
#   NCBI_API_KEY        Higher NCBI rate limits (optional, recommended)
#   ECOPRIMALS_ROOT     Root of ecoPrimals checkout (auto-detected)
#   NESTGATE_PORT       NestGate TCP port (default: 9500)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FOUNDATION_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NUCLEUS_ROOT="${NUCLEUS_ROOT:-$(cd "$FOUNDATION_ROOT/../projectNUCLEUS" 2>/dev/null && pwd || echo "$FOUNDATION_ROOT/../projectNUCLEUS")}"

ECOPRIMALS_ROOT="${ECOPRIMALS_ROOT:-$(cd "$FOUNDATION_ROOT/../.." 2>/dev/null && pwd || echo "$FOUNDATION_ROOT/../..")}"
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
            thread_help_text
            exit 0 ;;
        *)             echo "Unknown option: $1"; exit 1 ;;
    esac
done

mkdir -p "$RESULTS_DIR"

# Source shared IPC helpers (discovery, RPC clients, hashing)
# shellcheck source=lib/primal_ipc.sh
source "$SCRIPT_DIR/lib/primal_ipc.sh"
# shellcheck source=lib/json_rpc.sh
source "$SCRIPT_DIR/lib/json_rpc.sh"
# shellcheck source=lib/thread_registry.sh
source "$SCRIPT_DIR/lib/thread_registry.sh"

# Gate name: read from env, discovery, or default
GATE_NAME="${GATE_NAME:-${NUCLEUS_GATE:-irongate}}"

BEARDOG_PORT=$(discover_port "beardog")
SONGBIRD_PORT=$(discover_port "songbird")
TOADSTOOL_PORT=$(discover_port "toadstool")
NESTGATE_PORT=$(discover_port "nestgate")
RHIZOCRYPT_PORT=$(discover_port "rhizocrypt")
LOAMSPINE_PORT=$(discover_port "loamspine")
SWEETGRASS_PORT=$(discover_port "sweetgrass")

log() { echo "[$(date +%H:%M:%S)] $*"; }

if [[ $DISCOVERY_FALLBACK_COUNT -gt 0 ]]; then
    log "[WARN] $DISCOVERY_FALLBACK_COUNT primal(s) resolved via TCP bootstrap defaults."
    log "  VPS deployments should use --uds-only (discovery.sock or {PRIMAL}_SOCKET env)."
fi


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
    local resp lname

    # Normalize to lowercase for discover_socket lookup
    lname=$(echo "$name" | tr '[:upper:]' '[:lower:]')

    # UDS-first: try socket for any primal
    local sock
    sock=$(discover_socket "$lname")
    if [[ -n "$sock" ]]; then
        resp=$(_rpc_uds "$sock" '{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":0}')
        if rpc_has_result "$resp"; then
            log "  [OK] $name (UDS)"
            return 0
        fi
    fi

    # Songbird: HTTP GET /health (no JSON-RPC health)
    if [[ "$name" == "Songbird" ]]; then
        resp=$(curl -sf --max-time 3 "http://${PRIMAL_HOST}:$port/health" 2>/dev/null) || resp=""
        if [[ "$resp" == "OK" ]]; then
            log "  [OK] $name (HTTP $port)"
            return 0
        fi
        log "  [FAIL] $name"
        return 1
    fi

    # TCP fallback: curl HTTP POST, then raw nc
    resp=$(curl -sf --max-time 3 "http://${PRIMAL_HOST}:$port" \
        -X POST -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":0}' 2>/dev/null) || resp=""
    if rpc_has_result "$resp"; then
        log "  [OK] $name (TCP $port)"
        return 0
    fi

    resp=$(printf '{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":0}\n' | nc -w 3 "$PRIMAL_HOST" "$port" 2>/dev/null) || resp=""
    if rpc_has_result "$resp"; then
        log "  [OK] $name (TCP $port)"
        return 0
    fi

    log "  [FAIL] $name"
    return 1
}

HEALTH_FAIL=0
HEALTH_WARN=0
REQUIRED_PRIMALS=(NestGate rhizoCrypt loamSpine)

is_required_primal() {
    local name="$1"
    for req in "${REQUIRED_PRIMALS[@]}"; do
        [[ "$req" == "$name" ]] && return 0
    done
    return 1
}

for pair in "BearDog:$BEARDOG_PORT" "Songbird:$SONGBIRD_PORT" "ToadStool:$TOADSTOOL_PORT" "NestGate:$NESTGATE_PORT" "rhizoCrypt:$RHIZOCRYPT_PORT" "loamSpine:$LOAMSPINE_PORT" "sweetGrass:$SWEETGRASS_PORT"; do
    name="${pair%%:*}"
    port="${pair#*:}"
    if ! rpc_health "$name" "$port"; then
        if is_required_primal "$name"; then
            HEALTH_FAIL=$((HEALTH_FAIL + 1))
        else
            HEALTH_WARN=$((HEALTH_WARN + 1))
            log "  [WARN] $name not available — non-critical, continuing"
        fi
    fi
done

if [[ $HEALTH_FAIL -gt 0 ]]; then
    log ""
    log "  $HEALTH_FAIL required primal(s) not responding (provenance trio: NestGate, rhizoCrypt, loamSpine)."
    log "  Deploy NUCLEUS first:"
    log "    cd $NUCLEUS_ROOT/deploy"
    log "    bash deploy.sh --composition nest --gate $GATE_NAME"
    exit 1
fi

if [[ $DISCOVERY_UDS_COUNT -gt 0 ]]; then
    log "  $DISCOVERY_UDS_COUNT primal(s) using UDS transport (VPS-standard)"
fi
if [[ $HEALTH_WARN -gt 0 ]]; then
    log "  $HEALTH_WARN optional primal(s) not available — provenance chain will be partial"
fi

# ══════════════════════════════════════════════════════════════
# PHASE 2: Create provenance session
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 2: Create Provenance Session ──"

SESSION_NAME="foundation-$THREAD_FILTER-$(date +%Y%m%d-%H%M%S)"
SESSION_RESP=$(rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.session.create\",\"params\":{\"session_type\":\"Experiment\",\"description\":\"$SESSION_NAME\"},\"id\":1}")
SESSION_ID=$(rpc_extract_field "$SESSION_RESP" "session_id")

if [[ -z "$SESSION_ID" ]]; then
    log "  [FAIL] Could not create DAG session: $SESSION_RESP"
    exit 1
fi
log "  [OK] DAG Session: $SESSION_ID"

SPINE_RESP=$(rpc_loamspine "{\"jsonrpc\":\"2.0\",\"method\":\"spine.create\",\"params\":{\"name\":\"$SESSION_NAME\",\"owner\":\"foundation\"},\"id\":2}")
SPINE_ID=$(rpc_extract_field "$SPINE_RESP" "spine_id")

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

SOURCES_MANIFEST_DIR="$FOUNDATION_ROOT/data/sources"

# Map (database, accession, format) → expected file path under DATA_DIR.
# Mirrors the layout created by fetch_sources.sh fetch functions.
resolve_data_path() {
    local db="$1" acc="$2" fmt="$3"
    case "$db" in
        "ncbi nucleotide"|"ncbi")
            if [[ "$fmt" == "fasta" ]]; then
                echo "$DATA_DIR/ncbi/nucleotide/${acc}.fasta"
            else
                echo "$DATA_DIR/ncbi/nucleotide/${acc}.gb"
            fi ;;
        "ncbi assembly")    echo "$DATA_DIR/ncbi/assembly/${acc}_report.txt" ;;
        "uniprot")          echo "$DATA_DIR/uniprot/${acc}.fasta.gz" ;;
        "kegg")             echo "$DATA_DIR/kegg/${acc}_pathways.json" ;;
        "ncbi bioproject"|"ncbi sra")
                            echo "$DATA_DIR/ncbi/bioproject/${acc}.xml" ;;
        *)                  echo "" ;;
    esac
}

register_from_manifest() {
    local manifest="$1"
    local thread_name
    thread_name=$(basename "$manifest" .toml)
    python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib
with open('$manifest', 'rb') as f:
    data = tomllib.load(f)
for s in data.get('sources', []):
    sid = s.get('id', '')
    db = s.get('database', '').lower()
    fmt = s.get('format', '')
    accs = s.get('accessions', [])
    for acc in accs:
        print(f'{sid}\t{acc}\t{db}\t{fmt}')
    if not accs:
        print(f'{sid}\t\t{db}\t{fmt}')
" 2>/dev/null | while IFS=$'\t' read -r sid acc db fmt; do
        [[ -z "$sid" ]] && continue
        [[ -z "$acc" || ! -d "$DATA_DIR" ]] && continue
        local expected
        expected=$(resolve_data_path "$db" "$acc" "$fmt")
        if [[ -n "$expected" && -f "$expected" ]]; then
            register_data_file "$expected" "foundation:${thread_name}:${sid}:$(basename "$expected")"
        fi
    done
}

if [[ -d "$DATA_DIR" ]]; then
    if [[ "$THREAD_FILTER" == "all" ]]; then
        for manifest in "$SOURCES_MANIFEST_DIR"/*.toml; do
            [[ -f "$manifest" ]] || continue
            register_from_manifest "$manifest"
        done
    else
        while IFS= read -r manifest; do
            [[ -f "$manifest" ]] || continue
            register_from_manifest "$manifest"
        done < <(resolve_thread_manifests "$THREAD_FILTER")
    fi
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
TOTAL_SKIP=0

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
        # Prefer toadstool.validate (hardware-aware scheduling + validation)
        # with fallback to toadstool execute (run only, no validation)
        if "$TOADSTOOL" validate --timeout 300 --format text "$toml_path" > "$output_file" 2>&1; then
            log "  [$name] dispatched via toadstool.validate"
        elif "$TOADSTOOL" execute --timeout 300 --format text "$toml_path" > "$output_file" 2>&1; then
            log "  [$name] dispatched via toadstool execute (validate unavailable)"
        fi
    else
        log "  [$name] toadStool not found at $TOADSTOOL — running command directly"
        local cmd_line
        cmd_line=$(python3 -c "
import sys, json
try:
    import tomllib
except ImportError:
    import tomli as tomllib
with open('$toml_path', 'rb') as f:
    d = tomllib.load(f)
exe = d.get('execution', {})
cmd = exe.get('command', '')
args = exe.get('args', [])
# Output as JSON array for safe parsing
print(json.dumps([cmd] + args))
" 2>/dev/null) || cmd_line="[]"
        local cmd
        cmd=$(echo "$cmd_line" | python3 -c "import sys,json; a=json.load(sys.stdin); print(a[0] if a else '')")
        if [[ -n "$cmd" && -x "$cmd" ]]; then
            # Build command array safely from JSON
            readarray -t cmd_array < <(echo "$cmd_line" | python3 -c "
import sys,json
for x in json.load(sys.stdin):
    print(x)
")
            "${cmd_array[@]}" > "$output_file" 2>&1 || true
        else
            echo "[SKIP] No executable found" > "$output_file"
        fi
    fi

    local end_time
    end_time=$(date +%s)
    local elapsed=$((end_time - start_time))

    local ok_count fail_count skip_count
    read -r ok_count fail_count skip_count < <(parse_workload_results "$output_file")
    TOTAL_OK=$((TOTAL_OK + ok_count))
    TOTAL_FAIL=$((TOTAL_FAIL + fail_count))
    TOTAL_SKIP=$((TOTAL_SKIP + skip_count))

    local output_hash
    output_hash=$(blake3_hash "$output_file")

    rpc_nestgate "{\"jsonrpc\":\"2.0\",\"method\":\"storage.store\",\"params\":{\"key\":\"foundation:workload:$name\",\"value\":\"blake3:$output_hash\"},\"id\":$((EVENT_IDX+500))}" > /dev/null 2>&1 || true

    rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.event.append\",\"params\":{\"session_id\":\"$SESSION_ID\",\"event_type\":{\"ExperimentEnd\":{}},\"data\":{\"workload\":\"$name\",\"ok\":$ok_count,\"fail\":$fail_count,\"elapsed_s\":$elapsed,\"output_hash\":\"$output_hash\"}},\"id\":$((EVENT_IDX+600))}" > /dev/null 2>&1 || true
    EVENT_IDX=$((EVENT_IDX + 1))

    local status="RUN"
    [[ $skip_count -gt 0 && $ok_count -eq 0 && $fail_count -eq 0 ]] && status="SKIP"
    [[ $fail_count -gt 0 ]] && status="FAIL"
    [[ $ok_count -gt 0 && $fail_count -eq 0 && $skip_count -eq 0 ]] && status="PASS"

    WORKLOAD_TABLE+="| $name | $ok_count | $fail_count | $skip_count | ${elapsed}s | $status |\n"
    log "  [$name] $ok_count OK / $fail_count FAIL / $skip_count SKIP (${elapsed}s)"
}

if [[ "$THREAD_FILTER" == "all" ]]; then
    SCAN_DIRS=()
    for wdir in "$WORKLOAD_DIR"/*/; do
        [[ -d "$wdir" ]] && SCAN_DIRS+=("$wdir")
    done
else
    local_prefix=$(resolve_thread_dir "$THREAD_FILTER")
    if [[ -n "$local_prefix" && -d "$WORKLOAD_DIR/${local_prefix}_${THREAD_FILTER}" ]]; then
        SCAN_DIRS=("$WORKLOAD_DIR/${local_prefix}_${THREAD_FILTER}")
    else
        log "  [WARN] No workload directory found for thread '$THREAD_FILTER'"
        SCAN_DIRS=()
    fi
fi

for dir in "${SCAN_DIRS[@]}"; do
    [[ -d "$dir" ]] || continue
    for toml in "$dir"/*.toml; do
        [[ -f "$toml" ]] || continue
        execute_workload "$toml"
    done
done

# ══════════════════════════════════════════════════════════════
# PHASE 6: Compare results against validation targets
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 6: Compare Against Targets ──"

TARGET_DIR="$FOUNDATION_ROOT/data/targets"
TARGET_HITS=0
TARGET_MISS=0

# Source target comparison logic
# shellcheck source=lib/target_compare.sh
source "$SCRIPT_DIR/lib/target_compare.sh"

if [[ "$THREAD_FILTER" == "all" ]]; then
    while IFS= read -r short; do
        compare_targets "$short"
    done < <(list_thread_shorts)
else
    compare_targets "$THREAD_FILTER"
fi

log "  Targets: $TARGET_HITS hit, $TARGET_MISS miss"

# ══════════════════════════════════════════════════════════════
# PHASE 7: Commit provenance
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 7: Commit Provenance ──"

PROVENANCE_WARN=0

COMPLETE_RESP=$(rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.session.complete\",\"params\":{\"session_id\":\"$SESSION_ID\"},\"id\":800}")
MERKLE_ROOT=$(rpc_extract_field "$COMPLETE_RESP" "merkle_root")
[[ -z "$MERKLE_ROOT" ]] && MERKLE_ROOT=$(rpc_extract_field "$COMPLETE_RESP" "root")
if [[ -n "$MERKLE_ROOT" && "$MERKLE_ROOT" != "unknown" ]]; then
    log "  [OK] DAG Merkle root: $MERKLE_ROOT"
else
    if rpc_has_error "$COMPLETE_RESP"; then
        log "  [WARN] DAG session.complete error: $(rpc_error_message "$COMPLETE_RESP")"
    else
        log "  [WARN] DAG session.complete returned no Merkle root"
    fi
    MERKLE_ROOT="unknown"
    PROVENANCE_WARN=$((PROVENANCE_WARN + 1))
fi

MERKLE_BYTES=$(hash_to_byte_array "${MERKLE_ROOT:-0000000000000000000000000000000000000000000000000000000000000000}")
COMMIT_RESP=$(rpc_loamspine "{\"jsonrpc\":\"2.0\",\"method\":\"entry.append\",\"params\":{\"spine_id\":\"$SPINE_ID\",\"entry_type\":{\"SessionCommit\":{\"session_hash\":$MERKLE_BYTES}},\"committer\":\"did:primal:foundation\",\"data\":{\"session\":\"$SESSION_NAME\",\"merkle_root\":\"$MERKLE_ROOT\",\"events\":$EVENT_IDX,\"ok\":$TOTAL_OK,\"fail\":$TOTAL_FAIL}},\"id\":801}")
if rpc_has_result "$COMMIT_RESP"; then
    log "  [OK] loamSpine committed"
else
    if rpc_has_error "$COMMIT_RESP"; then
        log "  [WARN] loamSpine commit error: $(rpc_error_message "$COMMIT_RESP")"
    else
        log "  [WARN] loamSpine commit returned no result"
    fi
    PROVENANCE_WARN=$((PROVENANCE_WARN + 1))
fi

BRAID_RESP=$(rpc_sweetgrass "{\"jsonrpc\":\"2.0\",\"method\":\"braid.create\",\"params\":{\"creator\":\"did:primal:foundation\",\"subject\":\"foundation-validation:$SESSION_NAME\",\"claims\":[{\"type\":\"ProvenanceValidation\",\"data\":{\"session\":\"$SESSION_NAME\",\"merkle_root\":\"$MERKLE_ROOT\",\"ok\":$TOTAL_OK,\"fail\":$TOTAL_FAIL,\"events\":$EVENT_IDX}}]},\"id\":802}")
BRAID_URN=$(rpc_extract_field "$BRAID_RESP" "urn")
[[ -z "$BRAID_URN" ]] && BRAID_URN=$(rpc_extract_field "$BRAID_RESP" "id")
if [[ -n "$BRAID_URN" && "$BRAID_URN" != "unknown" ]]; then
    log "  [OK] sweetGrass braid: $BRAID_URN"
else
    if rpc_has_error "$BRAID_RESP"; then
        log "  [WARN] sweetGrass braid error: $(rpc_error_message "$BRAID_RESP")"
    else
        log "  [WARN] sweetGrass braid creation returned no URN"
    fi
    BRAID_URN="unknown"
    PROVENANCE_WARN=$((PROVENANCE_WARN + 1))
fi

if [[ $PROVENANCE_WARN -gt 0 ]]; then
    log "  [WARN] $PROVENANCE_WARN provenance step(s) incomplete — chain is partial"
fi

echo "$BRAID_RESP" > "$RESULTS_DIR/braid.json"

# ══════════════════════════════════════════════════════════════
# PHASE 8: Write validation report
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 8: Write Report ──"

# shellcheck source=lib/report_writer.sh
source "$SCRIPT_DIR/lib/report_writer.sh"

write_validation_report
write_provenance_toml
write_results_json
distribute_to_spring_folders

log ""
log "═══════════════════════════════════════════════════════════"
log "  Foundation Validation Complete"
log "  $TOTAL_OK OK / $TOTAL_FAIL FAIL / $TOTAL_SKIP SKIP across $EVENT_IDX provenance events"
log "  Merkle root: $MERKLE_ROOT"
log "  Braid: $BRAID_URN"
log "  Results: $RESULTS_DIR"
log "═══════════════════════════════════════════════════════════"

[[ $TOTAL_FAIL -gt 0 ]] && exit 1
exit 0
