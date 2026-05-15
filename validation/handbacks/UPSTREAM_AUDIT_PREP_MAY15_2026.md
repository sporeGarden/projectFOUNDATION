# Upstream Audit Preparation — primalPing Review

**Date**: 2026-05-15
**From**: CATHEDRAL (lithoSpore + foundation)
**For**: primalPing, upstream primal teams

## Summary

Both `lithoSpore` and `foundation` repos have been cleaned for upstream
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

## foundation — Open Items for Upstream Teams

### Data Integrity

| Item | Action | Owner |
|------|--------|-------|
| `data/sources/*.toml` — all `blake3 = ""` | Run `deploy/backfill_hashes.sh` after fetching | CATHEDRAL (needs fetch infrastructure) |
| Thread 1 WCM — 0/24 targets validated | Review `validation/wcm-20260509/`, flip where justified | CATHEDRAL |
| Thread 5 ML — `accessions = []` | Document as `source_type = "internal"` (neuralSpring models) | neuralSpring team |

### Validation State

| Thread | Last Run | Status |
|--------|----------|--------|
| 1 — Whole-Cell Modeling | 2026-05-09 | Attempted — fetch infra validated, RPC upstream-blocked |
| 2 — Plasma Physics | 2026-05-11 | 12/12 PASS |
| 6 — Agricultural Science | 2026-05-11 | 36/36 PASS |
| 7 — Anderson Mathematics | 2026-05-11 | 18/18 PASS |

### Handback Archive (geological record)

All handbacks in `validation/handbacks/` are dated snapshots. Each now
carries a banner noting that upstream state should be re-verified before
acting on the findings.

## Paper Count Reconciliation

- `THREAD_INDEX.toml`: 28 total baseCamp papers
- `BASECAMP_PAPER_MAP.toml`: 26 individually mapped (14, 23, 24 are meta)
- Both are correct; the distinction is now documented in both files

## Repos Ready for Push

Both repos pass local `cargo check` / `cargo test` / `cargo clippy`.
Foundation CI jobs (shellcheck, TOML syntax, thread index, hash coverage)
expected to pass on clean state.
