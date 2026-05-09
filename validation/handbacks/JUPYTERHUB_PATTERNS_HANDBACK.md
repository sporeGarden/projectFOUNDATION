# Upstream Gaps — JupyterHub Deployment Patterns and Sovereignty Wiring

**Date**: 2026-05-07
**From**: projectNUCLEUS (ironGate)
**For**: BearDog, biomeOS, NestGate, skunkBat teams, primalSpring
**Context**: The JupyterHub deployment on ironGate is the first real multi-user
service running through the full primal stack. Every friction point encountered
during deployment, user onboarding, and live testing reveals what the primals
must internalize as they replace external dependencies.

This document abstracts 8 patterns from the JupyterHub experience, maps each
to required primal capabilities, and identifies 5 new upstream gaps (including
1 critical security finding).

---

## Pattern 0 (CRITICAL): Convention Flags Are Not Enforcement

**Severity**: Critical (privilege escalation — fixed locally)
**Owner**: All primal teams, primalSpring (pattern-level)

Setting `NUCLEUS_READONLY=1` as an environment variable for reviewer/observer
tiers gave the appearance of read-only access while providing zero actual
enforcement. JupyterLab ignored this variable entirely — reviewer-tier accounts
(intended for external PIs with showcase-only visibility) could:

- Execute arbitrary Python/shell code as their Linux user
- Open terminals and interact with the system
- Call any primal JSON-RPC endpoint on localhost (all 13 primals bind
  0.0.0.0 at time of writing; **now 127.0.0.1 since Phase 60 PG-55, MethodGate enforced**)
- Read files outside their home directory if Linux permissions allowed

**Local fix applied**: The `pre_spawn_hook` now passes actual JupyterLab
server flags (`--ServerApp.terminals_enabled=False`,
`--KernelSpecManager.allowed_kernelspecs=set()`) that disable terminals
and kernel execution at the application level for observer/reviewer tiers.

**Upstream gap JH-0**: Every capability restriction must be enforced at the
gate, not signaled as a convention. This is the strongest argument for BearDog
ionic tokens carrying machine-readable capability scopes that every primal's
RPC dispatcher checks before executing a method. An environment variable is a
suggestion; a cryptographic capability token with an allowlist is enforcement.

This pattern — where a tier label is set but no gate checks it — will recur
everywhere that access control is delegated to convention rather than mechanism.
Every primal's JSON-RPC dispatch loop needs a pre-execution check:

```
fn dispatch(method, params, caller_token) -> Result {
    if !bearer_can_call(caller_token, method) {
        return Err(PermissionDenied)
    }
    // proceed with method execution
}
```

Without this, any composition that delegates access control to environment
variables or config flags is vulnerable to the same class of escalation.

---

## Pattern 1: Identity Is a System Primitive, Not an App Concern

**Severity**: High
**Owner**: BearDog team

JupyterHub uses PAM, which means identity = Linux user. We hit a
case-sensitivity bug (`ABGreviewer` vs `abgreviewer`) because PAM normalizes
usernames to lowercase but Linux group membership stores them as-created.
JupyterHub's `_abg_users()` function reads groups at startup and caches the
result — adding a new user required `systemctl restart jupyterhub`.

**Upstream gap JH-1**: Identity must live in BearDog as a first-class concept.
The BTSP ionic token design (Step 2b in `TUNNEL_EVOLUTION.md`) needs to solve:

- **Canonical identity format** — DIDs (`did:key:z6Mk...`) are case-sensitive
  by design, which is correct. The mapping from human-readable name to DID must
  be consistent and case-insensitive at the lookup layer.
- **Dynamic membership** — Token verification must be live (per-request RPC),
  not cached at service startup.
- **Tier as capability scope** — PAM groups are a crude capability system. BTSP
  ionic tokens should encode capabilities directly (which methods a user can
  call), not a tier label that gets interpreted downstream.

**Calibration**: PAM + Linux groups prove the access model works. BearDog
identity management is the sovereign replacement.

---

## Pattern 2: Resource Scoping Requires a Composition-Aware Spawner

**Severity**: High
**Owner**: biomeOS team, ToadStool team

The `pre_spawn_hook` sets `mem_limit`, `cpu_limit`, and environment variables
per tier. This is JupyterHub-specific Python glue. The pattern generalizes:

- ToadStool already has workload scoping (containers get resource limits).
  The JupyterHub spawner is doing the same thing for notebook processes.
- What's missing: a unified resource policy that BearDog's ionic token carries
  and ToadStool/biomeOS enforces. The token says "8 cores, 32G, methods X/Y/Z"
  and every primal in the composition respects it.
- This is the `neuralAPI` pattern — biomeOS orchestrates, but the resource
  envelope comes from the identity token, not from a Python hook.

**Upstream gap JH-2**: Ionic tokens need a resource envelope field that
composition-aware spawners (ToadStool, biomeOS) read and enforce.

**Calibration**: JupyterHub's `TIER_LIMITS` dict and `pre_spawn_hook` prove
the tier-to-resource mapping works. The sovereign version carries this in
the token itself.

---

## Pattern 3: Shared Workspace Is a Content-Addressed Problem

**Severity**: Medium
**Owner**: NestGate team

Users share data through symlinks to `/home/irongate/shared/abg/`. Reviewers
see only `showcase/`, compute users see everything. This is filesystem-level
access control via symlink targets.

- NestGate should own this — shared workspace = named collections. "showcase"
  is a collection. "commons" is a collection. Access control is per-collection
  via BearDog capabilities.
- **Gap NG-2 (Collection/Manifest)** directly blocks this. Without named
  collections, we can't express "reviewer can read collection `showcase` but
  not collection `commons`."
- The symlink pattern is the calibration — it proves the access model works.
  NestGate collections with per-collection ACL are the replacement.

**Cross-reference**: `NESTGATE_CONTENT_GAPS_HANDBACK.md` Gap NG-2.

---

## Pattern 4: Tunnel Auth Layering Reveals Trust Boundaries

**Severity**: Medium
**Owner**: BearDog team, Songbird team

The `_post_auth_hook` blocks `irongate` from tunnel login by inspecting
Cloudflare headers (`Cf-Connecting-Ip`, `X-Forwarded-For`). This is trust
boundary enforcement: local admin cannot be impersonated from outside.

- BTSP solves this natively — a BTSP channel knows its origin
  cryptographically, not by inspecting HTTP headers that a proxy sets.
- When Cloudflare is removed (Step 3b+), this header-based check disappears.
  BearDog TLS must provide an equivalent signal: "this connection is local"
  vs "this connection arrived from the internet."
- Songbird's BirdSong multicast already distinguishes local (USB-C link)
  from remote. This distinction needs to propagate to the auth layer.

**No new gap**: This is already implicit in the Step 3b design in
`TUNNEL_EVOLUTION.md`, but the JupyterHub deployment makes the requirement
concrete — the trust boundary signal must be available to application-level
auth hooks, not just transport-level code.

---

## Pattern 5: Service Persistence Is Solved, Process Lifecycle Is Not

**Severity**: Medium
**Owner**: biomeOS team

All three services (primals, JupyterHub, cloudflared) run as systemd units and
survive reboots. But:

- User list changes require restart — JupyterHub caches `allowed_users` at
  startup. Adding a user means `systemctl restart jupyterhub`.
- Primal updates require full composition restart via `deploy.sh`. There's
  no rolling update for a single primal.
- biomeOS should own lifecycle — the graph engine needs a
  `composition.reload` capability that hot-swaps a single primal without
  taking down the composition.

**Upstream gap JH-3**: biomeOS needs `composition.reload` (or equivalent) for
rolling primal updates without full composition restart.

**Calibration**: systemd `restart` proves the persistence model. The sovereign
version is biomeOS-managed lifecycle with per-primal hot-swap.

---

## Pattern 6: The Credential Anti-Pattern

**Severity**: Medium
**Owner**: BearDog team, primalSpring (UX)

We set passwords manually (`echo 'user:pass' | chpasswd`). This is the
anti-pattern that BTSP ionic tokens eliminate. But it exposed the real
workflow that the sovereign system must support:

1. Admin creates identity (BearDog `identity.create`)
2. Admin assigns capabilities (BearDog `auth.issue_ionic` with tier/scope)
3. Token delivered to user (out-of-band: in-person, secure message)
4. User presents token at login (replaces username/password)
5. Token is verified live against BearDog (not cached)
6. Token scopes what the user can do (not just "allowed in")

Steps 1-2 are the `abg_accounts.sh` equivalent in primal-native form.
Step 3 is the unsolved UX problem — how does a non-technical ABG member
receive and use their token?

**Upstream gap JH-4**: End-to-end token issuance UX for non-technical users.
BearDog needs a token delivery mechanism that doesn't require CLI skills.

---

## Pattern 7: Observability Requires a Consistent Event Surface

**Severity**: Medium
**Owner**: skunkBat team

We diagnosed the `ABGreviewer` login failure from `journalctl` logs. JupyterHub
logs to systemd journal. Primals log to their own files. Cloudflared logs
separately. There is no unified view.

- skunkBat should aggregate events from all sources (systemd journal, primal
  logs, tunnel logs) into a single queryable surface.
- The JupyterHub auth hook logs are calibration data — "Blocked tunnel login
  for local-only user irongate" is a security event that should flow through
  rhizoCrypt's DAG for provenance.
- When JupyterHub auth is replaced with BTSP, every auth event (success,
  failure, blocked) should be a rhizoCrypt event with a sweetGrass
  provenance braid.

**Upstream gap JH-5**: skunkBat needs a log aggregation capability that
consumes heterogeneous sources (systemd journal, primal JSON-RPC logs,
application logs) and feeds security-relevant events into the provenance
pipeline (rhizoCrypt DAG + sweetGrass braid).

---

## Gap Summary

| Gap | Pattern | Severity | Owner | Blocks |
|-----|---------|----------|-------|--------|
| JH-0: RPC dispatcher capability check | 0 (Enforcement) | **Critical** | All primal teams | Secure multi-user compositions |
| JH-1: BearDog identity management | 1 (Identity) | High | BearDog | Step 2b (BTSP auth) |
| JH-2: Token-carried resource envelope | 2 (Resource scope) | High | biomeOS + ToadStool | neuralAPI enforcement |
| JH-3: Composition hot-reload | 5 (Lifecycle) | Medium | biomeOS | Rolling primal updates |
| JH-4: Token issuance UX | 6 (Credentials) | Medium | BearDog + primalSpring | Non-technical user onboarding |
| JH-5: Log aggregation + provenance | 7 (Observability) | Medium | skunkBat | Unified security monitoring |

Patterns 3 and 4 map to existing documented gaps (NG-2 and Step 3b
respectively) and do not introduce new gap IDs.

---

## Sovereignty Wiring — How Patterns Chain Into Replacement Steps

Each row shows the current JupyterHub mechanism, what it calibrates, and
the sovereign primal replacement:

| Current Mechanism | Pattern | Calibrates | Sovereign Replacement | Step |
|-------------------|---------|------------|----------------------|------|
| `NUCLEUS_READONLY` env var (broken) | 0 | Nothing (false security) | RPC dispatcher capability check via ionic token | Pre-Step 2b |
| PAM + Linux groups | 1 | Identity + membership | BearDog identity + ionic tokens | Step 2b |
| `pre_spawn_hook` resource limits | 2 | Tier-to-resource mapping | Token-carried resource envelope in neuralAPI | Step 2b+ |
| Filesystem symlinks to `shared/abg/` | 3 | Collection-based access | NestGate named collections + per-collection ACL | Step 3a |
| CF header inspection (`post_auth_hook`) | 4 | Local vs remote trust boundary | BTSP connection origin signal | Step 3b |
| `systemctl restart` for user changes | 5 | Service lifecycle | biomeOS `composition.reload` | Post-Step 2b |
| `echo 'user:pass' \| chpasswd` | 6 | Credential distribution | BearDog token issuance UX | Step 2b |
| `journalctl` across services | 7 | Event diagnosis | skunkBat log aggregation + provenance | Post-Step 2b |

The sovereignty path is sequential: Patterns 0-1-6 must be solved before
Step 2b (BTSP auth replaces PAM). Pattern 2 follows immediately. Patterns
3-4 align with Steps 3a-3b. Patterns 5 and 7 are parallel improvements
that can happen at any point.

---

## Positive Findings

| Finding | Significance |
|---------|-------------|
| PAM tiered access model works | Validates the capability-scope concept for ionic tokens |
| `pre_spawn_hook` resource limits work | Validates the token-carried resource envelope concept |
| Symlink-based shared workspace works | Validates the NestGate collection access model |
| CF header trust boundary works | Validates the BTSP connection origin concept |
| systemd persistence survives reboots | Validates the deployment model (to be absorbed by biomeOS) |
| 13 primals stable under multi-user load | Composition is production-ready for the current tier |
| Security fix (Pattern 0) was 2 lines | JupyterLab has the right enforcement hooks; the gap was in wiring |

---

## Relationship to Other Handbacks

| Document | Relationship |
|----------|-------------|
| `PETALTONGUE_GAPS_HANDBACK.md` | Pattern 3 depends on PT-1 (catch-all route) for web-served collections |
| `NESTGATE_CONTENT_GAPS_HANDBACK.md` | Pattern 3 depends on NG-2 (collections) for access-scoped sharing |
| `ROOTPULSE_GAPS_HANDBACK.md` | Pattern 0 (enforcement) applies to RootPulse graph execution — graph-dispatched RPC calls must carry capability tokens |
| `TUNNEL_EVOLUTION.md` | Patterns 0-1-6 are prerequisites for Step 2b; Patterns 3-4 for Steps 3a-3b |
| `SOVEREIGNTY_VALIDATION_PROTOCOL.md` | Each pattern replacement follows the calibrate-replace-validate protocol |

---

## Action Summary for Upstream Teams

### BearDog Team
- **JH-0 (Critical)**: Define the RPC dispatcher capability check pattern that all primals implement
- **JH-1 (High)**: `identity.create`, `auth.issue_ionic`, `auth.verify_ionic` methods
- **JH-4 (Medium)**: Token delivery mechanism for non-technical users

### biomeOS Team
- **JH-2 (High)**: Token-carried resource envelope enforcement in neuralAPI
- **JH-3 (Medium)**: `composition.reload` for rolling primal updates

### skunkBat Team
- **JH-5 (Medium)**: Log aggregation across heterogeneous sources with provenance pipeline integration

### primalSpring
- **JH-0 (Critical)**: Codify the "enforcement at the gate" pattern as an ecosystem standard — every primal's RPC dispatcher must check capability tokens, not trust environment variables or config conventions
- Review all existing primals for convention-based access control that lacks enforcement

### NestGate Team
- Cross-reference **NG-2** — the collection/manifest gap is now validated as a real deployment need by Pattern 3

---

## Document History

| Date | Change |
|------|--------|
| 2026-05-07 | Initial handback. 8 patterns, 5 new gaps (1 critical). |
