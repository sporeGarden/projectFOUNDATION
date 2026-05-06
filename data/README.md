# data/

Data source manifests and validation targets for the unified lineage.

## Structure

```
data/
  sources/          Per-thread data source manifests
  targets/          Per-thread validation target manifests
```

## Data Source Manifests (`sources/`)

Each TOML file declares the external data sources required by a domain
thread. Fields include database accession IDs, retrieval URLs, expected
file formats, and (when available) BLAKE3 hashes of retrieved datasets.

**Schema** (per source entry):

```toml
[[sources]]
id = "brenda_mg_kinetics"
database = "BRENDA"
accessions = ["EC:2.7.1.69", "EC:6.1.1.6"]
url = "https://www.brenda-enzymes.org/"
format = "json"
blake3 = ""                   # populated after first retrieval
retrieved = ""                # ISO 8601 timestamp of last retrieval
thread = 1
papers = ["A", "D"]
notes = "Km and kcat values for M. genitalium metabolic enzymes"
```

Sources are stub-populated initially and filled in as springs begin
implementing thread expressions. The `blake3` field is empty until the
first validated retrieval; once populated, it serves as the integrity
anchor for all downstream computation.

## Validation Target Manifests (`targets/`)

Each TOML file declares the expected numerical results for a domain
thread's papers. These are the values that NUCLEUS compositions must
reproduce to validate the thread.

**Schema** (per target entry):

```toml
[[targets]]
id = "paper_a_cell_cycle"
paper = "A"
description = "Cell cycle duration for M. genitalium whole-cell model"
expected_value = 9.0
unit = "hours"
tolerance = 0.5
source = "Karr et al. 2012, Table 1"
spring = "hotSpring"
blake3 = ""                   # hash of the result artifact when validated
validated = false
```

Targets evolve from stubs (expected values from published papers) to
validated results (BLAKE3-hashed artifacts from NUCLEUS runs).

## BLAKE3 Integrity

All retrieved data and validated results are content-addressed via BLAKE3.
When a data source is first retrieved, its hash is computed and stored in
the manifest. When a validation target is met, the result artifact's hash
is recorded. This creates a verifiable chain from external database to
published result.

NestGate stores the actual data artifacts. These manifests store the
metadata pointers and integrity hashes.
