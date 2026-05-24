#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# thread_registry.sh — Runtime thread metadata from THREAD_INDEX.toml
#
# Provides typed access to thread names, shorts, and paths.
# Replaces hardcoded thread lists in scripts.

THREAD_INDEX="${FOUNDATION_ROOT}/lineage/THREAD_INDEX.toml"

# List all thread short names (one per line).
# Used for --help text and scan directory resolution.
list_thread_shorts() {
    python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib
with open('$THREAD_INDEX', 'rb') as f:
    data = tomllib.load(f)
for t in data.get('threads', []):
    print(t['short'])
" 2>/dev/null
}

# Resolve a thread short name to its zero-padded directory prefix.
# Usage: dir_prefix=$(resolve_thread_dir "wcm")  # → "thread01"
resolve_thread_dir() {
    local short="$1"
    python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib
with open('$THREAD_INDEX', 'rb') as f:
    data = tomllib.load(f)
for t in data.get('threads', []):
    if t['short'] == '$short':
        print(f\"thread{t['id']:02d}\")
        break
" 2>/dev/null
}

# Get thread metadata as key=value pairs.
# Usage: eval "$(thread_meta "wcm")"
#        echo "$thread_name"  # "Whole-Cell Modeling"
thread_meta() {
    local short="$1"
    python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib
with open('$THREAD_INDEX', 'rb') as f:
    data = tomllib.load(f)
for t in data.get('threads', []):
    if t['short'] == '$short':
        print(f\"thread_id={t['id']}\")
        print(f\"thread_name='{t['name']}'\")
        print(f\"thread_dir=thread{t['id']:02d}\")
        print(f\"thread_sources={t.get('data_sources','')}\")
        print(f\"thread_targets={t.get('data_targets','')}\")
        break
" 2>/dev/null
}

# Resolve thread short to source manifest path(s) (one per line, absolute).
# Handles ML companion manifests (thread 5 → ltee + ml_surrogates).
# Usage: while IFS= read -r f; do ... done < <(resolve_thread_manifests "ltee")
resolve_thread_manifests() {
    local short="$1"
    local sources_dir="${FOUNDATION_ROOT}/data/sources"
    python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib
with open('$THREAD_INDEX', 'rb') as f:
    data = tomllib.load(f)
for t in data.get('threads', []):
    if t['short'] == '$short':
        src = t.get('data_sources', '')
        if src:
            print('$FOUNDATION_ROOT/' + src)
        ml = t.get('ml_data_sources', '')
        if ml:
            print('$FOUNDATION_ROOT/' + ml)
        break
" 2>/dev/null
}

# Resolve thread short to target manifest path(s) (one per line, absolute).
resolve_thread_targets() {
    local short="$1"
    python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib
with open('$THREAD_INDEX', 'rb') as f:
    data = tomllib.load(f)
for t in data.get('threads', []):
    if t['short'] == '$short':
        tgt = t.get('data_targets', '')
        if tgt:
            print('$FOUNDATION_ROOT/' + tgt)
        ml = t.get('ml_data_targets', '')
        if ml:
            print('$FOUNDATION_ROOT/' + ml)
        break
" 2>/dev/null
}

# Build help text for --thread argument from THREAD_INDEX.
thread_help_text() {
    echo "Available threads:"
    python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib
with open('$THREAD_INDEX', 'rb') as f:
    data = tomllib.load(f)
for t in data.get('threads', []):
    print(f\"  {t['short']:12s} Thread {t['id']:02d}: {t['name']}\")
print(f\"  {'all':12s} All threads\")
" 2>/dev/null
}
