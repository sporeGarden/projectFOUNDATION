# guideStone / FOUNDATION Validation Boundary Specification

**Date**: June 3, 2026 (Wave 74, updated Wave 76+ for BTSP trust levels)
**Status**: Specification — defines validation responsibilities
**Upstream**: primalSpring (owns guideStone)
**Local**: projectFOUNDATION (owns scientific validation)

## Purpose

This document defines the boundary between what `guideStone` validates
(substrate parity across gates) and what `projectFOUNDATION` validates
(scientific truth), and specifies their shared interface.

## Validation Domains

### What FOUNDATION Validates

FOUNDATION owns **scientific correctness** — the question:
"Does this spring's code reproduce the published scientific results?"

| Dimension | Description | Example |
|-----------|-------------|---------|
| **Numerical accuracy** | Computed values match known targets within tolerance | hotSpring's hydrothermal model ±0.001% |
| **Data integrity** | Source datasets are intact (BLAKE3 verification) | CSV/JSON checksums match fetched data |
| **Provenance** | Every validation run is attributable | gate, session, DAG hash, timestamp |
| **Lineage** | baseCamp papers map to specific spring capabilities | 26 papers → 8 springs → 10 threads |
| **Regression detection** | Previous results are not lost when springs evolve | Version bump → re-verify targets |

FOUNDATION does **not** validate:
- Whether results are consistent across different hardware
- Whether the binary is deployable
- Whether the composition graph is healthy

### What guideStone Validates

guideStone owns **substrate parity** — the question:
"Does this binary produce identical results on all certified substrates?"

| Dimension | Description | Example |
|-----------|-------------|---------|
| **Cross-gate reproducibility** | Same inputs → same outputs on different gates | ironGate vs eastGate produce same float |
| **Architecture parity** | x86_64 and aarch64 produce identical results | Verified via cross-compilation tests |
| **Determinism** | Multiple runs on same gate are bitwise identical | No uninitialized memory, stable ordering |
| **Certification** | A gate is "certified" for a spring when parity holds | eastGate certified for hotSpring v0.6.32 |
| **Trust attestation** | Certifications are cryptographically signed by the certifying gate | Ed25519 via bearDog `TrustedIssuerRegistry` |

guideStone does **not** validate:
- Whether the science is correct (that's FOUNDATION)
- Whether the binary is healthy/alive (that's NUCLEUS)
- Data source integrity or availability

### Trust Infrastructure (Wave 76+)

bearDog w135+ introduces multi-issuer authentication and cross-gate trust
via Ed25519 key exchange. This affects guideStone's certification model:

- **Certification is now signed**: When guideStone certifies a spring on a
  gate, the certificate is signed by the gate's Ed25519 key and registered
  in bearDog's `TrustedIssuerRegistry`
- **Cross-gate verification**: Other gates can verify a certification by
  checking the issuer against their trusted registry
- **NestGate federation trust** (s90): BLAKE3 content hashes are now
  federated with trust attestation — lineage records crossing gate
  boundaries carry verifiable provenance

FOUNDATION's role is unchanged: produce lineage records. But the shared
schema (below) now includes optional trust fields that guideStone populates
when certifying cross-gate.

### BTSP Trust Levels and Certification Scope

guideStone certifications operate within the ecosystem's bonding model.
The bond type between gates determines what certification data crosses
the boundary and at what fidelity:

| Bond Type | Trust Model | Certification Data Shared | BTSP Cipher |
|-----------|-------------|---------------------------|-------------|
| **Covalent** | GeneticLineage | Full lineage records + raw actual values + BLAKE3 provenance | `BTSP_NULL` (same family) |
| **Metallic** | Organizational | Full lineage records + values (fleet compute, shared org) | `BTSP_HMAC_PLAIN` |
| **Ionic** | Contractual | Scoped results: pass/fail per target, no raw values | `BTSP_CHACHA20_POLY1305` |
| **Weak** | ZeroTrust | Summary only: certified/not-certified per spring version | `BTSP_CHACHA20_POLY1305` |

#### Covalent Certification (same-family gates)

Gates sharing a family seed (e.g., ironGate ↔ eastGate ↔ strandGate) have
covalent bonds. guideStone certifications between covalent-bonded gates:

- Share full `[record]` data including raw `actual` values and tolerances
- Ed25519 signature verified via `TrustedIssuerRegistry` (local fast path)
- No braid inspection at boundary (pass-through policy)
- Enable bitwise parity verification across architectures

#### Ionic Certification (cross-family partners)

A university lab or ABG ionic compute partner consuming validation results:

- Receives only pass/fail status per target (raw values stripped)
- Method-level capability filtering: `validation.results` allowed,
  `validation.raw_data` denied
- BLAKE3 provenance hash shared for audit, but braid metadata blocked
- guideStone reports scoped to contracted spring/thread subset

#### Weak Certification (public consumers)

sporePrint, public APIs, or extracellular consumers:

- Binary certified/not-certified status only
- No lineage records cross the boundary
- No provenance data (braid stripped per weak bond policy)
- Suitable for sporePrint gallery "certified" badges

### Certification × Bond Type Flow

```
FOUNDATION produces lineage records (always covalent-internal)
                         ↓
guideStone certifies on same gate (covalent — full access)
                         ↓
guideStone cross-gate comparison:
  ├─ Covalent peer → full record exchange, bitwise parity check
  ├─ Metallic fleet → full exchange within organization
  ├─ Ionic partner → scoped results only (capability filtered)
  └─ Weak edge → certified/not-certified badge only
```

### Token Claims for Lineage Queries

When external systems query FOUNDATION lineage via JSON-RPC, bearDog's
`IonicTokenPayload` determines access scope:

| Token field | Purpose |
|-------------|---------|
| `gate_id` | Identifies requesting gate for audit |
| `family_id` | Determines bond type (same family = covalent) |
| `scope` | Method-level access (`validation.*`, `lineage.read`) |
| `verification_source` | "local" / "remote" / "adhoc" — trust provenance |

FOUNDATION's health dashboard endpoint (`foundation.ecosystem_health`)
respects bond type: covalent consumers see full drift data, ionic see
summary counts, weak see only aggregate health status.

### What NUCLEUS Validates

For completeness — NUCLEUS owns **deployment health**:
- Is the composition alive and serving?
- Are all primals reachable?
- Can springs be deployed to target substrates?

## Interface: Shared Lineage Format

FOUNDATION and guideStone share data through a common lineage format.
This enables guideStone to consume FOUNDATION's validation targets and
compare them across gates.

### Shared Data Flow

```
FOUNDATION validates science → produces lineage records
                                         ↓
                              guideStone consumes lineage
                                         ↓
                              guideStone cross-gate comparison
                                         ↓
                              Parity report (pass/fail per gate)
```

### Lineage Record Schema (shared)

```toml
[record]
spring = "hotSpring"
version = "0.6.32"
gate = "ironGate"
thread = "thread01_thermo"
target_id = "hydrothermal_flux_kelvin"
expected = 4231.887
actual = 4231.887
tolerance = 0.001
passed = true
blake3_provenance = "abc123..."
timestamp = "2026-06-03T14:00:00Z"

# Trust fields (populated by guideStone when certifying cross-gate)
[record.trust]
issuer = "ironGate"                    # gate that produced this record
bond_type = "covalent"                 # covalent | metallic | ionic | weak
trust_model = "GeneticLineage"         # GeneticLineage | Organizational | Contractual | ZeroTrust
signature = ""                         # Ed25519 signature (bearDog w135+)
issuer_key_id = ""                     # key ID in TrustedIssuerRegistry
federation_hash = ""                   # NestGate s90 federated BLAKE3
verification_source = ""               # local | remote | adhoc
```

FOUNDATION produces records **without** the `[record.trust]` section.
guideStone enriches them with trust attestation when performing
cross-gate comparison. The trust fields are optional and additive —
FOUNDATION can ignore them when consuming its own records.

guideStone consumes these records and compares the `actual` field
across gates. If all gates produce identical `actual` values (within
the defined `tolerance`), the spring is certified for cross-gate use.
The certification itself is now cryptographically signed (Wave 76+).

### SPRING_VERSIONS.toml as Shared Reference

Both FOUNDATION and guideStone reference `SPRING_VERSIONS.toml`:

- **FOUNDATION** uses it for drift detection (`check-versions` command)
- **guideStone** uses it to know which version was last certified on which gate
- Updates flow: FOUNDATION bumps → guideStone re-certifies → primalSpring records

## Exit Criteria

The interface is considered stable when:

1. FOUNDATION produces machine-readable validation records (lineage TOML or JSON-RPC)
2. guideStone can query FOUNDATION for per-spring-per-gate results
3. Drift detection (`check-versions`) covers both scientific drift and parity drift

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| FOUNDATION lineage records | Active | THE_UNIFIED_LINEAGE.md + SPRING_VERSIONS.toml |
| FOUNDATION drift detection | Active (Wave 74) | `check-versions` subcommand |
| Trust infrastructure | Active upstream (Wave 76+) | bearDog w135+ multi-issuer, NestGate s90 federation |
| BTSP bond-type schema | Defined (Wave 76+) | `bond_type` + `trust_model` in `[record.trust]` |
| Bond-scoped data filtering | Designed | Covalent=full, ionic=scoped, weak=badge |
| guideStone consumption | Not yet built | guideStone spec pending upstream |
| Cross-gate comparison | Not yet built | Blocked on guideStone + trust wiring |
| JSON-RPC lineage query | Planned (P3) | Health dashboard data model ready |
| Token-scoped access | Designed | `IonicTokenPayload` determines response scope |

## Coordination Notes

- guideStone is owned by **primalSpring** (upstream)
- FOUNDATION defines the lineage schema that guideStone will consume
- No code changes in FOUNDATION are needed until guideStone is built
- Trust fields are additive — FOUNDATION ignores them, guideStone populates them
- Bond-type scoping is enforced at the IPC boundary (cellMembrane/bearDog),
  not within FOUNDATION itself — FOUNDATION always produces full records
- When guideStone becomes active, FOUNDATION may expose lineage via JSON-RPC
  (see P3: ecosystem health dashboard)
- Wave 76+ trust infrastructure (bearDog w135+, NestGate s90) enables signed
  cross-gate certifications with BTSP bond-type enforcement
- `BONDING_MODEL_STANDARD.md` in wateringHole is the canonical reference
  for bond types; this spec references it for guideStone-specific application
