# Provenance & Economics — Thread 10 Expression

**Springs**: ludoSpring (domain validation), primalSpring (composition orchestration)
**Thread**: 10 (Provenance / Economics)
**Last Updated**: May 12, 2026
**Status**: 6 sources, 8 targets, 3 validated. NFT lifecycle + provenance trio end-to-end proven. Co-seeded by ludoSpring + primalSpring.

## The Core Question

Can digital provenance — the unbroken chain of who created, modified, viewed,
and transferred any digital artifact — replace centralized trust authorities
(blockchain platforms, corporate marketplaces, institutional gatekeepers) with
sovereign, cryptographic, content-addressed evidence?

The answer is the Novel Ferment Transcript (NFT): a memory-bound digital object
whose value derives from its accumulated history rather than artificial scarcity.
A game item's tournament record, a patient record's access log, a scientific
sample's chain of custody — all are isomorphic provenance problems solved by the
same mathematical infrastructure.

## The Provenance Trio

Three NUCLEUS Nest-tier primals compose to form the provenance infrastructure:

| Primal | Role | Mathematical Foundation |
|--------|------|------------------------|
| **rhizoCrypt** | DAG session tracking | Kahn (1962) topological sort, Merkle (1987) content addressing |
| **loamSpine** | Permanent certificate ledger | ISO 17025:2017 chain-of-custody, append-only commit |
| **sweetGrass** | Attribution braids | W3C PROV-O (2013), Ed25519 witness signatures |

The composition pattern: content is BLAKE3-hashed → events accumulate in a
rhizoCrypt DAG → meaningful transitions commit to loamSpine → attribution
braids record who did what in a sweetGrass witness.

## Paper Coverage

| Paper | Domain | Key Results | Status |
|-------|--------|------------|--------|
| 20 | Ferment Transcript Economics | DAG session integrity, dehydration determinism, game session provenance | **89 checks validated** (exp061) |
| 21 | Permanent Ledger (loamSpine) | SessionCommit with Merkle root, immutability guarantees | **Partially validated** (RootPulse Phase 5: 80% executable) |
| 22 | Radiating Attribution (sweetGrass) | Ed25519 braid creation, W3C PROV compliance | **Validated** (witness creation PASS) |

## Five Dimensions of Provenance Validation

| Dimension | What It Measures | Source |
|-----------|-----------------|--------|
| **Integrity** | Content-addressed artifacts cannot be tampered | BLAKE3 + Merkle root determinism |
| **Custody** | Unbroken chain from creation to current holder | rhizoCrypt DAG topology |
| **Attribution** | Who contributed what, when, with what authority | sweetGrass Ed25519 braids |
| **Permanence** | Committed records are append-only, irrevocable | loamSpine SessionCommit |
| **Sovereignty** | No external blockchain, central authority, or vendor dependency | BearDog signing + Songbird discovery |

## Data Flow

```
Published standards + domain validation
    │
    ├── ludoSpring (domain proof): game item lifecycle
    │     └── exp061: mint → trade → loan → consume → achievements
    │     └── exp062: sample chain-of-custody (39/39 PASS)
    │     └── exp063: medical record provenance (35/35 PASS)
    │
    ├── primalSpring (composition proof): trio end-to-end
    │     └── exp094: NUCLEUS parity (Tower + Node + Nest + cross-atomic)
    │     └── RootPulse: dehydration workflow
    │
    └── Downstream consumption
          ├── Patient Records (Thread 8): HIPAA-grade audit trails
          ├── esotericWebb (Thread 9): game item trading without blockchain
          └── Scientific provenance: any domain validating data custody
```

## Cross-Domain Mapping

The insight that unifies Thread 10: all provenance problems are structurally
identical. ludoSpring proved this with exp061's cross-domain validation:

| Game Domain | Scientific Domain | Healthcare Domain | Common Math |
|------------|-------------------|-------------------|-------------|
| Sword with tournament history | DNA sample with lab chain | Medical record with access log | DAG + Merkle |
| Trading card with play record | Specimen with collection chain | Legal doc with custody log | Certificate ledger |
| Artist credit on skin | Author credit on visualization | Contributor attribution on report | Ed25519 braid |
| Atomic swap between players | Custody transfer between labs | Access delegation between providers | Signed transition |
| Public chain anchor for resale | Regulatory compliance proof | HIPAA audit trail | Optional external anchor |

## Economic Model: Memory-Bound Value

Traditional blockchain NFTs derive value from artificial scarcity (minting caps,
gas fees, speculative trading). The Novel Ferment Transcript inverts this:

- **Value = f(accumulated history)** — a Gompertz growth curve (1825) models
  how objects accrete value through meaningful interactions
- **No scarcity required** — value is intrinsic (provenance chain richness),
  not extrinsic (supply constraint)
- **No currency coupling** — objects trade by custody transfer, not financial
  transaction; value is orthogonal to monetary price
- **Fermentation kinetics** — objects "mature" through use; a game item with
  1000 battles is more valuable than one freshly minted, like a wine that has
  aged vs. one just bottled

## NUCLEUS Composition Blueprint

Thread 10 exercises the Nest atomic exclusively — the storage + provenance tier:

| Node | Role | Required |
|------|------|----------|
| BearDog | Ed25519 signing, key management | Yes |
| Songbird | Capability discovery | Yes |
| rhizoCrypt | DAG session management | Yes |
| loamSpine | Certificate ledger (SessionCommit) | Yes |
| sweetGrass | Attribution braids (W3C PROV witnesses) | Yes |
| NestGate | Content storage (BLAKE3-addressed) | Yes |
| biomeOS | Composition orchestration | Yes |
| barraCuda | Cryptographic compute (hashing, verification) | No (BearDog handles signing) |
| skunkBat | Audit logging of provenance operations | No (enrichment layer) |

## Connection to Downstream Products

### Patient Records

HIPAA-grade access logging uses the same trio composition. Every record access
is a DAG event, every modification is a signed certificate commit, every query
is attributed in a braid. ludoSpring proved the pattern with games; healthSpring
applies it to clinical data.

### esotericWebb

Game item trading without blockchain dependency. Players trade directly via
custody transfer (loamSpine certificate transition), with attribution preserved
(sweetGrass braid update) and history immutable (rhizoCrypt DAG finalization).

### Scientific Provenance (Cross-Foundation)

Any domain that produces experimental data benefits: sample chain-of-custody
(Thread 4 genomics, Thread 6 agriculture), computation provenance (Thread 5 LTEE
reproductions), model versioning (Thread 1 whole-cell).

## Cross-Thread Connections

- **Thread 9 (Gaming/Creative)**: Anti-cheat provenance, item economy, attribution
  conservation for player/AI creative collaboration
- **Thread 8 (Human Health)**: HIPAA audit trails, access delegation, consent management
- **Thread 5 (LTEE)**: Experimental data provenance — BLAKE3 content hashes anchor
  scientific results as geological layers
- **Thread 7 (Anderson Math)**: Cryptographic verification mathematics (hashing,
  signing, verification) validated against reference implementations

## What Remains

- **loamSpine SessionCommit**: RootPulse Phase 5 method name mismatch — 80% executable,
  blocked on upstream fix
- **W3C PROV schema automation**: Braid creation works, automated schema validation
  pending
- **Cross-spring provenance**: Thread 10 patterns need absorption by healthSpring
  (Thread 8) and wetSpring (Thread 4) for their domain-specific provenance needs
- **NestGate content pipeline**: Content-addressed storage is the missing piece
  between BLAKE3 hashing and loamSpine commit — awaiting NestGate API stabilization
