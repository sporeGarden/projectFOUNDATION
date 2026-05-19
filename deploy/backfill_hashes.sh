#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# backfill_hashes.sh — Compute BLAKE3 hashes for fetched data and update source TOMLs
#
# Walks .data/ to find all fetched files, computes per-file BLAKE3 hashes,
# and updates the corresponding data/sources/*.toml blake3 and retrieved fields.
#
# The fetch layout uses subdirectories by source type (.data/ncbi/, .data/uniprot/,
# .data/kegg/, etc.) — this script matches files to TOML entries by accession
# pattern, then fills empty blake3="" fields.
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

# Build a file index: accession-token → blake3 hash
# Written to a temp file to avoid shell quoting issues
MANIFEST_FILE=$(mktemp)
trap 'rm -f "$MANIFEST_FILE"' EXIT

log "Building file hash manifest from $DATA_DIR..."
FILE_COUNT=0
while IFS= read -r -d '' filepath; do
    hash=$(b3sum "$filepath" | cut -d' ' -f1)
    basename=$(basename "$filepath")
    # Strip common extensions to extract accession token
    token=$(echo "$basename" | sed 's/\.\(gb\|fasta\.gz\|fasta\|xml\|json\|txt\|csv\|tsv\|vcf\)$//' | sed 's/_report$//' | sed 's/_pathways$//')
    echo "${token}	${hash}	${filepath}" >> "$MANIFEST_FILE"
    FILE_COUNT=$((FILE_COUNT + 1))
done < <(find "$DATA_DIR" -type f -print0 2>/dev/null | sort -z)
log "  Indexed $FILE_COUNT files"

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

    if [[ "$DRY_RUN" == "true" ]]; then
        log "  [$thread_name] DRY RUN: would attempt to match $empty_count entries"
        return
    fi

    # Pass manifest via stdin to avoid shell quoting issues
    local updated
    updated=$(python3 - "$toml_file" "$TIMESTAMP" < "$MANIFEST_FILE" << 'PYEOF'
import sys, re

manifest_path = sys.stdin
toml_file = sys.argv[1]
timestamp = sys.argv[2]

try:
    import tomllib
except ImportError:
    import tomli as tomllib

# Build accession → hash lookup from manifest (stdin)
file_hashes = {}
for line in manifest_path:
    line = line.strip()
    if not line:
        continue
    parts = line.split('\t')
    if len(parts) >= 2:
        token = parts[0]
        bhash = parts[1]
        file_hashes[token.lower()] = bhash

with open(toml_file, 'rb') as f:
    data = tomllib.load(f)

lines = open(toml_file).readlines()
sources = data.get('sources', [])
updated = 0

for src in sources:
    accessions = src.get('accessions', [])
    src_id = src.get('id', '')
    matched_hash = ''

    # Try exact accession match against manifest tokens
    for acc in accessions:
        if not acc:
            continue
        acc_lower = acc.lower()
        # Exact match first
        if acc_lower in file_hashes:
            matched_hash = file_hashes[acc_lower]
            break
        # Try without version suffix (e.g., NC_000908.2 -> NC_000908)
        base = acc_lower.rsplit('.', 1)[0] if '.' in acc_lower else acc_lower
        if base in file_hashes:
            matched_hash = file_hashes[base]
            break

    if not matched_hash:
        continue

    # Find this source's blake3 field by locating the id line, then scanning forward
    in_source_block = False
    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped == f'id = "{src_id}"':
            in_source_block = True
        elif stripped == '[[sources]]' and in_source_block:
            break
        elif in_source_block and stripped == 'blake3 = ""':
            lines[i] = f'blake3 = "{matched_hash}"\n'
            if i + 1 < len(lines) and lines[i + 1].strip() == 'retrieved = ""':
                lines[i + 1] = f'retrieved = "{timestamp}"\n'
            updated += 1
            in_source_block = False

with open(toml_file, 'w') as f:
    f.writelines(lines)
print(updated)
PYEOF
    ) || updated=0

    if [[ "$updated" -gt 0 ]]; then
        log "  [$thread_name] Updated $updated entries"
        UPDATED=$((UPDATED + updated))
    else
        log "  [$thread_name] No accession matches found in fetched data"
        SKIPPED=$((SKIPPED + 1))
    fi
}

SOURCES_DIR="$FOUNDATION_ROOT/data/sources"
for toml_file in "$SOURCES_DIR"/*.toml; do
    [[ -f "$toml_file" ]] || continue
    [[ "$(basename "$toml_file")" == *.md ]] && continue
    backfill_source_toml "$toml_file"
done

echo ""
log "Backfill complete: $UPDATED updated, $SKIPPED skipped"
