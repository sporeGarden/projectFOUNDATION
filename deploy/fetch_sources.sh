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
            exit 0 ;;
        *)            echo "Unknown option: $1"; exit 1 ;;
    esac
done

log() { echo "[$(date +%H:%M:%S)] $*"; }

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

rpc_nestgate() {
    printf '%s\n' "$1" | nc -w 5 "${PRIMAL_HOST:-127.0.0.1}" "$NESTGATE_PORT" 2>/dev/null
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
# Thread 5: LTEE (Evolutionary Biology)
# ==========================================================================

fetch_thread05_ltee() {
    log ""
    log "── Thread 5: Evolutionary Biology / LTEE ──"

    # Genomes — aligned with data/sources/thread05_ltee.toml accessions
    fetch_ncbi_nucleotide "CP000819.1"      # REL606 ancestor (ncbi_barrick_2009_genome)
    fetch_ncbi_nucleotide "U00096.3"        # E. coli K-12 MG1655 outgroup (ncbi_k12_mg1655)

    # BioProject metadata — aligned with manifest
    fetch_ncbi_bioproject "PRJNA294072"     # Tenaillon 2016 264 genomes (ncbi_tenaillon_2016_genomes)
    fetch_ncbi_bioproject "PRJNA188989"     # Wiser 2013 population data (ncbi_wiser_2013_popdata)

    # SRA metadata (manifest uses SRP accessions — fetch BioProject metadata as proxy)
    fetch_ncbi_bioproject "PRJNA295606"     # Tenaillon 2016 VCF (ncbi_tenaillon_2016_vcf)

    # Dryad DOIs are not auto-fetchable via simple API — manifest documents them
    log "  [INFO] Dryad sources (Barrick 2009, Wiser 2013, Good 2017) documented in manifest"
    log "  [INFO] SRA accessions (SRP001064, SRP073287, SRP064605) need SRA toolkit for bulk download"
}

# ==========================================================================
# Thread 6: Agricultural Science
# ==========================================================================

fetch_thread06_ag() {
    log ""
    log "── Thread 6: Agricultural Science ──"

    # Soil microbiome sequencing projects
    fetch_ncbi_bioproject "PRJNA524590"     # No-till soil 16S
    fetch_ncbi_bioproject "PRJNA480600"     # Rhizosphere microbiome

    # NOAA weather data is fetched via separate tooling (large datasets).
    # IRIS/KEGG are web-service queries, not bulk downloads.
    log "  [INFO] FAO-56, NOAA, ERA5, IRIS sources are web-service or literature — fetch via workload"
}

# ==========================================================================
# Thread 5-ML: ML Surrogates & Evolutionary Dynamics
# ==========================================================================

fetch_thread05_ml() {
    log ""
    log "── Thread 5-ML: ML Surrogates & Evolutionary Dynamics ──"
    log "  [INFO] Thread 5-ML sources are literature references, public APIs (Open-Meteo),"
    log "  [INFO] and standard ML datasets (MNIST via torchvision)."
    log "  [INFO] Paper DOIs and dataset URLs are catalogued in thread05_ml_surrogates.toml."
    log "  [INFO] No bulk NCBI/UniProt data to fetch. ERA5 and MNIST fetched at train time."
    SKIP_COUNT=$((SKIP_COUNT + 1))
}

# ==========================================================================
# Thread 7: Anderson Mathematics
# ==========================================================================

fetch_thread07_anderson() {
    log ""
    log "── Thread 7: Anderson Mathematics ──"
    log "  [INFO] Thread 7 sources are literature references and internal models."
    log "  [INFO] No public data to fetch. Validation data is generated by springs."
    SKIP_COUNT=$((SKIP_COUNT + 1))
}

# ==========================================================================
# Thread 9: Gaming / Creative
# ==========================================================================

fetch_thread09_gaming() {
    log ""
    log "── Thread 9: Gaming / Creative ──"
    log "  [INFO] Thread 9 sources are literature references and procedural generation benchmarks."
    log "  [INFO] No public data to fetch. Validation data is generated by ludoSpring."
    SKIP_COUNT=$((SKIP_COUNT + 1))
}

# ==========================================================================
# Thread 10: Provenance / Economics
# ==========================================================================

fetch_thread10_provenance() {
    log ""
    log "── Thread 10: Provenance / Economics ──"
    log "  [INFO] Thread 10 sources are internal test vectors (rhizoCrypt, loamSpine, sweetGrass)."
    log "  [INFO] No external public data to fetch. Validation via NUCLEUS composition."
    SKIP_COUNT=$((SKIP_COUNT + 1))
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
    print(json.dumps(e))
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
    while IFS= read -r line; do
        local db acc fmt sid
        db=$(echo "$line" | python3 -c "import sys,json; print(json.load(sys.stdin)['database'])")
        acc=$(echo "$line" | python3 -c "import sys,json; print(json.load(sys.stdin)['accession'])")
        fmt=$(echo "$line" | python3 -c "import sys,json; print(json.load(sys.stdin)['format'])")
        sid=$(echo "$line" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")
        dispatch_manifest_entry "$db" "$acc" "$fmt" "$sid"
    done <<< "$entries"
}

# ==========================================================================
# Dispatch — manifest-driven with legacy fallback
# ==========================================================================

SOURCES_DIR="$FOUNDATION_ROOT/data/sources"

THREAD_MAP=(
    "wcm|thread01_wcm"
    "plasma|thread02_plasma"
    "immuno|thread03_immuno"
    "enviro|thread04_enviro"
    "ltee|thread05_ltee"
    "ml|thread05_ml_surrogates"
    "ag|thread06_ag"
    "anderson|thread07_anderson"
    "health|thread08_health"
    "gaming|thread09_gaming"
    "provenance|thread10_provenance"
)

resolve_thread_toml() {
    local filter="$1"
    for entry in "${THREAD_MAP[@]}"; do
        IFS='|' read -r short toml_stem <<< "$entry"
        if [[ "$filter" == "$short" || "$filter" == "$toml_stem" ]]; then
            echo "$SOURCES_DIR/${toml_stem}.toml"
            return 0
        fi
    done
    local glob_match
    glob_match=$(ls "$SOURCES_DIR"/thread*"${filter}"*.toml 2>/dev/null | head -1)
    [[ -n "$glob_match" ]] && echo "$glob_match" && return 0
    return 1
}

if [[ -n "$THREAD_FILTER" ]]; then
    if [[ "$THREAD_FILTER" == "all" ]]; then
        for toml_file in "$SOURCES_DIR"/*.toml; do
            [[ -f "$toml_file" ]] || continue
            run_manifest_driven "$toml_file"
        done
    else
        toml_file=$(resolve_thread_toml "$THREAD_FILTER") || {
            log "Unknown thread: $THREAD_FILTER"
            exit 1
        }
        run_manifest_driven "$toml_file"
    fi
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
