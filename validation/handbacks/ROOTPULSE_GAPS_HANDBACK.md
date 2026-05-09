# Upstream Gaps — RootPulse Commit Workflow Execution

**Date**: 2026-05-07
**From**: projectNUCLEUS (ironGate)
**For**: biomeOS, rhizoCrypt, LoamSpine, sweetGrass teams, primalSpring
**Context**: Ported `rootpulse_commit.toml` from `biomeOS/graphs/` to
`projectNUCLEUS/graphs/`. Executed all 6 phases of the workflow against
the live 13-primal NUCLEUS composition on ironGate.

---

## Test Methodology

Executed each phase of `rootpulse_commit.toml` manually against the running
primals via JSON-RPC TCP connections. The graph specifies a 6-phase sequential
workflow through rhizoCrypt, BearDog, NestGate, LoamSpine, and sweetGrass.

All 6 primals involved in the workflow were alive and accepting RPC calls.

---

## Phase-by-Phase Results

### Phase 1: Health Checks — PASS

| Primal | Method | Port | Result |
|--------|--------|------|--------|
| rhizoCrypt | `health.liveness` | 9602 | `{"status":"alive"}` |
| LoamSpine | `health.liveness` | 9700 | `{"status":"alive"}` |

Both primals healthy. Graph phase 1 succeeds.

### Phase 2: Dehydrate (rhizoCrypt) — PASS

**Graph specifies**: `capability = "dag.dehydrate"` with `session_id`

**Execution sequence**:
1. `dag.session.create` — created session `019e0315-2289-71e0-97ba-b3f355379302`
2. `dag.event.append` — added `DataCreate` event (struct variant required)
3. `dag.dehydrate` — returned merkle root `fb4ee7c9...bd32`

**Findings**:
- The graph specifies `dag.dehydrate` as a `capability_call`, but it's an
  `rpc_call` in the actual API (this doesn't affect execution)
- `dag.event.append` requires struct-variant event types (not simple strings).
  The graph doesn't document the event structure needed before dehydration.
- A fresh session with zero events still returns a dehydration result
  (zero hash), which is valid but potentially surprising

### Phase 3: Sign (BearDog) — PASS

**Graph specifies**: `capability = "crypto.sign"` with `data` and `did`

**Actual API**: `crypto.sign` with `message` (not `data`)

**Execution**:
```json
Request:  {"method":"crypto.sign","params":{"message":"fb4ee7c9..."}}
Response: {
    "algorithm": "Ed25519",
    "key_id": "default_signing_key",
    "public_key": "HAtppluL269afWBcbDd1kcpqucpE3kB6Mo4elIpZSAk=",
    "signature": "0Eb84SAlKc+ubHeie0aX5ApR3PgNhd/O+gJa/OVTugIZpbdr..."
}
```

**Gap RP-1**: Graph says `params = { data = "...", did = "..." }` but the
actual method takes `message` (not `data`), and `did` is not a parameter.

### Phase 4: Store Content (NestGate) — PASS

**Graph specifies**: `rpc_call` to `storage.store` with `key`, `value`, `family_id`

**Actual API**: Matches the graph specification.

```json
Request:  {"method":"storage.store","params":{"key":"rootpulse:commit:019e0315...","value":"..."}}
Response: {"status":"stored","key":"rootpulse:commit:019e0315...","family_id":"9b32f3a8"}
```

This phase works as specified. NestGate's `storage.store` API matches
the graph's expectations.

### Phase 5: Commit to Permanent History (LoamSpine) — FAIL

**Graph specifies**: `capability = "commit.session"` with `summary` and `content_hash`

**Actual API**: Multiple issues discovered.

1. **Method name mismatch**: `commit.session` doesn't exist as specified.
   The closest methods are:
   - `session.commit` — needs `spine_id` + `session_id` + `session_hash` + `committer`
   - `permanence.commit_session` — needs `spine_id` + `PermanentStorageDehydrationSummary` struct
   - `entry.append` — lowest-level, needs `spine_id` + `committer` + `entry_type` + `signature`

2. **Missing prerequisites**: All commit methods require a `spine_id`, which
   means `spine.create` must be called first. The graph doesn't include
   a spine creation step.

3. **Type mismatch**: `permanence.commit_session` expects a
   `PermanentStorageDehydrationSummary` struct, not a simple string summary.
   The struct requires `session_type` and other fields.

4. **`entry.append` requires cryptographic signature**: The `Entry` struct
   has a `signature` field that must be populated (from BearDog). But the
   graph puts signing in Phase 3 and commitment in Phase 5 — the signed
   data from Phase 3 is the merkle root, not the entry itself.

5. **Committer format**: The `committer` field expects a `Did` (e.g.,
   `did:key:z6Mk...`), which must come from BearDog's key management.
   The graph doesn't specify how to obtain the committer DID.

**Gap RP-2**: The `rootpulse_commit.toml` graph's Phase 5 cannot execute
against the current LoamSpine API. The method name, parameter structure,
and prerequisite steps don't match.

### Phase 6: Attribute (sweetGrass) — PARTIAL

**Graph specifies**: `capability = "provenance.create_braid"` with `commit_ref` and `agents`

**Actual API**: Method is `braid.create` (not `provenance.create_braid`), and
requires `data_hash`, `source`, `description`, `mime_type`, and `size`.

**Execution**:
```json
Request:  {"method":"braid.create","params":{
    "data_hash":"fb4ee7c9...","source":"rootpulse-test",
    "description":"RootPulse commit","mime_type":"application/json","size":64}}
Response: {
    "@id": "urn:braid:fb4ee7c9...",
    "was_attributed_to": "did:primal:55233215-...",
    "witness": {"algorithm":"ed25519","tier":"tower",...}
}
```

`braid.create` succeeds and returns a full W3C PROV-compatible braid with
Ed25519 witness signature. The braid is retrievable via `braid.get`.

**Gap RP-3**: Method name mismatch (`provenance.create_braid` vs `braid.create`)
and parameter mismatch (`commit_ref`/`agents` vs `data_hash`/`source`/
`description`/`mime_type`/`size`).

---

## Gap Summary

### RP-1: Graph→Primal Method Name and Parameter Mismatches

**Severity**: High (blocks graph execution)
**Owner**: biomeOS graph definitions + individual primal teams

| Phase | Graph Says | Primal Has | Issue |
|-------|-----------|------------|-------|
| 3 | `crypto.sign` with `data` | `crypto.sign` with `message` | Param name |
| 5 | `commit.session` with `summary`, `content_hash` | `session.commit`/`entry.append` with `spine_id`, `committer`, `signature`, etc. | Method + params |
| 6 | `provenance.create_braid` with `commit_ref`, `agents` | `braid.create` with `data_hash`, `source`, `mime_type`, `size` | Method + params |

**Fix needed**: Either update the graph to match current primal APIs, or add
canonical aliases in the primals that match the graph's expectations.

### RP-2: Missing Spine Lifecycle in Graph

**Severity**: High (Phase 5 prerequisite missing)
**Owner**: biomeOS graph + LoamSpine team

The graph jumps from "store content" to "commit to permanent history" without
creating a LoamSpine spine first. `spine.create` requires `name` and `owner`
parameters. This should be either:
- A Phase 0 setup step in the graph, or
- Automatic spine creation when `session.commit` is called, or
- A well-known default spine that the commit workflow uses

### RP-3: Graph Schema Lacks Type Information

**Severity**: Medium (blocks automated graph execution)
**Owner**: biomeOS graph engine

The `rootpulse_commit.toml` uses `${VARIABLE}` interpolation syntax, but:
- No documentation on what types these variables carry
- `dag.event.append` needs struct-variant enum types, not strings
- `entry.append` needs `[u8; 32]` byte arrays for hashes, not hex strings
- The graph engine must know how to serialize/deserialize between phases

### RP-4: biomeOS Graph Engine Not Testable Standalone

**Severity**: Medium (blocks end-to-end validation)
**Owner**: biomeOS team

There is no CLI tool to execute a graph TOML file against live primals.
`biomeos nucleus` coordinates primals, but there's no
`biomeos graph execute rootpulse_commit.toml --session-id=...` equivalent.
The graph files are specifications, not executable artifacts yet.

### RP-5: Entry Signing Lifecycle Unclear

**Severity**: Medium (blocks Phase 5 correctness)
**Owner**: LoamSpine + BearDog teams

LoamSpine's `Entry` struct requires a `signature` field, but the graph's
Phase 3 signs the dehydration summary (merkle root), not the LoamSpine entry.
The signing lifecycle needs clarification:
- Does BearDog sign the entry bytes? Or the merkle root?
- Does LoamSpine call BearDog internally, or does the graph orchestrate it?
- The graph assumes external orchestration, but LoamSpine's `entry.append`
  requires a pre-computed signature

---

## What Worked

| Component | Status | Significance |
|-----------|--------|-------------|
| rhizoCrypt DAG sessions | Fully functional | Create, append events, dehydrate, merkle root |
| BearDog Ed25519 signing | Fully functional | Signs arbitrary messages, returns public key |
| NestGate KV storage | Fully functional | Store and retrieve with family isolation |
| sweetGrass braid creation | Fully functional | W3C PROV-compatible braids with witnesses |
| LoamSpine spine management | Partially functional | Create + get work, entry append has param gaps |
| Health checks | All pass | All 6 primals alive and responsive |

---

## Recommendation

**The RootPulse commit workflow is 80% executable today.** Phases 1-4 and
Phase 6 work (with minor method name corrections). Phase 5 is the critical
gap — LoamSpine's commit API doesn't match the graph specification.

**Shortest path to end-to-end**:
1. Update `rootpulse_commit.toml` to match current primal method names
2. Add a `spine.create` Phase 0 step
3. Clarify the entry signing lifecycle between BearDog and LoamSpine
4. Add a `biomeos graph execute` CLI for standalone graph testing

**For primalSpring**: The graph TOML files in `biomeOS/graphs/` are design
documents, not tested integration contracts. A graph validation pass that
checks method names and parameter schemas against live primal capabilities
would catch these mismatches automatically.

---

## Action Summary

| Gap | Severity | Owner | Blocks |
|-----|----------|-------|--------|
| RP-1: Method/param name mismatches | High | biomeOS + primal teams | Graph execution |
| RP-2: Missing spine lifecycle | High | biomeOS + LoamSpine | Phase 5 |
| RP-3: Graph schema lacks types | Medium | biomeOS | Automated execution |
| RP-4: No standalone graph executor | Medium | biomeOS | End-to-end testing |
| RP-5: Entry signing lifecycle | Medium | LoamSpine + BearDog | Phase 5 correctness |
