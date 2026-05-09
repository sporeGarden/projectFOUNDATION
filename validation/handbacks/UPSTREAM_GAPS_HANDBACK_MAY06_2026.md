# Upstream Gaps — Handback from projectNUCLEUS Absorption

**Date**: 2026-05-06
**From**: projectNUCLEUS (ironGate)
**For**: primalSpring, primal teams

Gaps discovered during Phase 59 downstream absorption. Document here and
hand back via `primalSpring/docs/PRIMAL_GAPS.md`.

---

## Gap U1: primalSpring CHECKSUMS Stale After Phase 59 Refactoring

**Source**: primalSpring `validation/CHECKSUMS`
**Severity**: Medium (guidestone reports TAMPER on clean checkout)

7 of 18 checksums fail after the Phase 59 split of `composition/mod.rs`,
`deploy/mod.rs`, `coordination/mod.rs`, `btsp/mod.rs`, etc. into submodules.
The CHECKSUMS file was not regenerated after refactoring.

**Files affected**:
- `ecoPrimal/src/bin/primalspring_guidestone/main.rs`
- `ecoPrimal/src/composition/mod.rs`
- `ecoPrimal/src/validation/mod.rs`
- `ecoPrimal/src/tolerances/mod.rs`
- `ecoPrimal/src/coordination/mod.rs`
- `ecoPrimal/src/btsp/mod.rs`
- `ecoPrimal/src/deploy/mod.rs`

**Fix**: Regenerate `validation/CHECKSUMS` with `b3sum` against the current
source tree. Consider automating this in CI or a pre-commit hook.

**projectNUCLEUS note**: plasmidBin's `checksums.toml` may also be stale —
needs to be part of the absorption pipeline for deployments.

---

## Gap U2: Multi-Node Deploy Graphs Missing `by_capability` on Operation Nodes

**Source**: primalSpring `graphs/multi_node/`
**Severity**: Medium (guidestone structural failures)

Graphs that use operation-style `[[graph.nodes]]` (e.g., `mesh_init`,
`stun_discover`, `nat_detect`, `discover_peers`, etc.) set
`transport = "uds_only"` in `[graph.metadata]` but lack `by_capability`
on operation nodes. Guidestone flags this because UDS discovery requires
`by_capability` to find sockets.

**Affected graphs** (5 of 18 deploy graphs fail):
- `multi_node/basement_hpc_covalent.toml` — 10 operation nodes
- `multi_node/friend_remote_covalent.toml` — 11 operation nodes
- `multi_node/data_federation_cross_site.toml` — 8 operation nodes
- `multi_node/idle_compute_federation.toml` — 6 operation nodes
- `multi_node/three_node_covalent_cross_network.toml` — 12 operation nodes

**Fix options**:
1. Add `by_capability` to operation nodes pointing to the primal they RPC into
   (e.g., `mesh_init` → `by_capability = "discovery"` since it calls songbird)
2. Or change `transport` to `"uds_preferred"` (allows TCP fallback for operations)
3. Or make guidestone aware of operation-only nodes vs primal-spawn nodes

---

## Gap U3: Profile Graphs Missing `[graph.bonding_policy]`

**Source**: primalSpring `graphs/profiles/`
**Severity**: Low (guidestone structural failure, but profiles are templates)

8 profile graphs have nodes with `security_model = "btsp"` but no
`[graph.bonding_policy]` section. Guidestone treats this as a structural
inconsistency.

**Affected**: `full.toml`, `nest.toml`, `nest_viz.toml`, `node.toml`,
`node_ai.toml`, `nucleus.toml`, `tower_ai.toml`, `tower_viz.toml`

**Fix**: Add `bonding_policy = "btsp_required"` or `[graph.bonding_policy]`
sections to profile graphs.

---

## Gap U4: Capability Taxonomy Inconsistency (dag vs provenance)

**Source**: Cross-cutting
**Severity**: Low (both resolve to rhizoCrypt via routing.rs)

rhizoCrypt is sometimes referenced as `by_capability = "dag"` (fragments,
nucleus.toml) and sometimes as `by_capability = "provenance"`
(nucleus_complete.toml prior to fix). `composition/routing.rs`
`ALL_CAPS` lists both `dag` and `provenance` and maps both to rhizoCrypt,
so runtime routing works either way.

**Recommendation**: Standardize on `"dag"` (the primary capability domain)
and reserve `"provenance"` as an alias only. projectNUCLEUS has already
aligned to `"dag"`.

---

## Gap U5: sweetGrass TCP Port Confusion (39085 vs 9850)

**Source**: sweetGrass primal
**Severity**: Low (documented in known issues)

sweetGrass's BTSP TCP port is 9850 (canonical), but the HTTP endpoint
was historically on 39085. Integration scripts and docs referenced 39085.
Phase 59 declares 9850 as the Tier 5 fallback.

**Impact**: Any downstream that used 39085 for HTTP JSON-RPC to sweetGrass
needs updating. projectNUCLEUS has updated all references.

**Recommendation**: sweetGrass should bind its HTTP JSON-RPC endpoint on
the canonical 9850 port (or a well-known offset) rather than a dynamic port.

---

## Cross-Cutting Observations for Primal Teams

### Identity-Based Coupling in Primal Source

Every primal has some degree of identity-based coupling to siblings:
- rhizoCrypt has modules named `clients/beardog_http`, `nestgate_http`,
  `songbird`, `toadstool_http` — direct named references
- biomeOS uses `beardog`/`songbird` strings in binary layout and spawn
- squirrel has `beardog` security provider modules

**Evolution path**: Primals should discover capabilities at runtime via
Songbird `ipc.resolve` or biomeOS `capability.discover` rather than
hardcoding sibling names. The `by_capability` pattern in deploy graphs
should propagate into primal startup code.

### Hardcoded Port Ladder

The 9100-9900 port ladder appears in test fixtures and config defaults
across bearDog, songBird, squirrel, rhizoCrypt, biomeOS, petalTongue,
skunkBat, nestGate. These should be sourced from a single canonical
location (primalSpring `tolerances` or an env-driven config) rather than
scattered literals.

### unsafe Code Clusters

Heavy `unsafe` usage in coralReef (MMIO/VFIO/ioctl — expected for a GPU
driver) and toadStool (GPU, V4L2, plugin FFI). bearDog and skunkBat use
workspace-level `unsafe_code = "forbid"/"deny"` — good pattern for non-driver
primals to adopt.
