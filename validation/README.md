# validation/

Validation results, provenance manifests, and gap reports from projectFOUNDATION
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

### Spring-oriented (current — post-CATHEDRAL split)

```
validation/
  <spring>/
    <YYYY-MM-DD>/
      results.json          ValidationResult export
      provenance.toml       Run metadata (tier, primals, duration, degradation state)
      braid.json            sweetGrass attribution braid
      *.stdout              Raw output logs
    braids/                 Ferment transcript braids (computation-verified provenance)
      <dataset_id>.json     Machine-verifiable upstream computation record
```

Example: `validation/hotSpring/2026-05-11/` for hotSpring's May 11 run.
Example: `validation/wetSpring/braids/barrick_2009_mutations.json` for ferment braid.

See `PROVENANCE_FOLDER_CONVENTION.md` for the full template.

Partial trio provenance is valid per the ecosystem degradation behavior
standard. See `docs/DEGRADATION_BEHAVIOR.md`.

### Legacy thread-oriented (preserved as geological record)

```
validation/
  <thread_short>-<date>/
    PROVENANCE_MANIFEST.md
    braid.json
    *.stdout
    VALIDATION_SUMMARY.md
```

Example: `validation/wcm-20260509/` for Thread 1 on May 9, 2026.

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
| 1 — Whole-Cell Modeling | 2026-05-09 | hotSpring, wetSpring, healthSpring | 0/27 validated | Fetch + CI gates validated; RPC upstream-blocked; 10/25 sources BLAKE3-anchored |
| 2 — Plasma Physics | 2026-05-11 | hotSpring v0.6.32 | 12/12 PASS | Validated |
| 3 — Immunology | — | healthSpring | 0/12 pending | Expression + targets ready, spring validation pending |
| 4 — Env Genomics | — | wetSpring, airSpring | 0/12 pending | Expression + targets ready, spring validation pending |
| 5 — LTEE / Evolution | — | groundSpring, wetSpring, hotSpring, neuralSpring | 14/18 partial | 4 pending — braid evidence from wetSpring ferment transcripts |
| 6 — Agricultural Science | 2026-05-11 | airSpring v0.10.0 | 36/36 PASS | Validated |
| 7 — Anderson Mathematics | 2026-05-11 | groundSpring V142 | 23/23 PASS | Validated (18→23 after target expansion) |
| 8 — Human Health | — | healthSpring | 0/11 pending | Expression ready, spring expanding |
| 9 — Gaming / Creative | — | ludoSpring | 0/13 seeded | Expression + targets ready |
| 10 — Provenance | — | primalSpring, ludoSpring | 0/8 seeded | Expression + targets ready |

Legacy runs: `wcm-20260509/`, `plasma-20260511/`, `ag-20260511/`, `anderson-20260511/`.
Future runs: `<spring>/<YYYY-MM-DD>/` per `PROVENANCE_FOLDER_CONVENTION.md`.

## Filing Gaps

If a validation run reveals a missing NUCLEUS capability:

1. Document the gap in the validation summary
2. Create a wateringHole handoff:
   `wateringHole/handoffs/FOUNDATION_<THREAD>_<GAP>_HANDOFF_<DATE>.md`
3. Reference the blocking primal and the specific method/capability
