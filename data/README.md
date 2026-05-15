# data/

Data source manifests and validation targets for the unified lineage.

## Structure

```
data/
  sources/
    thread01_wcm.toml              Whole-Cell Modeling (27 sources, 7 ABG papers)
    thread02_plasma.toml            Plasma Physics & Lattice QCD (18 sources)
    thread03_immuno.toml            Immunology & Drug Discovery (18 sources)
    thread04_enviro.toml            Environmental Genomics & Field Science (23 sources)
    thread05_ltee.toml              Evolutionary Biology / LTEE (12 sources)
    thread05_ml_surrogates.toml     ML surrogates for LTEE (neuralSpring)
    thread06_ag.toml                Agricultural Science
    thread07_anderson.toml          Anderson Mathematics
    thread08_health.toml            Human Health & Clinical Translation (14 sources)
    thread09_gaming.toml            Gaming / Creative (ludoSpring)
    thread10_provenance.toml        Provenance / Economics
  targets/
    thread01_wcm_targets.toml            Whole-Cell Modeling (24 targets across Papers A-G)
    thread02_plasma_targets.toml         Plasma Physics & Lattice QCD targets
    thread03_immuno_targets.toml         Immunology & Drug Discovery (12 targets)
    thread04_enviro_targets.toml         Environmental Genomics targets
    thread05_ltee_targets.toml           LTEE evolutionary dynamics (10 targets)
    thread05_ml_surrogates_targets.toml  ML surrogate targets (neuralSpring)
    thread06_ag_targets.toml             Agricultural science targets
    thread07_anderson_targets.toml       Anderson mathematics targets
    thread08_health_targets.toml         Human health targets
    thread09_gaming_targets.toml         Gaming / creative targets (13 targets)
    thread10_provenance_targets.toml     Provenance / economics targets
```

## Public Data Repository Anchors

NCBI BioProjects, UniProt proteomes, PhysioNet waveform databases,
DrugBank entries, ChEMBL bioactivity data, KEGG pathway maps, SILVA
reference taxonomies, and PDB/AlphaFold structures serve as **provenance
chain starting points**. Each is a fetchable, versionable, content-
addressable resource that NUCLEUS hashes and tracks through the entire
computation pipeline via the provenance trio.

When a database updates, the old BLAKE3 hash remains in NestGate and the
new retrieval creates a new ledger entry. The diff between database
versions is structural, not editorial — enabling evolution opportunities
as public data improves.

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
