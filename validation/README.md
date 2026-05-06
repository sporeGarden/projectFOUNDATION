# validation/

Validation results, provenance manifests, and gap reports from foundation
validation runs.

## What Goes Here

When a NUCLEUS composition validates a data thread expression, the
results are recorded here:

- **Provenance manifests**: rhizoCrypt DAG session exports, loamSpine
  ledger entries, sweetGrass braid records
- **Result summaries**: Per-thread validation logs noting which targets
  passed, which failed, and what gaps were discovered
- **Gap reports**: Capability mismatches, missing methods, or wire
  surprises discovered during validation — filed back to wateringHole

## Directory Convention

```
validation/
  <thread_short>-<date>/
    PROVENANCE_MANIFEST.md
    braid.json
    *.stdout
    VALIDATION_SUMMARY.md
```

Example: `validation/wcm-20260510/` for a Thread 1 whole-cell modeling
validation run on May 10, 2026.

## Current State

No validation runs have been executed yet. This directory will be
populated as spring experiments begin implementing thread expressions
and running them through NUCLEUS compositions.

## Filing Gaps

If a validation run reveals a missing NUCLEUS capability:

1. Document the gap in the validation summary
2. Create a wateringHole handoff:
   `wateringHole/handoffs/FOUNDATION_<THREAD>_<GAP>_HANDOFF_<DATE>.md`
3. Reference the blocking primal and the specific method/capability
