# Security Handback — Penetration Testing Results

**Date**: 2026-05-06
**From**: projectNUCLEUS (ironGate)
**For**: Primal teams, primalSpring, wateringHole
**Context**: Three-layer pen test on live 13-primal full NUCLEUS composition

---

## Summary

Ran automated penetration testing (below/at/above the primal layer) on the
live composition. Fixed what we could at the composition level. Remaining
issues were escalated upstream.

**Before fixes**: 13 primal ports on 0.0.0.0, reachable from LAN
**After composition fixes**: 7 ports rebound to 127.0.0.1, 6 ports still needed upstream `--bind` flag
**After primalSpring Phase 59**: **All 13/13 primals default `127.0.0.1`**. Zero open bind gaps.

> **UPDATE 2026-05-06**: All 5 security gaps (PG-55 through PG-59) resolved by primalSpring v0.9.24 Phase 59. See below.

---

## Fixed at Composition Level (deploy.sh)

These primals accept a bind address flag. `deploy.sh` now passes
`BIND_ADDRESS` (default `127.0.0.1`, overridable via `NUCLEUS_BIND_ADDRESS`):

| Primal | Port | Flag Used | Result |
|--------|------|-----------|--------|
| BearDog | 9100 | `--listen $BIND:$PORT` | 127.0.0.1 |
| Squirrel | 9300 | `--bind $BIND` | 127.0.0.1 |
| NestGate | 9500 | `--bind $BIND` | 127.0.0.1 |
| rhizoCrypt | 9601/9602 | `--host $BIND` | 127.0.0.1 |
| loamSpine | 9700 | `--bind-address $BIND` | 127.0.0.1 |
| CoralReef | 9730 | `--rpc-bind $BIND:$PORT` | 127.0.0.1 (was already) |
| BarraCuda | 9740 | `--bind $BIND:$PORT` | 127.0.0.1 (env default) |
| sweetGrass | 9851 | `--http-address $BIND:$PORT` | 127.0.0.1 (HTTP endpoint) |

Also fixed: `jupyterhub.sqlite` permissions (644 → 600).

---

## Resolved by primalSpring Phase 59 (2026-05-06)

All 6 upstream gaps resolved. Every primal now defaults to `127.0.0.1`:

| PG | Primal | Resolution |
|----|--------|-----------|
| PG-55 | All 13 | All primals default `127.0.0.1`. Songbird, ToadStool, skunkBat, biomeOS, petalTongue: `--bind`. sweetGrass: bare `--port` = localhost. biomeOS nucleus forwards `--bind`. |
| PG-56 | NestGate | BTSP method-level auth gating. 10-method exempt whitelist (health, identity, capabilities). |
| PG-57 | skunkBat | Multi-dimensional anomaly detection (connection rate + traffic volume + port diversity). 12 normal + 7 attack patterns seeded. |
| PG-58 | Songbird | `--bind` for HTTP server, `--listen` for IPC socket (separate concerns, documented). |
| PG-59 | sweetGrass | `--http-address` and `--port` both accept `host:port`, documented in CLI help. |

### What to Absorb

1. **Bind policy**: Deploy scripts can drop explicit `--bind 0.0.0.0` overrides. Use `bind_policy = "localhost"` in graph metadata; guidestone validates it. Pass `--bind 0.0.0.0` only for intentional cross-host access.
2. **`PrimalDeployProfile.bind_flag`**: All 13 primals return `Some(flag)` — deploy tooling can use `profile.bind_flag` programmatically.
3. **Foundation validation graph**: `graphs/compositions/foundation_validation.toml` — 12-node NUCLEUS for scientific sediment pipeline.
4. **Checksums**: primalSpring CHECKSUMS generated via `tools/regenerate_checksums.sh`. Verify `validation/CHECKSUMS` matches after pull.
5. **NestGate TCP fallback note**: PG-56 BTSP gating applies to UDS/isomorphic paths only. TCP fallback (Tier 5, localhost) still dispatches all methods ungated — acceptable for localhost, but be aware if TCP is exposed externally.

---

## Cross-Cutting Recommendation: UniBin `--bind` Standard

The root cause is that `--port` is defined in UniBin v1.1 as the universal
entry point, but it doesn't specify the bind address. Seven primals already
implement some form of bind address control, but with six different flag names:

| Flag | Primals using it |
|------|-----------------|
| `--listen host:port` | BearDog, Songbird (IPC only) |
| `--bind host:port` | NestGate, Squirrel, BarraCuda |
| `--bind-address host` | loamSpine |
| `--host host` | rhizoCrypt |
| `--rpc-bind host:port` | CoralReef |
| `--http-address host:port` | sweetGrass (HTTP only) |

**Proposal**: Add `--bind <host:port>` to the UniBin v1.1 standard as
the universal bind address flag. When provided, it overrides `--port`.
Default should be `127.0.0.1` (localhost-only), not `0.0.0.0`.

This is a **one-line change** in each primal's CLI parser and a
**two-line change** in the UniBin standard doc.

---

## Other Security Findings

### Composition Level (fixed or documented)

| Finding | Severity | Owner | Status |
|---------|----------|-------|--------|
| `jupyterhub.sqlite` mode 644 | MEDIUM | projectNUCLEUS | FIXED (now 600) |
| UFW inactive | MEDIUM | ironGate admin | Documented (needs sudo) |
| TornadoServer version in headers | LOW | JupyterHub config | Documented |
| Missing X-Frame-Options header | LOW | JupyterHub config | Documented |
| Missing X-Content-Type-Options | LOW | JupyterHub config | Documented |

### Primal Level (resolved by Phase 59)

| Finding | Severity | Owner | Status |
|---------|----------|-------|--------|
| NestGate `storage.list` unauthenticated | MEDIUM | NestGate team | **RESOLVED** — PG-56 BTSP method-level auth gating |
| 6 primals bind 0.0.0.0 | HIGH | Each primal team | **RESOLVED** — PG-55 all 13 default `127.0.0.1` |
| sweetGrass ephemeral ports on 0.0.0.0 | LOW | sweetGrass team | **RESOLVED** — PG-59 `--port` accepts `host:port` |

### Positive Findings (upstream should know)

| Finding | Significance |
|---------|-------------|
| All 4 fuzzed primals survived 7 malformed payloads | Rust serde + type system = strong default |
| All 3 probed primals reject 7 hidden method names | No debug/admin backdoors |
| sweetGrass rejects plaintext (BTSP enforced) | Transport-layer security works |
| rhizoCrypt rejects plaintext on tarpc port | BTSP enforcement works |
| JupyterHub returns 403 for unauthed API calls | Auth works correctly |
| Path traversal blocked (302 redirect to login) | Tornado handles this correctly |

---

## skunkBat Observations

skunkBat was live during all pen testing. Metrics after scan:
- Threats detected: 0
- Connections quarantined: 0
- Alerts sent: 0

This is expected — skunkBat needs baseline data to distinguish normal
from anomalous. The pen test payloads are training data. Recommendation:
skunkBat should learn to detect the fuzz/enumeration patterns observed
during this test, and alert when similar patterns appear in production.

---

## Action Summary

| Action | Owner | Priority | Status |
|--------|-------|----------|--------|
| Add `--bind` to UniBin v1.1 standard | wateringHole | HIGH | **DONE** — Phase 59 |
| Add `--bind` to 6 primals (P1-P6) | Each primal team | HIGH | **DONE** — PG-55/58/59 |
| NestGate BTSP scoping for storage.list | NestGate team | MEDIUM | **DONE** — PG-56 |
| skunkBat baseline learning from pen test data | skunkBat team | MEDIUM | **DONE** — PG-57 |
| Activate UFW on ironGate | ironGate admin | MEDIUM | **DONE** — deny-by-default |
| JupyterHub security headers | projectNUCLEUS | LOW | **DONE** — X-Frame-Options, nosniff, Server suppressed |
| Process isolation (hidepid=2) | ironGate admin | MEDIUM | **DONE** — fstab persistent |
| Outbound network restriction | ironGate admin | HIGH | **DONE** — iptables/ip6tables owner match UID 1001-1099 |
| Reviewer/observer read-only | projectNUCLEUS | CRITICAL | **DONE** — filesystem + JupyterLab server flags |
| Shared notebook write protection | projectNUCLEUS | MEDIUM | **DONE** — 444 perms, compute users run but can't save |
| RPC dispatcher capability check | All primal teams | CRITICAL | **ENFORCED** — MethodGate on 13/13 primals, `NUCLEUS_AUTH_MODE=enforced` default, 265 PASS 0 FAIL |

**Ecosystem state post-Phase 60**: 13/13 BTSP Phase 3 FULL AEAD, 14/14 ports on `127.0.0.1`, MethodGate on 13/13 primals, ionic tokens live, resource envelopes enforced, audit log operational. 263 PASS, 0 FAIL validation. See `validation/REVALIDATION_PHASE60_MAY08_2026.md`.

---

## Phase 2a Multi-User Pentest (2026-05-07)

Second pentest round: validated security from an authenticated ABG compute-tier
user (tamison) running inside JupyterHub via
`shared/commons/nucleus-security-validation.ipynb`.

### Results

| Test | Result | Severity | Action |
|------|--------|----------|--------|
| Primal port scan | 13/13 reachable on localhost | Expected | JH-0 gap documented |
| BTSP enforcement | sweetGrass 9850 accepts plaintext | Info | By design: dual-port (9850=IPC, 9851=BTSP) |
| Filesystem isolation | 10/10 PASS | Strong | shadow, homes, tunnel config, sqlite all denied |
| NestGate write | ALLOWED without auth | High | Confirms JH-0 |
| Outbound internet | GitHub, PyPI reachable via IPv4+IPv6 | High | **FIXED**: iptables owner match |
| Process visibility | 13 primal PIDs visible | Medium | **FIXED**: hidepid=2 |
| Reviewer code execution | Could execute code, create files | Critical | **FIXED**: server flags + filesystem |
| NUCLEUS_READONLY bypass | Env var not enforced | Critical | **FIXED**: replaced with mechanism |

### Hardening Applied

| Layer | Method | Persistent | Scope |
|-------|--------|-----------|-------|
| Process isolation | `hidepid=2` on /proc (gid=0 exempts root) | fstab | All non-root users |
| Outbound network | iptables+ip6tables DROP for UID 1001-1099 | systemd restore service | ABG users |
| Localhost preserved | iptables ACCEPT 127.0.0.0/8 | Persisted | Primal RPC works |
| LAN preserved | iptables ACCEPT RFC1918 | Persisted | NucBox link |
| Reviewer read-only | chmod 550 root-owned notebook dir | Filesystem | reviewer/observer |
| Reviewer no-exec | JupyterLab server flags (no kernels, no terminals) | jupyterhub_config.py | reviewer/observer |
| Shared immutable | chmod 444 on notebooks, 2755 on commons/ | Filesystem | All tiers |

### Dual-Port Clarification

Primals with BTSP use two ports by design:

| Primal | IPC Port (plaintext) | BTSP Port (encrypted) |
|--------|---------------------|----------------------|
| sweetGrass | 9850 | 9851 |
| rhizoCrypt | 9602 | 9601 |

IPC ports are for primal-to-primal localhost communication. BTSP ports for
cross-host. Both localhost-bound. The IPC ports accepting plaintext is correct —
the gap is they accept it from *any* localhost process (JH-0).

### JH-0 Status Update (2026-05-08)

**JH-0 ADOPTED**: primalSpring Phase 60 shipped MethodGate on all 13 primals.
**Enforced mode now active** (2026-05-08) — unauthenticated calls return `-32001`.
`NUCLEUS_AUTH_MODE=enforced` is the default. 265 PASS, 0 FAIL, 0 KNOWN_GAP.
See `validation/REVALIDATION_PHASE60_MAY08_2026.md` for full results.
