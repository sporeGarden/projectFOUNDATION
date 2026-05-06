# foundation

The validated scientific lineage that gives sporeGarden products their
substance. Stand up the soil — data locations, provenance targets,
domain thread maps — so that projectNUCLEUS can grow on top and products
can focus on what matters to their audiences.

**Organization**: sporeGarden (products built on ecoPrimals)
**Generation**: gen4 — composition and deployment
**License**: AGPL-3.0-or-later (code), ORC (system mechanics), CC-BY-SA 4.0 (docs)

## What This Is

foundation is the scientific knowledge layer of the ecoPrimals ecosystem.
It maps the complete validated lineage — 28 baseCamp companion papers,
8 springs with 12,510+ quantitative checks, 70+ reproduced papers, and
12+ faculty and community contacts — as one unified whole, organized into
10 interconnected domain threads.

```
ecoPrimals (organisms) → syntheticChemistry (springs/validation) → sporeGarden (products)
                                    ↓                                       ↓
                            primalSpring                          projectNUCLEUS
                         (validates compositions)          (deploys compositions)
                                                                        ↑
                                                                  foundation
                                                          (the soil: what to validate,
                                                           where the data lives,
                                                           what the targets are)
```

projectNUCLEUS defines HOW to deploy (primals, graphs, gates).
foundation defines WHAT to validate (data, papers, targets, threads).
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

## Deploying via projectNUCLEUS

foundation does not deploy primals directly. It references projectNUCLEUS
graphs and plasmidBin binaries for validation runs.

```bash
# Validate a data thread using NUCLEUS infrastructure
# 1. Ensure plasmidBin binaries are available
cd ../../infra/plasmidBin && ./fetch.sh && ./doctor.sh

# 2. Deploy the foundation validation composition via projectNUCLEUS
cd ../../sporeGarden/projectNUCLEUS/deploy
bash deploy.sh --composition nest --gate irongate

# 3. Run a foundation validation workload
# (toadStool dispatches, provenance trio records, NestGate stores)
toadstool execute ../../sporeGarden/foundation/graphs/foundation_validation.toml
```

## Repo Structure

```
lineage/            The unified lineage — master map and machine-readable thread index
  THE_UNIFIED_LINEAGE.md    Master document: 10 threads, all papers/springs/contacts
  THREAD_INDEX.toml         Machine-readable inventory for tooling
expressions/        Domain thread expression documents
  ABG_WHOLE_CELL_REBUILD.md Thread 1: whole-cell modeling (first expression)
data/               Data source manifests and validation targets
  sources/          Per-thread data source TOMLs (accessions, databases, URLs)
  targets/          Per-thread validation target TOMLs (expected results)
graphs/             Foundation-specific deploy graphs (references projectNUCLEUS)
specs/              Contracts and authoring guides
validation/         Validation results, provenance manifests, gap reports
docs/               External-facing primers and guides
```

## Relationship to Other Repos

| Repo | Org | Relationship |
|------|-----|-------------|
| **projectNUCLEUS** | sporeGarden | The spore — deploys primals that foundation targets reference |
| **plasmidBin** | ecoPrimals/infra | Binary depot — foundation validation runs use primals from here |
| **primalSpring** | syntheticChemistry | Composition validation — foundation references validated graphs |
| **wateringHole** | ecoPrimals/infra | Standards and guidance — foundation follows these |
| **whitePaper** | ecoPrimals/infra | Historical lineage — gen4/foundations/ is the personal record; this repo is the living expression |
| **helixVision** | sporeGarden | Product focus — pulls Threads 1, 3, 4, 5 from foundation |
| **blueFish** | sporeGarden (pending) | Product focus — pulls Thread 4 + data pipeline from foundation |
| **esotericWebb** | sporeGarden | Product focus — pulls Thread 9 from foundation |
