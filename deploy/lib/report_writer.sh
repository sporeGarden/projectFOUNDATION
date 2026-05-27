#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# report_writer.sh — Phase 8 report generation and spring folder distribution
#
# Sourced by foundation_validate.sh after workload execution and provenance chain.
# Expects caller to have set: SESSION_NAME, THREAD_FILTER, GATE_NAME,
#   SESSION_ID, MERKLE_ROOT, SPINE_ID, BRAID_URN, EVENT_IDX,
#   TOTAL_OK, TOTAL_FAIL, TOTAL_SKIP, TARGET_HITS, TARGET_MISS,
#   PROVENANCE_WARN, DISCOVERY_FALLBACK_COUNT, RESULTS_DIR, FOUNDATION_ROOT,
#   ARTIFACT_TABLE, WORKLOAD_TABLE, BRAID_RESP

write_validation_report() {
    local TRIO_STATE
    TRIO_STATE=$(compute_trio_state "$SESSION_ID" "$SPINE_ID" "$BRAID_URN")

    cat > "$RESULTS_DIR/VALIDATION_REPORT.md" << REPORT
# Foundation Validation Report

**Session**: $SESSION_NAME
**Thread**: $THREAD_FILTER
**Date**: $(date -Iseconds)
**NUCLEUS Gate**: $GATE_NAME

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

| Workload | OK | FAIL | SKIP | Time | Status |
|----------|---:|-----:|-----:|-----:|--------|
$(echo -e "$WORKLOAD_TABLE")

**Total**: $TOTAL_OK OK / $TOTAL_FAIL FAIL / $TOTAL_SKIP SKIP

## Target Comparison

| Metric | Count |
|--------|------:|
| Targets hit | $TARGET_HITS |
| Targets missed | $TARGET_MISS |

## Degradation State

| Aspect | Value |
|--------|-------|
| Discovery fallbacks | $DISCOVERY_FALLBACK_COUNT |
| Provenance warnings | $PROVENANCE_WARN |
| Trio state | $(trio_state_label "$TRIO_STATE") |

> Science is never gated behind primal availability. Partial provenance
> is valid provenance. See \`docs/DEGRADATION_BEHAVIOR.md\`.

## Sediment Layer

This validation run is now a permanent layer in the foundation's
geological record. The Merkle root anchors the complete provenance
chain: data sources → computation → results → attribution.

Springs that absorb these patterns will strengthen the layer by adding
their own validation results, which flow back here as new sediment.
REPORT

    log "  [OK] Report: $RESULTS_DIR/VALIDATION_REPORT.md"
}

write_provenance_toml() {
    local TRIO_STATE
    TRIO_STATE=$(compute_trio_state "$SESSION_ID" "$SPINE_ID" "$BRAID_URN")

    cat > "$RESULTS_DIR/provenance.toml" << PROVTOML
[run]
session = "$SESSION_NAME"
thread = "$THREAD_FILTER"
date = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
gate = "$GATE_NAME"
composition = "nest"

[results]
total_ok = $TOTAL_OK
total_fail = $TOTAL_FAIL
total_skip = $TOTAL_SKIP
target_hits = $TARGET_HITS
target_misses = $TARGET_MISS
events = $EVENT_IDX

[provenance]
dag_session_id = "$SESSION_ID"
merkle_root = "$MERKLE_ROOT"
spine_id = "$SPINE_ID"
braid_urn = "$BRAID_URN"
provenance_warnings = $PROVENANCE_WARN

[degradation]
discovery_fallbacks = $DISCOVERY_FALLBACK_COUNT
provenance_partial = $PROVENANCE_WARN
trio_state = "$TRIO_STATE"
PROVTOML

    log "  [OK] Provenance: $RESULTS_DIR/provenance.toml"
}

write_results_json() {
    local TRIO_STATE
    TRIO_STATE=$(compute_trio_state "$SESSION_ID" "$SPINE_ID" "$BRAID_URN")

    python3 -c "
import json
results = {
    'schema': 'foundation-validation-result/v1',
    'session': '$SESSION_NAME',
    'thread': '$THREAD_FILTER',
    'date': '$(date -u +%Y-%m-%dT%H:%M:%SZ)',
    'gate': '$GATE_NAME',
    'composition': 'nest',
    'results': {
        'ok': $TOTAL_OK,
        'fail': $TOTAL_FAIL,
        'skip': $TOTAL_SKIP,
        'target_hits': $TARGET_HITS,
        'target_misses': $TARGET_MISS,
        'events': $EVENT_IDX,
    },
    'provenance': {
        'dag_session_id': '$SESSION_ID',
        'merkle_root': '$MERKLE_ROOT',
        'spine_id': '$SPINE_ID',
        'braid_urn': '$BRAID_URN',
    },
    'degradation': {
        'discovery_fallbacks': $DISCOVERY_FALLBACK_COUNT,
        'provenance_warnings': $PROVENANCE_WARN,
        'trio_state': '$TRIO_STATE',
    },
}
with open('$RESULTS_DIR/results.json', 'w') as f:
    json.dump(results, f, indent=2)
" 2>/dev/null
    log "  [OK] Results JSON: $RESULTS_DIR/results.json"
}

resolve_workload_spring() {
    local workload_name="$1"
    local toml_match
    toml_match=$(find "$FOUNDATION_ROOT/workloads" -name "${workload_name}.toml" -print -quit 2>/dev/null)
    if [[ -n "$toml_match" && -f "$toml_match" ]]; then
        local spring
        spring=$(python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib
with open('$toml_match', 'rb') as f:
    data = tomllib.load(f)
prov = data.get('provenance', {})
meta = data.get('metadata', {})
s = prov.get('upstream_spring', '') or meta.get('spring_upstream', '') or meta.get('spring', '')
print(s)
" 2>/dev/null) || spring=""
        if [[ -n "$spring" ]]; then echo "$spring"; return; fi
    fi
    case "$workload_name" in
        hs-*|hotspring*)      echo "hotSpring" ;;
        gs-*|groundspring*)   echo "groundSpring" ;;
        ws-*|wetspring*)      echo "wetSpring" ;;
        ns-*|neuralspring*)   echo "neuralSpring" ;;
        airspring*)           echo "airSpring" ;;
        healthspring*)        echo "healthSpring" ;;
        ludospring*)          echo "ludoSpring" ;;
        primalspring*)        echo "primalSpring" ;;
    esac
}

distribute_to_spring_folders() {
    local validation_base="$FOUNDATION_ROOT/validation"
    local run_date
    run_date=$(date +%Y-%m-%d)

    for stdout_file in "$RESULTS_DIR"/*.stdout; do
        [[ -f "$stdout_file" ]] || continue
        local local_name target_spring dated_dir
        local_name=$(basename "$stdout_file" .stdout)
        target_spring=$(resolve_workload_spring "$local_name")
        if [[ -n "$target_spring" ]]; then
            dated_dir="$validation_base/$target_spring/$run_date"
            mkdir -p "$dated_dir"
            cp "$RESULTS_DIR/provenance.toml" "$dated_dir/" 2>/dev/null || true
            cp "$RESULTS_DIR/results.json" "$dated_dir/" 2>/dev/null || true
            cp "$RESULTS_DIR/braid.json" "$dated_dir/" 2>/dev/null || true
            cp "$stdout_file" "$dated_dir/" 2>/dev/null || true
        fi
    done
}
