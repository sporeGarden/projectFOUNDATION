+++
title = "projectFOUNDATION Validation Summary"
date = 2026-05-20
template = "page.html"
[extra]
entity = "projectfoundation"
tier = "product"
+++

# projectFOUNDATION — Validation Summary

Scientific knowledge layer for the ecoPrimals sovereign compute ecosystem.
Defines **what** to validate across 10 domain threads spanning whole-cell
modeling, plasma physics, immunology, evolutionary biology, agricultural
science, and more.

## Coverage

| Metric | Value |
|--------|------:|
| Domain threads | 10 |
| Data sources | 165 (across 11 manifests) |
| BLAKE3-anchored sources | 10 |
| Validation targets | 184 (across 11 manifests) |
| Workloads | 29 |
| CPU parity benchmarks | 6 |

## Thread Status

| Thread | Targets | Status |
|--------|--------:|--------|
| 1 — Whole-Cell Modeling | 27 | Fetch + CI validated; 10/25 sources BLAKE3-anchored |
| 2 — Plasma Physics | 12 | Validated (hotSpring) |
| 3 — Immunology | 12 | Targets ready, spring pending |
| 4 — Environmental Genomics | 12 | Targets ready, spring pending |
| 5 — LTEE / Evolution | 18 | 14/18 partial; ferment braids pending |
| 6 — Agricultural Science | 36 | Validated (airSpring) |
| 7 — Anderson Mathematics | 23 | Validated (groundSpring) |
| 8 — Human Health | 11 | Targets ready, spring pending |
| 9 — Gaming / Creative | 13 | Expression + targets seeded |
| 10 — Provenance | 8 | Expression + targets seeded |

## Pipeline

Foundation validation runs through an 8-phase pipeline orchestrated by
`foundation_validate.sh`:

1. Health-check NUCLEUS primals (7 primals, 3 required)
2. Create provenance session (rhizoCrypt DAG + loamSpine spine)
3. Fetch data sources (manifest-driven from `data/sources/*.toml`)
4. Register artifacts in NestGate with BLAKE3
5. Execute workloads through toadStool
6. Compare results against validation targets
7. Commit provenance (Merkle root + loamSpine + sweetGrass braid)
8. Write `results.json`, `provenance.toml`, and `VALIDATION_REPORT.md`

## Key Patterns

- **Manifest-driven fetch**: `data/sources/*.toml` → dispatch by `database` field
- **Cross-tier parity**: Python baseline → Rust validator → Primal composition
- **Degradation behavior**: Science never gated behind primal availability
- **Typed IPC**: JSON-RPC responses parsed via typed helpers, not grep
- **Thread registry**: All thread metadata resolved from `lineage/THREAD_INDEX.toml`

## Source

- [projectFOUNDATION on GitHub](https://github.com/sporeGarden/projectFOUNDATION)
