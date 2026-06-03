# guideStone / FOUNDATION Validation Boundary Specification

**Date**: June 3, 2026 (Wave 74)
**Status**: Specification — defines validation responsibilities
**Upstream**: primalSpring (owns guideStone)
**Local**: projectFOUNDATION (owns scientific validation)

## Purpose

This document defines the boundary between what `guideStone` validates
(substrate parity across gates) and what `projectFOUNDATION` validates
(scientific truth), and specifies their shared interface.

## Validation Domains

### What FOUNDATION Validates

FOUNDATION owns **scientific correctness** — the question:
"Does this spring's code reproduce the published scientific results?"

| Dimension | Description | Example |
|-----------|-------------|---------|
| **Numerical accuracy** | Computed values match known targets within tolerance | hotSpring's hydrothermal model ±0.001% |
| **Data integrity** | Source datasets are intact (BLAKE3 verification) | CSV/JSON checksums match fetched data |
| **Provenance** | Every validation run is attributable | gate, session, DAG hash, timestamp |
| **Lineage** | baseCamp papers map to specific spring capabilities | 26 papers → 8 springs → 10 threads |
| **Regression detection** | Previous results are not lost when springs evolve | Version bump → re-verify targets |

FOUNDATION does **not** validate:
- Whether results are consistent across different hardware
- Whether the binary is deployable
- Whether the composition graph is healthy

### What guideStone Validates

guideStone owns **substrate parity** — the question:
"Does this binary produce identical results on all certified substrates?"

| Dimension | Description | Example |
|-----------|-------------|---------|
| **Cross-gate reproducibility** | Same inputs → same outputs on different gates | ironGate vs eastGate produce same float |
| **Architecture parity** | x86_64 and aarch64 produce identical results | Verified via cross-compilation tests |
| **Determinism** | Multiple runs on same gate are bitwise identical | No uninitialized memory, stable ordering |
| **Certification** | A gate is "certified" for a spring when parity holds | eastGate certified for hotSpring v0.6.32 |

guideStone does **not** validate:
- Whether the science is correct (that's FOUNDATION)
- Whether the binary is healthy/alive (that's NUCLEUS)
- Data source integrity or availability

### What NUCLEUS Validates

For completeness — NUCLEUS owns **deployment health**:
- Is the composition alive and serving?
- Are all primals reachable?
- Can springs be deployed to target substrates?

## Interface: Shared Lineage Format

FOUNDATION and guideStone share data through a common lineage format.
This enables guideStone to consume FOUNDATION's validation targets and
compare them across gates.

### Shared Data Flow

```
FOUNDATION validates science → produces lineage records
                                         ↓
                              guideStone consumes lineage
                                         ↓
                              guideStone cross-gate comparison
                                         ↓
                              Parity report (pass/fail per gate)
```

### Lineage Record Schema (shared)

```toml
[record]
spring = "hotSpring"
version = "0.6.32"
gate = "ironGate"
thread = "thread01_thermo"
target_id = "hydrothermal_flux_kelvin"
expected = 4231.887
actual = 4231.887
tolerance = 0.001
passed = true
blake3_provenance = "abc123..."
timestamp = "2026-06-03T14:00:00Z"
```

guideStone consumes these records and compares the `actual` field
across gates. If all gates produce identical `actual` values (within
the defined `tolerance`), the spring is certified for cross-gate use.

### SPRING_VERSIONS.toml as Shared Reference

Both FOUNDATION and guideStone reference `SPRING_VERSIONS.toml`:

- **FOUNDATION** uses it for drift detection (`check-versions` command)
- **guideStone** uses it to know which version was last certified on which gate
- Updates flow: FOUNDATION bumps → guideStone re-certifies → primalSpring records

## Exit Criteria

The interface is considered stable when:

1. FOUNDATION produces machine-readable validation records (lineage TOML or JSON-RPC)
2. guideStone can query FOUNDATION for per-spring-per-gate results
3. Drift detection (`check-versions`) covers both scientific drift and parity drift

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| FOUNDATION lineage records | Active | THE_UNIFIED_LINEAGE.md + SPRING_VERSIONS.toml |
| FOUNDATION drift detection | Active (Wave 74) | `check-versions` subcommand |
| guideStone consumption | Not yet built | guideStone spec pending upstream |
| Cross-gate comparison | Not yet built | Blocked on guideStone implementation |
| JSON-RPC lineage query | Planned (P3) | Wave 74+ — health dashboard data |

## Coordination Notes

- guideStone is owned by **primalSpring** (upstream)
- FOUNDATION defines the lineage schema that guideStone will consume
- No code changes in FOUNDATION are needed until guideStone is built
- When guideStone becomes active, FOUNDATION may expose lineage via JSON-RPC
  (see P3: ecosystem health dashboard)
