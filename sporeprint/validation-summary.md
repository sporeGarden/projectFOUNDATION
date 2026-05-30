+++
title = "projectFOUNDATION Validation Summary"
date = 2026-05-30
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

## Current State (Wave 63+)

| Metric | Value |
|--------|------:|
| Rust workspace lines | 8,001 |
| Tests (unit + integration) | 150 |
| Binary size (ecoBin, zero C deps) | 3.2 MB |
| Domain threads | 10 |
| Data sources | 165 (across 11 manifests) |
| BLAKE3-anchored sources | 10 |
| Validation targets | 185 (across 11 manifests) |
| Workloads | 29 |
| CPU parity benchmarks | 6 scripts, 32 test cases |

## Thread Status

| Thread | Targets | Status |
|--------|--------:|--------|
| 1 — Whole-Cell Modeling | 27 | Fetch + CI validated; 10/25 sources BLAKE3-anchored |
| 2 — Plasma Physics | 12 | Validated (hotSpring) |
| 3 — Immunology | 12 | 12/12 spring-validated |
| 4 — Environmental Genomics | 12 | 8/12 partial |
| 5 — LTEE / Evolution | 18 | 14/18 partial; ferment braids pending |
| 6 — Agricultural Science | 36 | Validated (airSpring) |
| 7 — Anderson Mathematics | 23 | Validated (groundSpring) |
| 8 — Human Health | 11 | 11/11 spring-validated |
| 9 — Gaming / Creative | 13 | 13/13 spring-validated |
| 10 — Provenance | 9 | 5/9 partial (NC-1 spore ingest added) |

## Pipeline

Foundation validation runs through an 8-phase pipeline, implemented in both
the `foundation` Rust UniBin and the canonical bash pipeline:

### Rust UniBin (`foundation validate`)

Phase B complete, Phase C in progress. Type-safe, ecoBin-compliant, IPC
wired with graceful degradation:

1. **Health** — `HealthTriad` probes `VALIDATION_PRIMALS` via JSON-RPC
2. **Provenance open** — `dag.session.create` via rhizoCrypt (degrades gracefully)
3. **Fetch check** — Verifies source data availability from `ThreadIndex` manifests
4. **Registry** — `ArtifactRegistry` BLAKE3 scan of fetched data
5. **Execute** — Native subprocess with timeout enforcement
6. **Compare** — Typed tolerance checking against target manifests
7. **Provenance commit** — `dag.session.commit` finalization
8. **Report** — Structured Markdown with per-workload and per-target results

### Bash pipeline (`foundation_validate.sh`)

Production-canonical until Phase C cutover. Full NestGate registration,
toadStool dispatch, and provenance trio commit.

## Crate Architecture

```
foundation-core      Types, TOML parsing, config, env_keys, primal_names
foundation-ipc       JSON-RPC 2.0 clients, HealthTriad, ProvenanceSession
foundation-fetch     Manifest-driven fetch, BLAKE3 content addressing
foundation-validate  8-phase pipeline, executor, comparison, reporting
foundation-cli       UniBin entry: validate, fetch, health, targets, backfill
```

## Key Patterns

- **Manifest-driven**: All paths from `ThreadIndex` fields, not hardcoded
- **Type-safe enums**: `ExecType`, `IsolationLevel`, `SkipCondition`, `IpcPhase`
- **Zero-copy**: `Cow<str>` env expansion, `bytes::Bytes` in IPC transport
- **Graceful degradation**: Unreachable primals → warnings, not abort
- **ecoBin compliant**: Pure Rust, zero C dependencies, 3.0 MB stripped binary
- **Centralized identity**: `env_keys` module, `primal_names` constants

## Source

- [projectFOUNDATION on GitHub](https://github.com/sporeGarden/projectFOUNDATION)
- [projectFOUNDATION on Forgejo](https://git.primals.eco/sporeGarden/projectFOUNDATION)
