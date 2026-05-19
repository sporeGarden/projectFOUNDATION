#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# target_compare.sh — Compare workload results against validation targets
#
# Sourced by foundation_validate.sh.
# Requires: FOUNDATION_ROOT, RESULTS_DIR, TARGET_HITS, TARGET_MISS, log()

resolve_target_file() {
    local short="$1"
    local dir="$2"
    # Explicit aliases for thread shorts that don't match filename patterns
    case "$short" in
        ml)          echo "$dir/thread05_ml_surrogates_targets.toml" ;;
        wcm)         echo "$dir/thread01_wcm_targets.toml" ;;
        plasma)      echo "$dir/thread02_plasma_targets.toml" ;;
        immuno)      echo "$dir/thread03_immuno_targets.toml" ;;
        enviro)      echo "$dir/thread04_enviro_targets.toml" ;;
        ltee)        echo "$dir/thread05_ltee_targets.toml" ;;
        ag)          echo "$dir/thread06_ag_targets.toml" ;;
        anderson)    echo "$dir/thread07_anderson_targets.toml" ;;
        health)      echo "$dir/thread08_health_targets.toml" ;;
        gaming)      echo "$dir/thread09_gaming_targets.toml" ;;
        provenance)  echo "$dir/thread10_provenance_targets.toml" ;;
        *)
            # Fallback: glob match
            # shellcheck disable=SC2086
            ls "$dir"/thread*_${short}*_targets.toml 2>/dev/null | head -1
            ;;
    esac
}

compare_targets() {
    local thread_short="$1"
    local match
    match=$(resolve_target_file "$thread_short" "$TARGET_DIR")
    if [[ -z "$match" || ! -f "$match" ]]; then
        log "  [INFO] No target file for thread: $thread_short"
        return 0
    fi

    local targets
    targets=$(python3 -c "
try:
    import tomllib
except ImportError:
    import tomli as tomllib
import sys
with open('$match', 'rb') as f:
    data = tomllib.load(f)
for t in data.get('targets', []):
    tid = t.get('id', '')
    expected = t.get('expected_value', t.get('expected', ''))
    tol = t.get('tolerance', '')
    tol_pct = t.get('tolerance_pct', '')
    print(f'{tid}|{expected}|{tol}|{tol_pct}')
" 2>/dev/null) || true

    if [[ -z "$targets" ]]; then
        return 0
    fi

    while IFS='|' read -r tid expected tolerance tol_pct; do
        [[ -z "$tid" ]] && continue

        local found=false actual_val=""
        for result_file in "$RESULTS_DIR"/*.stdout; do
            [[ -f "$result_file" ]] || continue
            actual_val=$(python3 -c "
import re, sys
pat = re.compile(re.escape('$tid') + r'[^0-9.eE+-]*([-+]?[0-9]*\.?[0-9]+(?:[eE][-+]?[0-9]+)?)')
for line in open('$result_file'):
    m = pat.search(line)
    if m:
        print(m.group(1))
        sys.exit(0)
" 2>/dev/null) || true
            if [[ -n "$actual_val" ]]; then
                found=true
                break
            fi
            if grep -q "$tid" "$result_file" 2>/dev/null; then
                found=true
                break
            fi
        done

        if $found; then
            if [[ -n "$actual_val" && -n "$expected" ]]; then
                local match
                match=$(python3 -c "
import sys
e, a = float('$expected'), float('$actual_val')
tol_abs = float('$tolerance') if '$tolerance' else None
tol_pct = float('$tol_pct') if '$tol_pct' else None
if tol_abs is not None:
    print('PASS' if abs(a - e) <= tol_abs else 'FAIL')
elif tol_pct is not None:
    denom = abs(e) if e != 0 else 1.0
    print('PASS' if abs(a - e) / denom * 100.0 <= tol_pct else 'FAIL')
else:
    print('SKIP')
" 2>/dev/null) || match="SKIP"
                if [[ "$match" == "PASS" ]]; then
                    log "  [OK]   $tid: $actual_val ≈ $expected (±${tolerance:-${tol_pct}%})"
                    TARGET_HITS=$((TARGET_HITS + 1))
                elif [[ "$match" == "FAIL" ]]; then
                    log "  [FAIL] $tid: $actual_val vs $expected (±${tolerance:-${tol_pct}%})"
                    TARGET_MISS=$((TARGET_MISS + 1))
                else
                    log "  [SKIP] $tid: could not compare (no tolerance defined)"
                fi
            else
                log "  [HIT]  $tid (expected: $expected)"
                TARGET_HITS=$((TARGET_HITS + 1))
            fi
        else
            log "  [MISS] $tid not found in workload output"
            TARGET_MISS=$((TARGET_MISS + 1))
        fi
    done <<< "$targets"
}
