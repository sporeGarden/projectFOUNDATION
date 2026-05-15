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

Example: `validation/wcm-20260509/` for a Thread 1 whole-cell modeling
validation run on May 9, 2026.

## Current State

### Handbacks (from projectNUCLEUS)

The `handbacks/` directory contains gap reports and pattern handbacks
produced during NUCLEUS deployment validation. These are geological
records — evidence of what worked, what broke, and what upstream primals
need to evolve:

- `SECURITY_HANDBACK_MAY06_2026.md` — Five-layer security validation results
- `UPSTREAM_GAPS_HANDBACK_MAY06_2026.md` — Gaps requiring upstream primal evolution
- `JUPYTERHUB_PATTERNS_HANDBACK.md` — Multi-user patterns validated in production
- `NESTGATE_CONTENT_GAPS_HANDBACK.md` — Content-addressed storage gaps
- `PETALTONGUE_GAPS_HANDBACK.md` — Self-hosted rendering gaps
- `PRIMAL_DEEP_DEBT_HANDBACK.md` — Technical debt across primal implementations
- `ROOTPULSE_GAPS_HANDBACK.md` — Monitoring and metrics gaps
- `UPSTREAM_AUDIT_PREP_MAY15_2026.md` — Pre-push audit prep (lithoSpore + petalTongue refactor summary)
- `../COMPOSITION_GAPS.md` — Composition-level capability mismatches (at `validation/COMPOSITION_GAPS.md`)

### Validation Runs

| Thread | Date | Spring | Targets | Status |
|--------|------|--------|---------|--------|
| 1 — Whole-Cell Modeling | 2026-05-09 | hotSpring, wetSpring, healthSpring | 0/24 validated | Attempted — `fetch_sources.sh` post-download validation added (size + gzip checks). rhizoCrypt RPC + ToadStool sandbox upstream-blocked. See `wcm-20260509/` |
| 2 — Plasma Physics | 2026-05-11 | hotSpring v0.6.32 | 12/12 PASS | Validated |
| 6 — Agricultural Science | 2026-05-11 | airSpring v0.10.0 | 36/36 PASS | Validated |
| 7 — Anderson Mathematics | 2026-05-11 | groundSpring V132 | 18/18 PASS | Validated |

See `wcm-20260509/`, `plasma-20260511/`, `ag-20260511/`, `anderson-20260511/` for provenance manifests and logs.

## Filing Gaps

If a validation run reveals a missing NUCLEUS capability:

1. Document the gap in the validation summary
2. Create a wateringHole handoff:
   `wateringHole/handoffs/FOUNDATION_<THREAD>_<GAP>_HANDOFF_<DATE>.md`
3. Reference the blocking primal and the specific method/capability
