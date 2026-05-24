#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# fetch_sources.sh — Retrieve public data sources and anchor them with BLAKE3
#
# Reads data/sources/*.toml manifests, fetches datasets from NCBI, UniProt,
# KEGG, and other public repositories, computes BLAKE3 hashes, and optionally
# registers artifacts with NestGate.
#
# Usage:
#   ./fetch_sources.sh [--thread THREAD_SHORT] [--data-dir DIR] [--register]
#
# Examples:
#   ./fetch_sources.sh --thread wcm              # Fetch Thread 1 sources only
#   ./fetch_sources.sh --register                 # Fetch all + register with NestGate
#   ./fetch_sources.sh --thread enviro --data-dir /data/foundation
#
# Requires: curl, b3sum
# Optional: NestGate running (for --register)
#
# NCBI E-utilities API: set NCBI_API_KEY for higher rate limits (10/sec vs 3/sec).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FOUNDATION_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

THREAD_FILTER=""
DATA_DIR="$FOUNDATION_ROOT/.data"
REGISTER=false
NESTGATE_PORT="${NESTGATE_PORT:-9500}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --thread)     THREAD_FILTER="$2"; shift 2 ;;
        --data-dir)   DATA_DIR="$2"; shift 2 ;;
        --register)   REGISTER=true; shift ;;
        -h|--help)
            echo "Usage: $0 [--thread THREAD_SHORT] [--data-dir DIR] [--register]"
            if [[ -f "$SCRIPT_DIR/lib/thread_registry.sh" ]]; then
                source "$SCRIPT_DIR/lib/thread_registry.sh"
                thread_help_text
            fi
            exit 0 ;;
        *)            echo "Unknown option: $1"; exit 1 ;;
    esac
done

log() { echo "[$(date +%H:%M:%S)] $*"; }

# Source shared IPC helpers for blake3_hash and discovery
if [[ -f "$SCRIPT_DIR/lib/primal_ipc.sh" ]]; then
    # shellcheck source=lib/primal_ipc.sh
    source "$SCRIPT_DIR/lib/primal_ipc.sh"
fi

# blake3_hash with Python fallback (extends primal_ipc.sh's b3sum-only version)
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

fetch_with_retry() {
    local url="$1" out="$2" max_retries="${3:-3}" timeout="${4:-120}"
    local attempt=0
    while [[ $attempt -lt $max_retries ]]; do
        attempt=$((attempt + 1))
        if curl -sf --max-time "$timeout" -o "$out" "$url"; then
            return 0
        fi
        [[ $attempt -lt $max_retries ]] && log "    Retry $attempt/$max_retries..." && sleep "$((attempt * 2))"
    done
    return 1
}

# Use primal_ipc.sh rpc_nestgate when sourced; local fallback for standalone use
if ! declare -f rpc_nestgate >/dev/null 2>&1; then
    rpc_nestgate() {
        printf '%s\n' "$1" | nc -w 5 "${PRIMAL_HOST:-127.0.0.1}" "${NESTGATE_PORT:-9500}" 2>/dev/null
    }
fi

NCBI_BASE="https://eutils.ncbi.nlm.nih.gov/entrez/eutils"
NCBI_PARAMS=""
[[ -n "${NCBI_API_KEY:-}" ]] && NCBI_PARAMS="&api_key=$NCBI_API_KEY"

FETCH_COUNT=0
SKIP_COUNT=0
FAIL_COUNT=0

log "═══════════════════════════════════════════════════════════"
log "  Foundation Data Source Fetcher"
log "  Data directory: $DATA_DIR"
[[ -n "$THREAD_FILTER" ]] && log "  Thread filter: $THREAD_FILTER"
$REGISTER && log "  NestGate registration: ENABLED"
log "═══════════════════════════════════════════════════════════"

# --------------------------------------------------------------------------
# NCBI Nucleotide/Assembly fetchers
# --------------------------------------------------------------------------

fetch_ncbi_nucleotide() {
    local accession="$1"
    local out_dir="$DATA_DIR/ncbi/nucleotide"
    local out_file="$out_dir/${accession}.gb"

    mkdir -p "$out_dir"

    if [[ -f "$out_file" ]]; then
        log "  [EXIST] ncbi:$accession — already fetched"
        SKIP_COUNT=$((SKIP_COUNT + 1))
        return 0
    fi

    log "  [FETCH] ncbi:$accession ..."
    local url="${NCBI_BASE}/efetch.fcgi?db=nucleotide&id=${accession}&rettype=gb&retmode=text${NCBI_PARAMS}"

    if fetch_with_retry "$url" "$out_file" 3 120; then
        local hash
        hash=$(blake3_hash "$out_file")
        local size
        size=$(stat -c%s "$out_file" 2>/dev/null || stat -f%z "$out_file" 2>/dev/null)
        log "  [OK]    ncbi:$accession → blake3:${hash:0:16}… (${size}B)"

        if $REGISTER; then
            rpc_nestgate "{\"jsonrpc\":\"2.0\",\"method\":\"storage.store\",\"params\":{\"key\":\"ncbi:nucleotide:$accession\",\"value\":\"blake3:$hash size:$size\"},\"id\":$FETCH_COUNT}" > /dev/null 2>&1 || true
        fi

        FETCH_COUNT=$((FETCH_COUNT + 1))
    else
        log "  [FAIL]  ncbi:$accession — download failed"
        rm -f "$out_file"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi

    sleep "${NCBI_DELAY:-0.4}"
}

fetch_ncbi_assembly() {
    local accession="$1"
    local out_dir="$DATA_DIR/ncbi/assembly"
    local out_file="$out_dir/${accession}_report.txt"

    mkdir -p "$out_dir"

    if [[ -f "$out_file" ]]; then
        log "  [EXIST] assembly:$accession — already fetched"
        SKIP_COUNT=$((SKIP_COUNT + 1))
        return 0
    fi

    log "  [FETCH] assembly:$accession ..."
    local url="https://api.ncbi.nlm.nih.gov/datasets/v2/genome/accession/${accession}/dataset_report${NCBI_PARAMS:+?${NCBI_PARAMS#&}}"

    if fetch_with_retry "$url" "$out_file" 3 60; then
        local hash
        hash=$(blake3_hash "$out_file")
        log "  [OK]    assembly:$accession → blake3:${hash:0:16}…"
        FETCH_COUNT=$((FETCH_COUNT + 1))
    else
        log "  [FAIL]  assembly:$accession — download failed"
        rm -f "$out_file"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi

    sleep "${NCBI_DELAY:-0.4}"
}

# --------------------------------------------------------------------------
# UniProt proteome fetcher
# --------------------------------------------------------------------------

fetch_uniprot_proteome() {
    local accession="$1"
    local out_dir="$DATA_DIR/uniprot"
    local out_file="$out_dir/${accession}.fasta.gz"

    mkdir -p "$out_dir"

    if [[ -f "$out_file" ]]; then
        log "  [EXIST] uniprot:$accession — already fetched"
        SKIP_COUNT=$((SKIP_COUNT + 1))
        return 0
    fi

    log "  [FETCH] uniprot:$accession ..."
    local url="https://rest.uniprot.org/uniprotkb/stream?compressed=true&format=fasta&query=(proteome:${accession})"

    if fetch_with_retry "$url" "$out_file" 3 120; then
        local size
        size=$(stat -c%s "$out_file" 2>/dev/null || stat -f%z "$out_file" 2>/dev/null)

        if [[ "$size" -lt 100 ]]; then
            log "  [FAIL]  uniprot:$accession — file too small (${size}B, likely empty/excluded proteome)"
            rm -f "$out_file"
            FAIL_COUNT=$((FAIL_COUNT + 1))
            return 0
        fi

        if ! gzip -t "$out_file" 2>/dev/null; then
            log "  [FAIL]  uniprot:$accession — gzip integrity check failed (truncated/corrupt)"
            rm -f "$out_file"
            FAIL_COUNT=$((FAIL_COUNT + 1))
            return 0
        fi

        local hash
        hash=$(blake3_hash "$out_file")
        log "  [OK]    uniprot:$accession → blake3:${hash:0:16}… (${size}B)"

        if $REGISTER; then
            rpc_nestgate "{\"jsonrpc\":\"2.0\",\"method\":\"storage.store\",\"params\":{\"key\":\"uniprot:proteome:$accession\",\"value\":\"blake3:$hash size:$size\"},\"id\":$FETCH_COUNT}" > /dev/null 2>&1 || true
        fi

        FETCH_COUNT=$((FETCH_COUNT + 1))
    else
        log "  [FAIL]  uniprot:$accession — download failed"
        rm -f "$out_file"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
}

# --------------------------------------------------------------------------
# NCBI BioProject/SRA metadata fetcher
# --------------------------------------------------------------------------

fetch_ncbi_bioproject() {
    local accession="$1"
    local out_dir="$DATA_DIR/ncbi/bioproject"
    local out_file="$out_dir/${accession}.xml"

    mkdir -p "$out_dir"

    if [[ -f "$out_file" ]]; then
        log "  [EXIST] bioproject:$accession — already fetched"
        SKIP_COUNT=$((SKIP_COUNT + 1))
        return 0
    fi

    log "  [FETCH] bioproject:$accession ..."
    local url="${NCBI_BASE}/efetch.fcgi?db=bioproject&id=${accession}&rettype=xml${NCBI_PARAMS}"

    if fetch_with_retry "$url" "$out_file" 3 60; then
        local hash
        hash=$(blake3_hash "$out_file")
        log "  [OK]    bioproject:$accession → blake3:${hash:0:16}…"
        FETCH_COUNT=$((FETCH_COUNT + 1))
    else
        log "  [FAIL]  bioproject:$accession — download failed"
        rm -f "$out_file"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi

    sleep "${NCBI_DELAY:-0.4}"
}

# --------------------------------------------------------------------------
# KEGG pathway fetcher
# --------------------------------------------------------------------------

fetch_kegg_org() {
    local org_code="$1"
    local out_dir="$DATA_DIR/kegg"
    local out_file="$out_dir/${org_code}_pathways.json"

    mkdir -p "$out_dir"

    if [[ -f "$out_file" ]]; then
        log "  [EXIST] kegg:$org_code — already fetched"
        SKIP_COUNT=$((SKIP_COUNT + 1))
        return 0
    fi

    log "  [FETCH] kegg:$org_code ..."
    if fetch_with_retry "https://rest.kegg.jp/list/pathway/$org_code" "$out_file" 3 60; then
        local hash
        hash=$(blake3_hash "$out_file")
        log "  [OK]    kegg:$org_code → blake3:${hash:0:16}…"
        FETCH_COUNT=$((FETCH_COUNT + 1))
    else
        log "  [FAIL]  kegg:$org_code — download failed"
        rm -f "$out_file"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
}

# ==========================================================================
# Manifest-driven fetch — reads data/sources/*.toml, dispatches by database
# ==========================================================================

fetch_from_manifest() {
    local toml_file="$1"
    local thread_name
    thread_name=$(basename "$toml_file" .toml)
    log ""
    log "── Manifest: $thread_name ──"

    python3 << MANIFEST_EOF
import sys, json
try:
    import tomllib
except ImportError:
    import tomli as tomllib

with open("$toml_file", "rb") as f:
    data = tomllib.load(f)

sources = data.get("sources", [])
fetchable = []
for s in sources:
    db = s.get("database", "").lower()
    accs = s.get("accessions", [])
    sid = s.get("id", "")
    fmt = s.get("format", "")
    if not accs:
        continue
    for acc in accs:
        if not acc or acc.startswith("10.") or acc.startswith("doi:"):
            continue
        entry = {"id": sid, "database": db, "accession": acc, "format": fmt}
        fetchable.append(entry)

for e in fetchable:
    print(f"{e['database']}\t{e['accession']}\t{e['format']}\t{e['id']}")
MANIFEST_EOF
}

dispatch_manifest_entry() {
    local db="$1" accession="$2" fmt="$3" sid="$4"
    case "$db" in
        "ncbi nucleotide"|"ncbi")
            if [[ "$fmt" == "genbank" || "$fmt" == "fasta" ]]; then
                fetch_ncbi_nucleotide "$accession"
            fi ;;
        "ncbi assembly")
            fetch_ncbi_assembly "$accession" ;;
        "uniprot")
            fetch_uniprot_proteome "$accession" ;;
        "kegg")
            fetch_kegg_org "$accession" ;;
        "ncbi bioproject"|"ncbi sra")
            if [[ "$accession" == PRJNA* || "$accession" == PRJEB* ]]; then
                fetch_ncbi_bioproject "$accession"
            else
                log "  [SKIP] $sid: SRA accession $accession needs SRA toolkit"
                SKIP_COUNT=$((SKIP_COUNT + 1))
            fi ;;
        *)
            log "  [SKIP] $sid: no fetcher for database '$db'"
            SKIP_COUNT=$((SKIP_COUNT + 1)) ;;
    esac
}

run_manifest_driven() {
    local toml_file="$1"
    local entries
    entries=$(fetch_from_manifest "$toml_file" 2>/dev/null) || return
    if [[ -z "$entries" ]]; then
        log "  [INFO] No fetchable accessions in $(basename "$toml_file")"
        SKIP_COUNT=$((SKIP_COUNT + 1))
        return
    fi
    while IFS=$'\t' read -r db acc fmt sid; do
        dispatch_manifest_entry "$db" "$acc" "$fmt" "$sid"
    done <<< "$entries"
}

# ==========================================================================
# Dispatch — manifest-driven, thread resolution via THREAD_INDEX.toml
# ==========================================================================

SOURCES_DIR="$FOUNDATION_ROOT/data/sources"

# Source thread_registry.sh if available for typed resolution;
# fall back to direct TOML read if not sourced.
if [[ -f "$SCRIPT_DIR/lib/thread_registry.sh" ]]; then
    # shellcheck source=lib/thread_registry.sh
    source "$SCRIPT_DIR/lib/thread_registry.sh"
fi

resolve_thread_toml() {
    local filter="$1"
    if type resolve_thread_manifests &>/dev/null; then
        local out
        out=$(resolve_thread_manifests "$filter")
        [[ -n "$out" ]] && echo "$out" && return 0
    fi
    return 1
}

if [[ -n "$THREAD_FILTER" && "$THREAD_FILTER" != "all" ]]; then
    resolved_files=$(resolve_thread_toml "$THREAD_FILTER") || {
        log "Unknown thread: $THREAD_FILTER"
        exit 1
    }
    while IFS= read -r toml_file; do
        [[ -f "$toml_file" ]] || continue
        run_manifest_driven "$toml_file"
    done <<< "$resolved_files"
else
    for toml_file in "$SOURCES_DIR"/*.toml; do
        [[ -f "$toml_file" ]] || continue
        run_manifest_driven "$toml_file"
    done
fi

log ""
log "═══════════════════════════════════════════════════════════"
log "  Fetch complete: $FETCH_COUNT fetched, $SKIP_COUNT cached, $FAIL_COUNT failed"
log "  Data directory: $DATA_DIR"
log "═══════════════════════════════════════════════════════════"

if [[ $FAIL_COUNT -gt 0 ]]; then
    log "  Some sources failed to download. Check network, NCBI_API_KEY, or retry."
    exit 1
fi
