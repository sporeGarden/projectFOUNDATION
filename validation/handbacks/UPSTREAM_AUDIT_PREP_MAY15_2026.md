# Upstream Audit Preparation — primalPing Review

**Date**: 2026-05-15
**From**: CATHEDRAL (lithoSpore + projectFOUNDATION)
**For**: primalPing, upstream primal teams

## Summary

Both `lithoSpore` and `projectFOUNDATION` repos have been cleaned for upstream
audit. This document consolidates open items that require attention from
upstream primal teams.

## lithoSpore — Open Items for Upstream Teams

### Blocked on External

| Item | Owner | Status |
|------|-------|--------|
| Module 5 (BioBrick burden) | External — Burden 2024 DOI pending | Scaffold only |
| Songbird TURN client library | Songbird team | Stub (env-var discovery only) |
| BearDog FIDO2/CTAP2 witness | BearDog team | Not started |
| sporePrint pipeline wiring | sporePrint/Zola team | Not started |
| genomeBin Tier 3 USB packaging | genomeBin team | Not started |

### lithoSpore Deep Evolution (completed)

- `viz.rs` (1248 lines) refactored into `viz/mod.rs`, `viz/modules.rs`, `viz/baselines.rs`
- `ltee-cli/main.rs` (994 lines) refactored into subcommand modules: `validate.rs`, `visualize.rs`, `verify.rs`, `ops.rs`
- UDS RPC transport **implemented** (was stub returning `None`) — `rpc_uds()` via `UnixStream`
- Hardcoded IPs/env keys/socket paths evolved to capability-based discovery
- 13 unit + 8 integration tests added to `ltee-cli`

### petalTongue Deep Evolution (completed)

- `web_mode.rs` (1167 lines) refactored into `web_mode/mod.rs` + `web_mode/nestgate.rs`
- `scene_viewer.rs` (864 lines) refactored into `scene_viewer/mod.rs`, `scene_viewer/interaction.rs`, `scene_viewer/parameters.rs`
- 6 `#[allow(dead_code)]` evolved to `#[expect(dead_code, reason = "...")]` — SceneGraph supersession documented per-function
- Interactive SceneGraph pipeline fully operational (semantic data_id, click-to-select, pan/zoom, IPC bridge, animation, parameter controls)

### Discovery Chain

UDS RPC transport is **implemented**. TURN-relayed RPC remains a documented
stub. All callers degrade gracefully. Needs Songbird client library for
actual TURN relay IPC.

## projectFOUNDATION — Open Items for Upstream Teams

### Data Integrity

| Item | Action | Owner |
|------|--------|-------|
| `data/sources/*.toml` — all `blake3 = ""` | Run `deploy/backfill_hashes.sh` after fetching | CATHEDRAL (needs fetch infrastructure) |
| Thread 1 WCM — 0/27 targets validated | Review `validation/wcm-20260509/`, flip where justified | CATHEDRAL |
| Thread 5 ML — `accessions = []` | Document as `source_type = "internal"` (neuralSpring models) | neuralSpring team |

### Validation State

| Thread | Last Run | Status |
|--------|----------|--------|
| 1 — Whole-Cell Modeling | 2026-05-09 | Attempted — fetch infra validated, RPC upstream-blocked |
| 2 — Plasma Physics | 2026-05-11 | 12/12 PASS |
| 6 — Agricultural Science | 2026-05-11 | 36/36 PASS |
| 7 — Anderson Mathematics | 2026-05-11 | 18/18 PASS |

### Upstream Primal Gaps (for primalSpring audit)

| Primal | Gap | Impact |
|--------|-----|--------|
| rhizoCrypt | `dag.session.create` response schema undocumented — we infer `result.session_id` | Pipeline guesses at response structure |
| loamSpine | `entry.append` with `SessionCommit` — response format undocumented | Cannot distinguish partial vs full commit |
| sweetGrass | `braid.create` returns `result.urn` or `result.id` — format inconsistent | Pipeline handles both, but no canonical schema |
| toadStool | `trusted_directories` interaction with `working_dir` precedence undocumented | All 29 workloads set both; may not be necessary |
| Discovery | `capability.resolve` response schema and error cases undocumented | Pipeline falls back to env/defaults |
| NestGate | `storage.store` value is ad-hoc string `"blake3:$hash size:$size"` | Needs structured value schema or metadata fields |

### Deep Debt Evolution (completed May 16, 2026)

- Self-validating mocks eliminated (enviro-qs, anderson-math → real spring delegation)
- All 15 workloads upgraded from `isolation_level = "None"` to `"Standard"`
- Pipeline Phase 7 now validates all provenance RPC responses
- Pipeline Phase 4 now manifest-driven (reads source TOMLs before glob fallback)
- `[SKIP]` counting added throughout pipeline and report
- Script modularized (535 + 92 + 92 lines across 3 files)
- 33 `ironGate` → `irongate` naming fixes across 11 files
- CI expanded with target schema validation, workload integrity, gate naming enforcement
- Thread 9 gaming targets migrated to numeric schema
- Benchmark provenance headers added to all 3 Python baselines
- 6 thread06_ag workloads missing thread metadata fixed
- Stale paths and counts corrected in docs

See `wateringHole/handoffs/PROJECTFOUNDATION_DEEP_DEBT_EVOLUTION_HANDOFF_MAY16_2026.md`
for the full handoff.

### Handback Archive (geological record)

All handbacks in `validation/handbacks/` are dated snapshots. Each now
carries a banner noting that upstream state should be re-verified before
acting on the findings.

## Paper Count Reconciliation

- `THREAD_INDEX.toml`: 28 total baseCamp papers
- `BASECAMP_PAPER_MAP.toml`: 26 individually mapped (14, 23, 24 are meta)
- Both are correct; the distinction is now documented in both files

## Repos Ready for Push

projectFOUNDATION CI jobs (shellcheck, TOML syntax, target schema, workload integrity,
thread index, hash coverage, gate naming) expected to pass on clean state.
All 184 targets validated against schema, 29 workloads pass integrity check.
