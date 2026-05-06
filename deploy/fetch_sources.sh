#!/usr/bin/env bash
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
            exit 0 ;;
        *)            echo "Unknown option: $1"; exit 1 ;;
    esac
done

log() { echo "[$(date +%H:%M:%S)] $*"; }

blake3_hash() { b3sum "$1" | cut -d' ' -f1; }

rpc_nestgate() {
    printf '%s\n' "$1" | nc -w 5 127.0.0.1 "$NESTGATE_PORT" 2>/dev/null
}

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

    if curl -sf --max-time 120 -o "$out_file" "$url"; then
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

    if curl -sf --max-time 60 -o "$out_file" "$url"; then
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

    if curl -sf --max-time 120 -o "$out_file" "$url"; then
        local hash
        hash=$(blake3_hash "$out_file")
        local size
        size=$(stat -c%s "$out_file" 2>/dev/null || stat -f%z "$out_file" 2>/dev/null)
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

    if curl -sf --max-time 60 -o "$out_file" "$url"; then
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
    if curl -sf --max-time 60 -o "$out_file" "https://rest.kegg.jp/list/pathway/$org_code"; then
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
# Thread 1: Whole-Cell Modeling
# ==========================================================================

fetch_thread01_wcm() {
    log ""
    log "── Thread 1: Whole-Cell Modeling ──"

    # Genomes
    fetch_ncbi_nucleotide "NC_000908.2"     # M. genitalium
    fetch_ncbi_nucleotide "CP016816.2"      # JCVI-syn3A
    fetch_ncbi_nucleotide "NC_000913.3"     # E. coli K-12

    # Assemblies
    fetch_ncbi_assembly "GCA_000027325.1"   # M. genitalium assembly
    fetch_ncbi_assembly "GCA_900015295.1"   # JCVI-syn3.0 assembly

    # Proteomes
    fetch_uniprot_proteome "UP000000807"    # M. genitalium
    fetch_uniprot_proteome "UP000000625"    # E. coli K-12
    fetch_uniprot_proteome "UP000018174"    # M. mycoides (closest to syn3A)

    # KEGG metabolic networks
    fetch_kegg_org "mge"                    # M. genitalium
    fetch_kegg_org "eco"                    # E. coli
    fetch_kegg_org "mmc"                    # M. mycoides

    # BioProject metadata
    fetch_ncbi_bioproject "PRJNA357500"     # JCVI synthetic biology
}

# ==========================================================================
# Thread 4: Environmental Genomics
# ==========================================================================

fetch_thread04_enviro() {
    log ""
    log "── Thread 4: Environmental Genomics ──"

    # BioProject metadata
    fetch_ncbi_bioproject "PRJNA488170"     # Saginaw Bay HAB (already validated!)
    fetch_ncbi_bioproject "PRJNA285472"     # Lake Erie cyanobacteria
    fetch_ncbi_bioproject "PRJNA636789"     # PFAS microbiome
    fetch_ncbi_bioproject "PRJNA503411"     # Deep-sea cold seep
    fetch_ncbi_bioproject "PRJNA473816"     # Coral holobiont
    fetch_ncbi_bioproject "PRJNA524590"     # No-till soil
    fetch_ncbi_bioproject "PRJNA546013"     # MinION reference
    fetch_ncbi_bioproject "PRJNA547561"     # Anaerobic digester
    fetch_ncbi_bioproject "PRJNA517152"     # Gut anaerobic
    fetch_ncbi_bioproject "PRJNA480600"     # Rhizosphere
}

# ==========================================================================
# Thread 3: Immunology & Drug Discovery
# ==========================================================================

fetch_thread03_immuno() {
    log ""
    log "── Thread 3: Immunology & Drug Discovery ──"

    # GEO datasets (metadata)
    fetch_ncbi_bioproject "PRJNA175577"     # GSE32924 approximate BioProject
    fetch_ncbi_bioproject "PRJNA187999"     # GSE36842 approximate BioProject
    fetch_ncbi_bioproject "PRJNA422434"     # Gut metabolome
    fetch_ncbi_bioproject "PRJNA388210"     # FMT clinical
    fetch_ncbi_bioproject "PRJNA355023"     # Antibiotic perturbation
}

# ==========================================================================
# Thread 8: Human Health
# ==========================================================================

fetch_thread08_health() {
    log ""
    log "── Thread 8: Human Health ──"

    # Canine genome
    fetch_ncbi_assembly "GCA_011100685.1"   # CanFam4

    # Health BioProjects (metadata)
    fetch_ncbi_bioproject "PRJNA388210"     # FMT (shared with thread 3)
    fetch_ncbi_bioproject "PRJNA355023"     # Antibiotic perturbation (shared)
}

# ==========================================================================
# Dispatch
# ==========================================================================

if [[ -n "$THREAD_FILTER" ]]; then
    case "$THREAD_FILTER" in
        wcm|thread01)    fetch_thread01_wcm ;;
        plasma|thread02) log "Thread 2 (Plasma) sources are literature-only — no automated fetch." ;;
        immuno|thread03) fetch_thread03_immuno ;;
        enviro|thread04) fetch_thread04_enviro ;;
        health|thread08) fetch_thread08_health ;;
        all)
            fetch_thread01_wcm
            fetch_thread03_immuno
            fetch_thread04_enviro
            fetch_thread08_health
            ;;
        *) log "Unknown thread: $THREAD_FILTER"; exit 1 ;;
    esac
else
    fetch_thread01_wcm
    fetch_thread03_immuno
    fetch_thread04_enviro
    fetch_thread08_health
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
