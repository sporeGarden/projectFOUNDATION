# Thread 6 (Agricultural Science) — GuideStone Instance Proposal

**Date:** 2026-05-16
**Status:** Draft
**Standard:** TARGETED_GUIDESTONE_STANDARD v1.0

## Rationale

Thread 6 is the strongest candidate for the second lithoSpore instance:

- **36/36 targets PASS** (validated 2026-05-11, airSpring v0.10.0)
- **6 workloads** exercising distinct validation scenarios
- **16 published papers** cross-validated
- **braid.json** with full sweetGrass provenance chain
- **Frozen BLAKE3 artifacts** for reproducibility
- Two contributing springs with mature validation: airSpring + groundSpring

## Proposed Artifact Name

`ag-guidestone` — scoped to Thread 6 (Agricultural Science).

## scope.toml (draft)

See `specs/ag_guidestone_scope.toml` for the machine-readable version.

## Module Crate Mapping

The LTEE guideStone uses 7 module crates (ltee-fitness, ltee-mutations, etc.)
each wrapping a domain of validation. For Thread 6, map the 6 existing
workloads to lithoSpore-compatible module crates:

| Workload TOML | airSpring scenario | Proposed module crate | Domain |
|---|---|---|---|
| `airspring-et0-fao56.toml` | `fao56-et0` | `ag-et0` | Reference evapotranspiration (FAO-56 PM) |
| `airspring-et0-methods.toml` | `et0-methods` | `ag-et0` | Multi-method ET0 (PT, TH, HS, MK, TC, HM, BC) |
| `airspring-water-balance.toml` | `water-balance` | `ag-hydro` | Water balance + yield response (Stewart, SCS-CN) |
| `airspring-soil-physics.toml` | `soil-physics` | `ag-hydro` | Pedotransfer, Richards equation, Green-Ampt |
| `airspring-atlas-pipeline.toml` | `atlas-pipeline` | `ag-atlas` | Michigan Crop Water Atlas (ERA5 + ET0 + yield) |
| `airspring-full-suite.toml` | `paper-chain` | `ag-cli` | Unified runner (all modules, all papers) |

### Proposed Crate Structure

```
crates/
  litho-core/          (shared — from existing lithoSpore)
  ag-et0/              evapotranspiration validation (FAO-56, 7 methods)
  ag-hydro/            soil physics + water balance
  ag-atlas/            Michigan atlas pipeline
  ag-cli/              unified CLI (litho validate --scope ag)
```

Each module crate would:
1. Import `litho-core` for shared validation harness, tolerance types, provenance
2. Wrap the corresponding `airspring validate --scenario` invocation
3. Parse expected values from `data/targets/thread06_ag_targets.toml`
4. Compare against golden outputs in `validation/expected/`
5. Produce structured JSON validation output

## Data Manifest (data.toml)

The `data/sources/thread06_ag.toml` manifest lists 18 sources across:
- NOAA GHCN-Daily weather stations
- FAO-56 reference tables
- ERA5 reanalysis data
- NCBI SRA 16S rRNA (soil microbiome, cross-spring with wetSpring)
- IRIS FDSN seismic (cross-thread with groundSpring)
- Literature references (7 ET0 method papers)

## Tier Model

- **Tier 1 (Python):** Python baselines from airSpring `control/` scripts
- **Tier 2 (Rust static):** `ag-cli` musl-static binary runs all 6 scenarios
- **Tier 3 (NUCLEUS):** Full composition with provenance trio + toadStool dispatch

## Next Steps

1. Create `ag-guidestone` scope.toml in lithoSpore (or new artifact repo)
2. Scaffold crate structure mirroring LTEE pattern
3. Extract golden outputs from airSpring frozen artifacts into `validation/expected/`
4. Wire `tolerances.toml` from `thread06_ag_targets.toml` field values
5. Build musl-static Tier 2 binary
6. Run clean-machine Tier 1 + Tier 2 validation
7. Register in wateringHole GuideStone registry
