# foundation_validate.sh — Rust Elevation Feasibility Review

**Date:** 2026-05-16 (updated May 27 for Wave 55 context)
**Status:** Phase A complete, Phase B unblocked by primalSpring v0.9.30
**Referenced by:** lithoSpore UPSTREAM_GAPS.md, primalSpring CROSS_SPRING_PARITY_SCORECARD

## Current State

`deploy/foundation_validate.sh` is a ~712-line bash script (plus ~509 lines in
4 sourced libraries: `json_rpc.sh`, `primal_ipc.sh`, `target_compare.sh`,
`thread_registry.sh`) orchestrating 8 phases:

1. Health-check 7 NUCLEUS primals (HTTP + JSON-RPC + UDS)
2. Create provenance session (rhizoCrypt DAG + loamSpine spine)
3. Fetch data sources (delegates to `fetch_sources.sh`)
4. Register artifacts in NestGate with BLAKE3
5. Execute workloads through toadStool
6. Compare results against validation targets
7. Commit provenance (Merkle root + sweetGrass braid)
8. Write validation report

## Why Elevate

- **Phase 6 target comparison** requires TOML parsing with mixed schemas
  (expected_value vs expected, tolerance vs tolerance_pct) — fragile in bash
- **JSON-RPC IPC** to 7 primals via mixed transports (HTTP, TCP, UDS) —
  each has different framing requirements
- **BLAKE3 hashing** of all fetched files — native Rust is 10x+ faster
- **Error handling** — bash `|| true` silently swallows RPC failures
- **Type safety** — target TOML schema variants need proper sum types
- **Testability** — zero behavioral tests possible in current form
- **ecoBin compliance** — the ecosystem standard targets pure Rust binaries

## Proposed Architecture: `foundation` UniBin

A single `foundation` binary following the UniBin pattern:

```
foundation validate [--thread THREAD] [--skip-fetch] [--data-dir DIR]
foundation fetch [--thread THREAD] [--data-dir DIR] [--register]
foundation backfill [--data-dir DIR] [--dry-run]
foundation health [--verbose]
foundation targets [--thread THREAD] [--check]
```

### Crate Structure

```
crates/
  foundation-core/     shared types: Thread, Target, Source, Tolerance
  foundation-ipc/      typed JSON-RPC clients for all 7 primals
  foundation-fetch/    manifest-driven data fetch (replaces fetch_sources.sh)
  foundation-validate/ validation pipeline (replaces foundation_validate.sh)
  foundation-cli/      UniBin CLI entry point
```

### Dependencies (all pure Rust, ecoBin compliant)

| Crate | Purpose | ecoBin status |
|-------|---------|---------------|
| `clap` | CLI parsing | pure |
| `serde` + `toml` | TOML manifest parsing | pure |
| `serde_json` | JSON-RPC construction/parsing | pure |
| `blake3` (features=["pure","std"]) | Content addressing | pure |
| `ureq` | HTTP client (health checks, fetch) | pure |
| `tokio` (optional) | Async UDS + TCP for IPC | pure |
| `chrono` | Timestamps | pure |
| `walkdir` | File tree walking | pure |

### Phase-by-Phase Mapping

| Phase | Bash implementation | Rust equivalent |
|-------|-------|------|
| 1. Health | `curl`/`nc`/`pgrep` per primal | `foundation-ipc::HealthClient` — typed per-primal health check |
| 2. Session | `rpc_rhizocrypt`/`rpc_loamspine` | `foundation-ipc::RhizoCryptClient::create_session()` |
| 3. Fetch | Delegates to `fetch_sources.sh` | `foundation-fetch` — reads `data/sources/*.toml`, downloads per-accession |
| 4. Register | Walk `$DATA_DIR`, BLAKE3 + NestGate RPC | `foundation-validate::register_artifacts()` with `walkdir` + `blake3` |
| 5. Workloads | Parse workload TOML, exec or toadstool dispatch | `foundation-validate::execute_workloads()` — native or IPC |
| 6. Compare | Python-in-bash TOML extraction + grep | `foundation-validate::compare_targets()` — typed `Tolerance` enum |
| 7. Commit | Three RPC calls (DAG complete, spine append, braid create) | `foundation-ipc::commit_provenance()` |
| 8. Report | `cat > VALIDATION_REPORT.md << REPORT` | `foundation-validate::write_report()` — structured Markdown generation |

### Tolerance Type (eliminates Phase 6 schema mismatch)

```rust
enum Tolerance {
    Absolute { value: f64 },
    Percentage { value: f64 },
    Qualitative { description: String },
}

struct Target {
    id: String,
    expected_value: Option<f64>,
    expected_string: Option<String>,
    tolerance: Tolerance,
    paper: String,
    spring: String,
    // ...
}
```

### IPC Client (capability-based discovery)

```rust
struct PrimalClient {
    name: String,
    transport: Transport, // UDS | TCP | HTTP
    port: Option<u16>,
}

enum Transport {
    Uds(PathBuf),
    Tcp(SocketAddr),
    Http(Url),
}

impl PrimalClient {
    async fn discover(name: &str) -> Self { /* UDS → env → default */ }
    async fn health(&self) -> Result<bool, IpcError> { /* ... */ }
    async fn rpc(&self, method: &str, params: Value) -> Result<Value, IpcError> { /* ... */ }
}
```

## Effort Estimate

| Component | Lines (est.) | Complexity | Weeks |
|-----------|---:|---|---|
| `foundation-core` (types, tolerances, TOML parsing) | ~400 | Low | 0.5 |
| `foundation-ipc` (7 primal clients, transport discovery) | ~600 | Medium | 1.0 |
| `foundation-fetch` (manifest-driven download) | ~500 | Medium | 1.0 |
| `foundation-validate` (pipeline + target comparison) | ~700 | Medium | 1.0 |
| `foundation-cli` (UniBin shell) | ~200 | Low | 0.5 |
| Tests | ~800 | Medium | 1.0 |
| **Total** | **~3,200** | | **5 weeks** |

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| fetch_sources.sh has NCBI rate-limiting logic | `ureq` with configurable delay; respect `NCBI_API_KEY` |
| rhizoCrypt UDS framing is non-trivial | Use `foundation-ipc` to centralize; test against live rhizoCrypt |
| workload execution delegates to toadStool or runs directly | Keep `Command::new()` fallback for non-toadStool environments |
| Parallel with lithoSpore chassis evolution | Share `litho-core` tolerance/provenance types where possible |

## Signal Adoption (Wave 18)

When the Rust elevation lands, the `foundation` UniBin should adopt the
Neural API signal dispatch model per `SIGNAL_ADOPTION_STANDARD.md`:

| Current (bash 4-call) | Target (Rust signal) |
|----------------------|---------------------|
| `rpc_nestgate storage.store` + `rpc_rhizocrypt dag.event.append` + `rpc_loamspine entry.append` + `rpc_sweetgrass braid.create` | `ctx.dispatch("nest.store", data)` |
| `rpc_rhizocrypt dag.session.complete` + `rpc_loamspine entry.append` (SessionCommit) + `rpc_sweetgrass braid.create` | `ctx.dispatch("nest.commit", session)` |

This collapses the Phase 4 registration and Phase 7 provenance commit
from 4+ sequential RPC calls to 1 signal dispatch each. biomeOS manages
sequencing, error recovery, and partial failure rollback.

The bash pipeline cannot adopt signals (no `CompositionContext`), but the
current 4-call pattern with response validation is the correct interim.

## Recommendation

**Elevate in phases:**

1. **Phase A (complete):** Bash script fixes — Phase 2 params, Phase 6
   schema alignment, trusted_directories, modularization, skip counting,
   provenance response validation. Shipped May 16, 2026.
2. **Phase B (next sprint — unblocked by Wave 46):** Extract `foundation-core`
   types + `foundation-ipc` clients. These are reusable across lithoSpore and
   other gardens. Use `CompositionContext` from primalSpring for IPC.
   **Wave 46 provides:** `CompositionContext::from_live_discovery_with_fallback()`,
   `env_keys.rs` (FAMILY_ID, socket/manifest discovery, port overrides),
   `DispatchError` / `PhasedIpcError` typed error system, 5-tier discovery
   chain (songbird → NeuralAPI → UDS → manifest → TCP opt-in).
3. **Phase C (following sprint):** Replace `foundation_validate.sh` with
   `foundation validate` UniBin command. Adopt `ctx.dispatch("nest.store")`
   and `ctx.dispatch("nest.commit")` for signal-based provenance (14 atomic
   signals available). Use `primalspring::env_keys` for all env resolution.
4. **Phase D:** Replace `fetch_sources.sh` with `foundation fetch`.
   At this point the repo is pure Rust + TOML + Markdown.

This progression lets the bash script keep working while Rust phases land
incrementally. Each phase is independently useful and testable.

### Wave 55 Context (primalSpring v0.9.30)

The primal/spring layer is at zero gate debt. Key upstream APIs for Phase B:

| API | Module | Foundation use |
|-----|--------|----------------|
| `CompositionContext::discover()` | `composition/context.rs` | Replace `discover_port()` + `rpc_*()` |
| `CompositionContext::dispatch()` | `composition/context.rs` | Replace 4-call RPC sequences |
| `env_keys::FAMILY_ID` | `env_keys.rs` | Replace `${FAMILY_ID:-}` bash |
| `env_keys::NESTGATE_PORT` etc | `env_keys.rs` | Replace hardcoded port defaults |
| `DispatchError` | `composition/neural_dispatch.rs` | Replace `\|\| true` silent failures |
| `PhasedIpcError` | `ipc/error.rs` | Typed IPC error chains |
| `primal.announce` | IPC standard | Single-call registration (12/12 compliant) |
| `nucleus.ingest_spore` | capability_registry.toml | New Wave 55 — spore gateway (NC-1) |
| `nucleus.emit_spore` | capability_registry.toml | New Wave 55 — spore retrieval (NC-1) |

The 460-method registry and 56-scenario test suite validates the surface
foundation-ipc will consume.

### postPrimordial Spore Flow (NC-1 / NC-5)

Wave 55 introduces the three-era provenance model and NC-5 emission contract.
Foundation Thread 10 is the natural touchpoint:

**Era model**: Era 1 (ad-hoc) → Era 2 (pipeline, v1.6.1) → Era 3 (NUCLEUS
Nest deploy, filled trio braid). Foundation validation runs produce Era 2
provenance today; the `nucleus-spore-ingest` workload targets Era 3.

**Signal composition**: `nest_ingest_spore` composes existing primal capabilities
(NestGate store → rhizoCrypt DAG → loamSpine ledger → sweetGrass braid →
BearDog sign) via a 6-step signal graph. No new primal methods needed — only
biomeOS orchestration (v3.77+ CLI).

**Spore ownership split**: domain science (springs), envelope (`pseudospore-core`
/ lithoSpore), gateway (`biomeos nucleus ingest/emit`). Foundation defines the
science; lithoSpore packages it; biomeOS routes it through NUCLEUS.

Phase B can share types with lithoSpore's `pseudospore-core` for the
receipt/checksum layer, accelerating convergence.
