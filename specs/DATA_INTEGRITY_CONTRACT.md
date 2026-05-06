# Data Integrity Contract

**Version**: 1.0.0
**Status**: Active
**License**: scyBorg triple

---

## Purpose

This contract defines how foundation validates data integrity across
the complete scientific lineage. Every data source, intermediate result,
and final output must be verifiable through a chain of BLAKE3 hashes
anchored by the provenance trio.

---

## 1. Content Addressing

All data artifacts are content-addressed via BLAKE3:

- **Data sources**: External databases (NCBI, BRENDA, PDB, UniProt) are
  retrieved and hashed. The hash is recorded in `data/sources/*.toml`.
- **Intermediate results**: Computation outputs from NUCLEUS runs are
  hashed and stored via NestGate.
- **Validation targets**: When a NUCLEUS run reproduces a published
  result, the result artifact's hash is recorded in
  `data/targets/*.toml`.

BLAKE3 is used exclusively. No MD5, SHA-1, or SHA-256.

---

## 2. Provenance Trio Integration

Every foundation validation run engages the full provenance trio:

| Component | Role in Foundation |
|-----------|-------------------|
| **rhizoCrypt** | Ephemeral DAG session per validation run. Records the sequence of operations from data retrieval through computation to result comparison. |
| **loamSpine** | Permanent ledger entry when a validation target is met. The entry includes the target ID, result hash, source hashes, and timestamp. |
| **sweetGrass** | Attribution braid connecting: original paper authors → data sources → our reproduction → any downstream products that cite the result. |

The trio is **required** (not optional) for foundation validation runs.
The `graphs/foundation_validation.toml` deploy graph reflects this by
marking all three provenance primals as `required = true`.

---

## 3. Data Source Retrieval Protocol

When retrieving external data:

1. Record the source URL, database version, and access timestamp
2. Compute BLAKE3 hash of the retrieved content
3. Store the hash in the appropriate `data/sources/*.toml` manifest
4. Store the artifact via NestGate (`store.put` with BLAKE3 key)
5. Open a rhizoCrypt DAG session recording the retrieval

If the same source is retrieved again and the hash differs, the new
hash is recorded alongside the old one (version tracking). NestGate
stores both versions.

---

## 4. Validation Protocol

When validating a target:

1. Load data sources referenced by the target (verify BLAKE3 hashes)
2. Execute the NUCLEUS composition for the relevant paper
3. Compare numerical results against `data/targets/*.toml` expected values
4. If within tolerance: mark `validated = true`, record result BLAKE3
5. Commit to loamSpine permanent ledger
6. Create sweetGrass attribution braid

A target is only marked validated when the full provenance chain is
recorded. Results without provenance do not count.

---

## 5. Cross-Thread Integrity

When a parameter is shared across threads (e.g. ODE solver constants
shared between Thread 1 whole-cell modeling and Thread 8 healthSpring
PK/PD), the provenance trio records the cross-reference. sweetGrass
braids link the shared parameter to both threads' validation results.

This prevents the "jelly strings" problem: parameters cannot float
across domains without a verifiable attribution trail.

---

## 6. Version Pinning

External databases change. Force fields get updated. Reference genomes
are revised. Foundation pins versions by:

1. Recording the database version in the source manifest
2. Hashing the specific version retrieved
3. Storing the versioned artifact in NestGate

Validation results are always tied to specific data source versions.
If a database updates, re-validation against the new version produces
a new ledger entry — it does not overwrite the old one.
