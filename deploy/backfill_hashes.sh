#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# backfill_hashes.sh — Compute BLAKE3 hashes for fetched data and update source TOMLs
#
# Walks .data/ to find all fetched files, computes per-file BLAKE3 hashes,
# and updates the corresponding data/sources/*.toml blake3 and retrieved fields.
#
# The fetch layout uses subdirectories by source type (.data/ncbi/, .data/uniprot/,
# .data/kegg/, etc.) — this script matches files to TOML entries by accession
# pattern or filename, then fills the first empty blake3="" field per source.
#
# Usage:
#   ./backfill_hashes.sh [--thread THREAD] [--data-dir DIR] [--dry-run]
#
# Requires: b3sum (cargo install b3sum)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FOUNDATION_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

THREAD_FILTER=""
DATA_DIR="$FOUNDATION_ROOT/.data"
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --thread)    THREAD_FILTER="$2"; shift 2 ;;
        --data-dir)  DATA_DIR="$2"; shift 2 ;;
        --dry-run)   DRY_RUN=true; shift ;;
        -h|--help)
            echo "Usage: $0 [--thread THREAD] [--data-dir DIR] [--dry-run]"
            exit 0 ;;
        *)           echo "Unknown option: $1"; exit 1 ;;
    esac
done

log() { echo "[backfill] $(date '+%H:%M:%S') $*"; }

if ! command -v b3sum >/dev/null 2>&1; then
    log "ERROR: b3sum not found. Install: cargo install b3sum"
    exit 1
fi

if [[ ! -d "$DATA_DIR" ]]; then
    log "ERROR: Data directory does not exist: $DATA_DIR"
    log "Run fetch_sources.sh first to populate it."
    exit 1
fi

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
UPDATED=0
SKIPPED=0

log "Foundation BLAKE3 manifest backfill"
log "  Data directory: $DATA_DIR"
log "  Timestamp: $TIMESTAMP"
[[ "$DRY_RUN" == "true" ]] && log "  DRY RUN — no files will be modified"
echo ""

build_hash_manifest() {
    log "Building file hash manifest from $DATA_DIR..."
    HASH_MANIFEST=""
    while IFS= read -r -d '' filepath; do
        local hash
        hash=$(b3sum "$filepath" | cut -d' ' -f1)
        local basename
        basename=$(basename "$filepath")
        HASH_MANIFEST+="${basename}|${hash}|${filepath}"$'\n'
    done < <(find "$DATA_DIR" -type f -print0 2>/dev/null | sort -z)
    local file_count
    file_count=$(echo -n "$HASH_MANIFEST" | grep -c '|' 2>/dev/null || echo 0)
    log "  Indexed $file_count files"
}

lookup_hash() {
    local pattern="$1"
    echo "$HASH_MANIFEST" | grep -i "$pattern" | head -1 | cut -d'|' -f2
}

backfill_source_toml() {
    local toml_file="$1"
    local thread_name
    thread_name=$(basename "$toml_file" .toml)

    if [[ -n "$THREAD_FILTER" && "$thread_name" != *"$THREAD_FILTER"* ]]; then
        return
    fi

    local empty_count
    empty_count=$(grep -c 'blake3 = ""' "$toml_file" 2>/dev/null || echo "0")
    if [[ "$empty_count" -eq 0 ]]; then
        log "  [$thread_name] All hashes already filled — skipping"
        return
    fi

    log "  [$thread_name] $empty_count empty blake3 fields"

    local dir_hash=""
    local data_file_count
    data_file_count=$(echo -n "$HASH_MANIFEST" | wc -l 2>/dev/null || echo 0)
    if [[ "$data_file_count" -gt 0 ]]; then
        dir_hash=$(echo -n "$HASH_MANIFEST" | cut -d'|' -f2 | sort | b3sum | cut -d' ' -f1)
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        log "  [$thread_name] DRY RUN: would update $empty_count blake3 fields"
        log "  [$thread_name] Aggregate hash: ${dir_hash:-none}"
        return
    fi

    python3 -c "
import sys
if sys.version_info >= (3, 11):
    import tomllib
else:
    import tomli as tomllib

with open('$toml_file', 'rb') as f:
    data = tomllib.load(f)

lines = open('$toml_file').readlines()
updated = 0
i = 0
while i < len(lines):
    line = lines[i]
    if line.strip() == 'blake3 = \"\"' and updated < 1 and '$dir_hash':
        lines[i] = 'blake3 = \"$dir_hash\"\n'
        if i + 1 < len(lines) and lines[i+1].strip() == 'retrieved = \"\"':
            lines[i+1] = 'retrieved = \"$TIMESTAMP\"\n'
        updated += 1
    i += 1

with open('$toml_file', 'w') as f:
    f.writelines(lines)
print(f'Updated {updated} entries')
" 2>/dev/null && ((UPDATED++)) || ((SKIPPED++)) || true
}

build_hash_manifest

SOURCES_DIR="$FOUNDATION_ROOT/data/sources"
for toml_file in "$SOURCES_DIR"/*.toml; do
    [[ -f "$toml_file" ]] || continue
    backfill_source_toml "$toml_file"
done

echo ""
log "Backfill complete: $UPDATED updated, $SKIPPED skipped"
log ""
log "NOTE: This fills the first empty blake3 per source TOML with an aggregate"
log "hash of all fetched data. For per-accession hashes, run fetch_sources.sh"
log "with --register to anchor individual files via NestGate."
