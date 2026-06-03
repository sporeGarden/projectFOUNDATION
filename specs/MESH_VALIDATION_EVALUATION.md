# Mesh-Aware Validation — Evaluation

**Date**: June 3, 2026 (Wave 73)
**Status**: Evaluation — not yet implemented
**Context**: Mesh is now LIVE across gates. primalSpring asks whether
FOUNDATION should validate that a spring's lineage holds across gates.

## Question

Can FOUNDATION verify that a spring's lineage (reproducible scientific
results) holds when the spring runs on a different gate than where it was
originally validated?

## Analysis

### What FOUNDATION validates today

1. **Scientific correctness**: Do spring binaries produce numerically
   correct results? (compare against known targets)
2. **Data integrity**: Do fetched datasets match BLAKE3 checksums?
3. **Provenance chain**: Can the run be attributed to a gate, session, DAG?

All validation is **gate-local**: FOUNDATION runs on ironGate, executes
spring binaries on ironGate, compares results against targets stored on
ironGate.

### What mesh-aware validation would mean

Cross-gate validation would verify: "If hotSpring produces result X on
ironGate, does it produce the same result X on eastGate/southGate?"

This is **numerical reproducibility across hardware**, which depends on:
- CPU architecture (x86_64 vs aarch64)
- Floating-point implementation (fma, rounding modes)
- GPU availability and driver version
- Memory layout and allocation patterns

### Recommendation: NOT a FOUNDATION responsibility

Cross-gate parity is a **guideStone** responsibility, not a FOUNDATION
responsibility. The separation of concerns:

| Layer | Responsibility | Owner |
|-------|---------------|-------|
| **guideStone** | "Does this binary produce identical results on all certified substrates?" | primalSpring |
| **FOUNDATION** | "Does this spring's science reproduce the published literature?" | projectFOUNDATION |
| **NUCLEUS** | "Is this composition deployable and healthy across the mesh?" | projectNUCLEUS |

FOUNDATION validates **scientific truth** (does the code match the paper).
guideStone validates **substrate parity** (does the binary behave
identically across hardware). NUCLEUS validates **deployment health**
(is the composition alive and serving).

### What FOUNDATION CAN do for mesh

1. **Consume cross-gate provenance**: When a spring reports validation
   results from another gate, FOUNDATION can ingest that as additional
   evidence in the provenance chain. This requires no code changes —
   the provenance phase already records gate identity.

2. **Verify lineage consistency**: FOUNDATION can check that the SAME
   targets pass on ALL gates where a spring reports results. This is a
   comparison of provenance records, not a new execution.

3. **Gate-aware discovery**: If FOUNDATION needs to reach a primal on
   another gate (e.g., to query songBird for cross-gate events), the
   discovery config would need a `mesh` tier above UDS/TCP. This is a
   future transport evolution.

## Decision

**No immediate code changes required.** FOUNDATION's role in the mesh is:
- Continue validating science locally (gate-local execution)
- Record gate identity in provenance (already done via `gate_name`)
- Accept cross-gate provenance from upstream (passive consumption)

If primalSpring later emits cross-gate comparison data, FOUNDATION can
add a comparison phase that checks "did all gates agree?" This would be
a read-only comparison, not a cross-gate execution.

## Future: Discovery Config Evolution

If mesh transport is ever needed:

```toml
[mesh]
relay = "songBird"
topology = "star"
gates = ["ironGate", "eastGate", "southGate"]
```

This would add a 4th discovery tier: env → UDS → TCP → mesh relay.
Not needed today — FOUNDATION's validation is inherently local.
