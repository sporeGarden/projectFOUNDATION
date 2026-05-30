# projectFOUNDATION

The validated scientific lineage that gives sporeGarden products their
substance. Stand up the soil — data locations, provenance targets,
domain thread maps — so that projectNUCLEUS can grow on top and products
can focus on what matters to their audiences.

**Organization**: sporeGarden (products built on ecoPrimals)
**Generation**: gen4 — composition and deployment (Wave 63, primalSpring v0.9.30)
**License**: AGPL-3.0-or-later (code), ORC (system mechanics), CC-BY-SA 4.0 (docs)

## What This Is

projectFOUNDATION is the scientific knowledge layer of the ecoPrimals ecosystem.
It maps the complete validated lineage — 26 baseCamp companion papers
(28 including companion hardware and coordination notes),
8 springs with 13,100+ quantitative checks, 70+ reproduced papers, and
16 faculty and community contacts — as one unified whole, organized into
10 interconnected domain threads.

```
ecoPrimals (organisms) → syntheticChemistry (springs/validation) → sporeGarden (products)
                                    ↓                                       ↓
                            primalSpring                          projectNUCLEUS
                         (validates compositions)          (deploys compositions)
                                                                        ↑
                                                          projectFOUNDATION
                                                          (the soil: what to validate,
                                                           where the data lives,
                                                           what the targets are)
```

projectNUCLEUS defines HOW to deploy (primals, graphs, gates).
projectFOUNDATION defines WHAT to validate (data, papers, targets, threads).
Products define WHO benefits (helixVision for biologists, esotericWebb
for creators, blueFish for labs).

The foundation is the soil. NUCLEUS is the spore that grows on top.
Products are the fruiting bodies.

## One Foundation, Many Expressions

The science is not separate. Anderson localization threads through
microbial ecology, immunology, digester engineering, and plasma physics.
The provenance trio validates game sessions, biological samples, and
medical records using the same DAG/certificate/braid machinery.
barraCuda's GPU math powers lattice QCD, protein structure prediction,
environmental monitoring, and drug screening.

Separating these into independent foundations would sever the connections
that make the ecosystem powerful. The foundation is one interconnected
graph. Expression documents zoom into subgraphs.

## The 10 Domain Threads

| Thread | Domain | Key Springs | Key Contacts |
|--------|--------|-------------|-------------|
| 1 | Whole-Cell Modeling | hotSpring, wetSpring, healthSpring | ABG community |
| 2 | Plasma Physics / Lattice QCD | hotSpring | Murillo, Chuna, Bazavov |
| 3 | Immunology / Drug Discovery | wetSpring, airSpring, healthSpring | Gonzales, Lisabeth, Neubig |
| 4 | Environmental Genomics | wetSpring, airSpring | Anderson R., Cahill, Liao |
| 5 | Evolutionary Biology / LTEE | wetSpring, neuralSpring | Dolson, Waters |
| 6 | Agricultural Science | airSpring, groundSpring, wetSpring | Dong, Liao |
| 7 | Anderson Mathematics | hotSpring, groundSpring, wetSpring, neuralSpring | Kachkovskiy |
| 8 | Human Health / Clinical | healthSpring | Mok |
| 9 | Gaming / Creative | ludoSpring | — |
| 10 | Provenance / Economics | ludoSpring, primalSpring | — |

See `lineage/THE_UNIFIED_LINEAGE.md` for the complete mapping with
baseCamp paper numbers, spring check counts, and cross-thread connections.

## Products as Focuses

Products are lenses — each pulls specific threads from the foundation
and presents them for a particular audience.

| Product | Foundation Threads | Audience |
|---------|-------------------|----------|
| **helixVision** | 1, 3, 4, 5 | Field biologists, wastewater engineers, lab scientists |
| **blueFish** | 4 + data pipeline across all threads | Labs needing sovereign ETL and format conversion |
| **esotericWebb** | 9 | Game designers, writers, solo creators |
| **Patient Records** | 8, 10 | Clinicians, patients |
| **Games@Home** | 9 | Players, citizen scientists |

## Launch a Validation Run

### Rust UniBin (`foundation` — Phase B complete, Phase C in progress)

```bash
# Build the foundation binary (pure Rust, ecoBin-compliant, zero C deps)
cargo build --release

# Run the full validation pipeline (skip-fetch for local-only)
# Phases 1-8: health → provenance open → fetch check → registry → execute → compare → commit → report
target/release/foundation validate --skip-fetch

# Check all targets against manifests
target/release/foundation targets --check

# Inspect health of discovered primals (graceful degradation)
target/release/foundation health

# Fetch all sources from manifests (BLAKE3 verified)
target/release/foundation fetch

# Backfill BLAKE3 hashes for local data
target/release/foundation backfill

# Generate sporePrint gallery pages from lithoSpore pseudoSpore registry
target/release/foundation publish --registry ../lithoSpore/pseudospores/registry.toml

# Scan and index domain_profile.toml files from a spring
target/release/foundation profiles --scan-dir ../../springs/hotSpring --spring hotSpring
```

**Current state**: 6 crates, 150 tests, 8k lines, 3.2MB binary, zero library warnings.
IPC phases wired with graceful degradation. Type-safe enums for execution, isolation,
skip conditions. `Cow<str>` zero-copy env expansion. Zero-copy `Observation` types for
comparison. Typed `FetchStatus` and `ProvenanceIds`. Sync CLI with async isolated to
`validate` only. sporePrint gallery generation from pseudoSpore registry. Domain profile
indexing across springs.

**Phase C remaining**: NestGate registration, toadStool dispatch, full `ProvenanceSession`
trio, `backfill --write` TOML mutation, database-specific fetch orchestration, sporePrint
notify trigger from `publish`, bidirectional Forgejo mirror.

### Bash pipeline (canonical — production, pre-Phase C cutover)

```bash
# 1. Deploy NUCLEUS composition (via projectNUCLEUS)
cd "${NUCLEUS_ROOT:-${ECOPRIMALS_ROOT}/projectNUCLEUS}/deploy"
bash deploy.sh --composition nest --gate <active-gate>

# 2. Fetch public data sources and run validation with full provenance
cd "${FOUNDATION_ROOT:-$(git rev-parse --show-toplevel)}/deploy"
bash foundation_validate.sh --thread wcm
```

This fetches genomes from NCBI, proteomes from UniProt, pathway maps from
KEGG — hashes everything with BLAKE3, registers artifacts in NestGate,
executes validation workloads through toadStool, and commits the complete
provenance chain (rhizoCrypt DAG + loamSpine ledger + sweetGrass braid).

See `deploy/README.md` for full options and the sediment layer model.

## Repo Structure

```
crates/             Rust workspace — foundation UniBin (ecoBin-compliant)
  foundation-core/    Types, TOML parsing, config discovery, env expansion
  foundation-ipc/     JSON-RPC 2.0 clients, health triad, provenance sessions
  foundation-fetch/   Manifest-driven fetch, BLAKE3 content addressing, registry
  foundation-validate/ 8-phase pipeline, comparison, execution, reporting
  foundation-publish/ sporePrint gallery generation, pseudoSpore catalog, domain profiles
  foundation-cli/     UniBin entry: validate, fetch, health, targets, backfill, publish, profiles
Cargo.toml          Workspace root (edition 2024, AGPL-3.0, clippy pedantic+nursery)
lineage/            The unified lineage — master map and thread index
  THE_UNIFIED_LINEAGE.md    Master document: 10 threads, all papers/springs/contacts
  THREAD_INDEX.toml         Machine-readable inventory for tooling
  BASECAMP_PAPER_MAP.toml   baseCamp papers → threads, springs, data anchors
expressions/        Domain thread expression documents
  ABG_WHOLE_CELL_REBUILD.md Thread 1: whole-cell modeling (first expression)
data/               Data source manifests and validation targets
  sources/          Per-thread data source TOMLs (11 files, 165 sources, 10 BLAKE3-anchored)
  targets/          Per-thread validation target TOMLs (11 files, 185 targets)
graphs/             Foundation-specific deploy graphs (references projectNUCLEUS)
deploy/             Operational scripts (production-canonical until Phase C cutover)
  lib/              Sourced shell libraries (6 modules)
    env.sh              Centralized env bootstrap (ECOPRIMALS_ROOT, SPRINGS_ROOT, FAMILY_ID)
    primal_ipc.sh     Primal discovery, RPC clients, blake3_hash
    json_rpc.sh       Typed JSON-RPC response parsing
    thread_registry.sh  Runtime thread metadata from THREAD_INDEX.toml
    target_compare.sh   Phase 6 target comparison logic
    report_writer.sh    Phase 8 report generation and spring distribution
  discovery_defaults.toml  Bootstrap port defaults (single source of truth)
  fetch_sources.sh  Fetch NCBI/UniProt/KEGG data, compute BLAKE3 hashes
  backfill_hashes.sh  Compute BLAKE3 hashes and update source TOMLs
  foundation_validate.sh  Full validation pipeline with provenance wrapping
sporeprint/         sporePrint content — validation summary + generated pseudoSpore gallery
workloads/          toadStool-executable workload definitions (29 workloads, 10 threads)
benchmarks/         barraCuda CPU parity baselines (6 scripts, 32 test cases)
specs/              Contracts and authoring guides
validation/         Validation results, provenance manifests, gap reports
  <spring>/<date>/  Spring-oriented dated folders (per PROVENANCE_FOLDER_CONVENTION.md)
  wetSpring/braids/  Ferment transcript braids (computation-verified provenance)
  handbacks/        Geological record from projectNUCLEUS deployment validation
  COMPOSITION_GAPS.md  Composition-level capability mismatches (Wave 20 resolutions marked)
docs/               External-facing primers and guides
  FOUNDATION_PRIMER.md  projectFOUNDATION orientation and thread map
  BONDING_MODELS.md Atomic bonding architecture (covalent/ionic/metallic)
  DEGRADATION_BEHAVIOR.md  Pipeline degradation matrix (per ecosystem standard)
  NUCLEUS_PRIMER.md Ecosystem primer — orientation for new contributors
```

## Relationship to Other Repos

| Repo | Org | Relationship |
|------|-----|-------------|
| **lithoSpore** | sporeGarden | First Targeted GuideStone — USB-deployable LTEE validation artifact built from foundation threads 1, 2, 4, 7 |
| **projectNUCLEUS** | sporeGarden | The spore — deploys primals, produces gap handbacks that settle here as geological record |
| **plasmidBin** | ecoPrimals/infra | Binary depot — foundation validation runs use primals from here |
| **primalSpring** | syntheticChemistry | Composition validation — foundation references validated graphs |
| **wateringHole** | ecoPrimals/infra | Standards and guidance — foundation follows these |
| **whitePaper** | ecoPrimals/infra | Historical lineage — gen4/foundations/ is the personal record; this repo is the living expression |
| **helixVision** | sporeGarden | Product focus — pulls Threads 1, 3, 4, 5 from foundation |
| **blueFish** | sporeGarden (pending) | Product focus — pulls Thread 4 + data pipeline from foundation |
| **esotericWebb** | sporeGarden | Product focus — pulls Thread 9 from foundation |
