# Evolution Gaps — projectNUCLEUS

Living tracker of remaining gaps across three horizons. Updated as gaps
close and new ones emerge. Each gap is local — actionable by projectNUCLEUS
without waiting on upstream unless noted.

**Last updated**: 2026-05-09 (open observer, pappusCast, tunnelKeeper, multi-tier tests)
**Validation baseline**: 267 PASS, 0 FAIL, 0 KNOWN_GAP (bash 5-layer)
**Rust validator**: 175 PASS, 0 FAIL, 6 DARK_FOREST (`validation/darkforest/` v0.2.0 — 939KB, authoritative)
**Multi-tier tests**: observer + reviewer + compute + hub + pappusCast health (`deploy/tier_test_all.sh`)

Related specs:
- [TUNNEL_EVOLUTION.md](TUNNEL_EVOLUTION.md) — sovereignty replacement roadmap
- [SECURITY_VALIDATION.md](SECURITY_VALIDATION.md) — five-layer validation model
- [SOVEREIGNTY_VALIDATION_PROTOCOL.md](SOVEREIGNTY_VALIDATION_PROTOCOL.md) — replacement methodology
- [COMPLETE_DEPENDENCY_INVENTORY.md](COMPLETE_DEPENDENCY_INVENTORY.md) — full dependency map

**Rust evolution**: `validation/darkforest/` v0.2.0 — modular auditable security framework (939KB,
zero runtime deps). 7 source modules: `check.rs` (structured types), `net.rs` (TCP/HTTP helpers),
`pentest.rs` (3 threat actors), `fuzz.rs` (13 primals + JupyterHub), `crypto.rs` (13 crypto
strength checks), `report.rs` (pipe + JSON output), `main.rs` (CLI + runner). Backward-compatible
`PASS|FAIL|KNOWN_GAP|DARK_FOREST` pipe format, plus `--output json` for auditable structured reports
with per-check severity, category, evidence, remediation, and timestamps.

---

## Horizon 1: External Security & ABG Hosting

Hardening what's live now — JupyterHub, Cloudflare tunnel, PAM auth,
systemd services. No primal evolution required. All local work.

### Open

None. All Horizon 1 gaps resolved.

### Resolved (fossil record)

| ID | What | When | How |
|----|------|------|-----|
| JH-0 | Unauthenticated RPC | 2026-05-08 | MethodGate enforced, 10/13 confirmed TCP |
| JH-1 | No caller identity | 2026-05-08 | BearDog ionic tokens live |
| JH-2 | No resource limits | 2026-05-08 | biomeOS + ToadStool enforce envelopes |
| JH-3 | Full restart required | 2026-05-08 | biomeOS `composition.reload` |
| JH-4 | Session UX | 2026-05-08 | `auth.issue_session` with presets |
| JH-5 | No audit log | 2026-05-08 | skunkBat ring buffer, Phase 2 |
| JH-8 | DNS exfil open | 2026-05-08 | iptables DNS restricted to local resolver |
| JH-9 | Conda envs group-writable | 2026-05-08 | Root-owned, 755 |
| DF-1 | 5 primals bind 0.0.0.0 | 2026-05-08 | Phase 60 PG-55 default binding |
| H1-01 | `/hub/api/` version disclosure | 2026-05-08 | Headers suppressed; body is upstream built-in (JH-10). Accepted risk — localhost only |
| H1-02 | Voila executes as hub user | 2026-05-08 | Dedicated `voila` system user (UID 998). `user: 'voila'` in JupyterHub service config |
| H1-03 | rustdesk on 0.0.0.0 | 2026-05-08 | Already binds LAN IP (192.168.1.238), not 0.0.0.0. No action needed |
| H1-06 | Cookie secret rotation | 2026-05-08 | `deploy/rotate_cookie_secret.sh` created. Manual rotation — run monthly or after events |
| H1-07 | Baseline 7-day summary | 2026-05-08 | Cron capturing hourly (deduplicated). Run `summarize_baselines.sh` after May 14 |
| H1-04 | systemd enumeration | 2026-05-08 | `setfacl` deny execute on `/usr/bin/systemctl` for abg-compute/reviewer/observer groups |
| H1-05 | Reviewer python3 access | 2026-05-08 | `setfacl` deny execute on `/usr/bin/python3` for abg-reviewer/observer. Compute and admin unaffected |

---

## Horizon 2: Sovereignty — Replacing External Services

Progressive elimination of Cloudflare, GitHub, and PAM. Each step follows
the calibrate → shadow-run → cutover protocol in `SOVEREIGNTY_VALIDATION_PROTOCOL.md`.

### Step 2b: BTSP Auth (replaces PAM passwords)

**Status**: Ready to build. All primal prerequisites resolved.

| ID | Gap | Effort | Notes |
|----|-----|--------|-------|
| H2-01 | Build `jupyterhub_btsp_auth.py` authenticator plugin | Medium | Skeleton in TUNNEL_EVOLUTION.md. Calls `beardog.auth.verify_ionic`. PAM fallback during shadow run |
| H2-02 | Token distribution UX | Low | CLI: `auth.issue_session(purpose="jupyterhub")` → user pastes token as password. No web UI needed yet |
| H2-03 | Dual-auth shadow run (7 days) | Ops | PAM + BTSP both accepted. Log which auth each login uses. Compare success rates and latency |
| H2-04 | PAM cutover criteria | — | BTSP auth success rate ≥ 99.9%, latency overhead < 50ms, 7-day shadow run clean |

**Not blocked by JH-11**: The authenticator only calls beardog (same-primal),
not cross-primal methods. Cross-primal federation is Horizon 3.

### Step 3a: sporePrint on NestGate (replaces GitHub Pages)

**Status**: Primal prerequisites resolved (Phase 60). Local pipeline not built.

| ID | Gap | Effort | Notes |
|----|-----|--------|-------|
| H2-05 | Build `deploy/publish_sporeprint.sh` | Medium | Zola build → NestGate `content.put` for each file → manifest creation |
| H2-06 | Configure petalTongue web mode | Low | `--docroot` resolved Phase 60. Config: listen 9901, NestGate backend, sporeprint collection |
| H2-07 | Cloudflare ingress route for `primals.eco` → petalTongue:9901 | Trivial | Add route to cloudflared config |
| H2-08 | Shadow run: GitHub Pages vs NestGate/petalTongue (7 days) | Ops | benchScale `nestgate_content_parity.sh`. Compare TTFB, Lighthouse scores |
| H2-09 | Cutover: disable GitHub Pages, petalTongue primary | — | 100% content parity, TTFB within 10% of GH Pages |

### Step 3b: BearDog TLS (replaces Cloudflare TLS)

**Status**: Blocked on BearDog X.509/ACME support.

| ID | Gap | Effort | Notes |
|----|-----|--------|-------|
| H2-10 | BearDog TLS listener with X.509 + ACME | High | Upstream: BearDog needs Let's Encrypt client, SNI routing, cert auto-renewal |
| H2-11 | BearDog rate limiting | Medium | Upstream: connection + request rate limiting to replace Cloudflare DDoS protection |
| H2-12 | Shadow run on port 8443 alongside Cloudflare 443 | Ops | benchScale `btsp_tls_parity.sh` |

### Step 3c: Songbird NAT (replaces cloudflared)

**Status**: Blocked on Songbird STUN/TURN production hardening.

| ID | Gap | Effort | Notes |
|----|-----|--------|-------|
| H2-13 | Songbird STUN client for NAT punch-through | High | Upstream: Songbird team |
| H2-14 | Self-hosted STUN/TURN VPS relay | Medium | ~$5/mo Hetzner/OVH, BearDog key-authenticated |
| H2-15 | Dynamic DNS update mechanism | Low | Songbird notifies DNS when IP changes |
| H2-16 | Connection fallback chain (direct → STUN → TURN → cloudflared emergency) | Medium | Local |

### Step 4: Sovereign DNS (replaces Cloudflare NS)

**Status**: Specified, not started.

| ID | Gap | Effort | Notes |
|----|-----|--------|-------|
| H2-17 | `knot-dns` authoritative on VPS | Medium | DNSSEC, monitoring via skunkBat |
| H2-18 | NS transfer from Cloudflare registrar | Ops | Update registrar to point NS at VPS |
| H2-19 | BTSP direct resolution for ecosystem clients | Low | Bypass DNS for BTSP-aware tools |

---

## Horizon 3: Primal-Only (zero external dependencies)

Long-term evolution. Each item replaces an external service with a primal
composition. Not actionable until Horizon 2 steps validate the patterns.

| ID | External Service | Primal Replacement | Status | Blocks |
|----|-----------------|-------------------|--------|--------|
| H3-01 | JupyterHub (notebook UX) | petalTongue dashboards + biomeOS dispatch | Gap — no notebook execution in petalTongue | Horizon 2 complete |
| H3-02 | GitHub Releases (plasmidBin) | NestGate blob storage + manifest queries | Gap — `fetch.sh` hardcodes GitHub URLs | Step 3a (NestGate content pipeline) |
| H3-03 | GitHub Actions (CI/CD) | Forgejo Actions or self-hosted runners | Gap — 74 workflow files to port | Forgejo primary adoption |
| H3-04 | GitHub repos (source hosting) | Forgejo primary, GitHub mirror | Calibration — Forgejo installed, not primary | Forgejo Actions working |
| H3-05 | Docker Hub / ghcr.io | NestGate OCI blob store | Gap — ToadStool config-swappable | NestGate content pipeline |
| H3-06 | Anthropic / OpenAI | Ollama + barraCuda WGSL inference | Partial — Ollama works locally | barraCuda shader maturity |
| H3-07 | JH-11 cross-primal token federation | biomeOS composition forwarding with `_resource_envelope` | Gap — upstream architectural decision | biomeOS federation design |
| H3-08 | JH-5 cross-primal audit forwarding | skunkBat → rhizoCrypt DAG + sweetGrass braids | Gap — local ring buffer only | JH-11 federation path |
| H3-09 | conda/pip/crates.io | Vendored deps, private registry | Low priority — offline modes exist | Not blocking |
| H3-10 | NCBI / UniProt / KEGG | Local mirror + `abg_data.sh` provenance | Partial — data registry operational | Not blocking (data, not service) |

### Irreducible Externals (never sovereign)

These are not gaps — they are accepted constraints:

- Domain registrar (`primals.eco`)
- Linux kernel / systemd
- NVIDIA GPU drivers
- Let's Encrypt / ACME (browser trust chain)
- VPS for STUN relay (~$5/mo, commodity)

---

## Upstream Dependencies (waiting on primal teams)

Tracked here for visibility. Handed off via wateringHole. Not blocking
Horizon 1 or local Horizon 2 work.

| ID | What | Owner | Handoff |
|----|------|-------|---------|
| DF-2 | toadstool `TOADSTOOL_AUTH_MODE` env var mapping | toadStool team | `PROJECTNUCLEUS_UPSTREAM_GAPS_CONSOLIDATED_MAY08_2026.md` |
| DF-3 | songbird/squirrel silent on `auth.mode` TCP | Each team | Same |
| U1 | primalSpring `CHECKSUMS` stale | primalSpring | Same |
| U2 | 5 deploy graphs missing `by_capability` | primalSpring | Same |
| U3 | 8 profile graphs missing `bonding_policy` | primalSpring | Same |
| U5 | sweetGrass port 39085 vs 9850 | sweetGrass | Same |
| JH-11 | Cross-primal token federation | biomeOS/primalSpring | Same (Tier 4 — flagged, not actionable) |

---

## Scoring

```
Horizon 1 (external security):    ██████████  COMPLETE — 7/7 resolved, 175 PASS 0 FAIL 6 DARK_FOREST (darkforest v0.2.0)
Horizon 2 (sovereignty):          ██░░░░░░░░  Step 2a done, 2b ready, 3a close
Horizon 3 (primal-only):          █░░░░░░░░░  10 items, all blocked on H2
Upstream (waiting):                ████████░░  7 items handed off, not blocking
```

---

## Changelog

| Date | Change |
|------|--------|
| 2026-05-08 | Initial spec. Phase 60 enforced. 3 horizons, 37 gaps tracked. |
| 2026-05-08 | H1-01→H1-03, H1-06, H1-07 resolved. Voila isolated (UID 998). Cron deduplicated. |
| 2026-05-08 | H1-04, H1-05 resolved. systemctl + python3 ACLs for ABG users. Voila home dir fix (500→200). Revalidation: **267 PASS, 0 FAIL**. Horizon 1 COMPLETE. |
| 2026-05-08 | Pure Rust `darkforest` validator created. Replaces bash+python pen/fuzz tools. 863KB binary, 159 PASS 0 FAIL 3 DARK_FOREST. Primal ecosystem validated in Rust. |
| 2026-05-08 | `darkforest` v0.2.0 — modular auditable framework. 7 source modules. 13 crypto strength checks (CRY-01→CRY-13). JSON report output (`--output json`). Per-check severity, evidence, remediation. **177 PASS, 0 FAIL, 4 DARK_FOREST**. Cloudflare config.yml fixed (664→600). |
| 2026-05-08 | Workspace scaffolding: pilot lifecycle, per-user scratch, reviewer showcase-only visibility, welcome notebooks, validation dashboard. Compute usability: per-user venvs, wheelhouse, offline pip. darkforest revalidation post-scaffolding: **175 PASS, 0 FAIL, 6 DARK_FOREST** (2 new: reviewer-visibility items detected at filesystem level, remediated via symlink isolation). |
| 2026-05-09 | Open observer landing: Voila public surface at `lab.primals.eco` (no credentials). Observer is default; reviewer/user gated by Cloudflare Access + PAM. Root redirect to Welcome.ipynb, source stripping, internal directory blocking, page titles on all notebooks. |
| 2026-05-09 | Multi-tier test suite: `tier_test_observer.py` (structural, execution, HTTP), `tier_test_reviewer.py` (access, parse, no-write), `tier_test_compute.py` (venv, packages, kernels, execution). `tier_test_all.sh` runner. Identified and fixed: kernel mismatches, missing metadata, relative path errors, dashboard KeyError, package import issues. |
| 2026-05-09 | `pappusCast` auto-propagation daemon: tiered validation (light/medium/heavy), adaptive rate limiting (scales with active users), snapshot architecture (copies not symlinks), quarantine for failures, changelog tracking. Python-first with Rust evolution path. |
| 2026-05-09 | `tunnelKeeper` Rust crate: programmatic Cloudflare tunnel health checks, DNS resolution fallback chain, config file parsing. Integrated into darkforest A6 pen test. First step toward Rust-native Cloudflare interaction. |
