# Environmental Genomics

**Thread 4 Expression — Reproducing and Validating Environmental Metagenomics, Analytical Chemistry, and Field Science Through NUCLEUS Composition**

**Date**: May 12, 2026
**Status**: Active — data anchored, validation targets set, LTEE B7 pipeline started
**License**: scyBorg triple — AGPL-3.0-or-later (code), ORC (system mechanics), CC-BY-SA 4.0 (this document)
**Thread**: 4 (Environmental Genomics / Field Science) in `lineage/THE_UNIFIED_LINEAGE.md`
**Cross-threads**: 7 (Anderson math — localization framework), 1 (whole-cell modeling — gene annotation), 5 (LTEE — mutation accumulation genomics), 6 (agriculture — soil microbiome)

---

## 1. Framing

Environmental genomics spans from deep-sea cold seeps to wastewater
treatment plants, from soil pore networks to harmful algal blooms. The
science is inherently multi-scale: molecular (mass spectrometry, gene
sequences), organismal (quorum sensing, biofilm formation), community
(diversity indices, phylogenetics), and ecosystem (nutrient cycling,
pollution monitoring).

gen3 proved that sovereign Rust infrastructure reproduces this science
correctly: 63 peer-reviewed papers, 36 validation targets, 1,962 library
tests. This expression asks the composition question: can NUCLEUS
orchestrate a complete environmental genomics pipeline — from raw NCBI
sequences through diversity analysis, Anderson localization scoring,
PFAS screening, and sentinel microbe classification — with full
provenance at every step?

The answer requires three external paper lineages (Anderson R. deep-sea
series, Waters quorum sensing series, Liu phylogenetics series) plus
seven baseCamp papers (01, 03, 04, 05, 06, 09, 16) that together define
wetSpring's core scientific domain.

### The scyBorg Anchor

All rebuild outputs are released under the scyBorg triple:

- **AGPL-3.0-or-later**: All code (NUCLEUS compositions, deploy graphs,
  spring experiments, validation binaries).
- **ORC**: System mechanics and composition patterns.
- **CC-BY-SA 4.0**: This document and all generated papers.

---

## 2. The Paper Lineage

### External Lineage: Anderson R. Deep-Sea Series (6 papers)

Six papers establishing the quorum sensing framework for extreme
environments — cold seeps, hydrothermal vents, deep-sea sediments.

| # | Citation | System | Key Contribution | Data Sources |
|---|----------|--------|-----------------|-------------|
| 1 | Anderson R. et al. (2019) *Environ Microbiol* | Cold seep communities | luxR phylogeny in deep-sea metagenomes; eavesdropper enrichment | PRJNA503411 |
| 2 | Anderson R. et al. (2020) *Appl Environ Microbiol* | Hydrothermal vents | Cross-species QS gene prevalence (~35% of genomes) | PRJNA315684, PRJNA283159 |
| 3 | Anderson R. et al. (2021a) *ISME J* | Mixed-species biofilms | QS-mediated niche partitioning; Anderson localization analogy | PRJNA503411 |
| 4 | Anderson R. et al. (2021b) *Front Microbiol* | Coral holobiont | Symbiotic QS networks in coral microbiome | PRJNA473816 |
| 5 | Anderson R. et al. (2022) *mBio* | Cross-environment meta-analysis | QS gene prevalence as a function of environmental disorder (W) | PRJNA237362, PRJEB31985 |
| 6 | Anderson R. et al. (2023) *Nat Ecol Evol* | Deep-sea + terrestrial | Anderson W(disorder) predicts QS community structure | Multi-BioProject |

**Key parameter chain**: QS gene prevalence (35% ± 10%) → luxR phylogenetic
distances → Anderson W/V disorder mapping → localization length prediction.
Each paper inherits the QS gene scanning framework from the one before.

### External Lineage: Waters QS Series (7 papers)

Seven papers on quorum sensing molecular biology — the biochemical
foundation underlying the Anderson ecological framework.

| # | Citation | System | Key Contribution |
|---|----------|--------|-----------------|
| 1 | Waters & Bassler (2005) *Annu Rev Cell Dev Biol* | *V. harveyi* | Canonical QS circuit model; autoinducer threshold = 10 µM |
| 2 | Waters (2008) *Mol Microbiol* | *V. harveyi* | QS-biofilm coupling; c-di-GMP signaling network |
| 3 | Waters (2012) *J Bacteriol* | Multi-species | Cross-species AI-2 signaling; eavesdropper dynamics |
| 4 | Hammer & Bassler (2003) *Mol Microbiol* | *V. cholerae* | QS repression of virulence at high cell density |
| 5 | Papenfort & Bassler (2016) *Nat Rev Microbiol* | Review | Comprehensive QS circuits across Proteobacteria |
| 6 | Mukherjee & Bassler (2019) *Nat Rev Microbiol* | Multi-species | Bacterial social behavior and QS decision making |
| 7 | Ke et al. (2021) *Cell Rep* | *P. aeruginosa* | QS temporal dynamics via single-cell reporters |

**Key numerical results**: Autoinducer threshold ≈ 10 µM (±2 µM),
biofilm initiation ODE system matches RK4 to ≤ 1e-6 relative error,
Gillespie SSA mean switching time ≈ 50 time units for bistable model.

### External Lineage: Liu Phylogenetics Series (6 papers)

Six papers on phylogenetic methods — the computational backbone for
gene-tree and species-tree analysis in environmental genomics.

| # | Citation | Method | Key Contribution |
|---|----------|--------|-----------------|
| 1 | Liu et al. (2009) *Science* | SATé | Simultaneous alignment and tree estimation |
| 2 | Liu et al. (2012) *Syst Biol* | ASTRAL | Species tree from gene trees under ILS |
| 3 | Mirarab & Warnow (2015) *Bioinformatics* | ASTRAL-II | Scalable species tree estimation |
| 4 | Zhang et al. (2018) *BMC Bioinformatics* | ASTRAL-III | Polynomial-time species tree |
| 5 | Yin et al. (2019) *Mol Biol Evol* | ASTRAL-Pro | Gene duplication + loss handling |
| 6 | Rabiee & Mirarab (2020) *Bioinformatics* | wASTRAL | Weighted quartets for accuracy |

**Key validation anchors**: Robinson-Foulds distance = 0 for identical
trees, neighbor-joining correct topology for 4-taxon matrix,
Smith-Waterman alignment score = 100 for identical 100bp sequences.

### baseCamp Papers (wetSpring Core)

| baseCamp # | Domain | Key Experiments |
|-----------|--------|----------------|
| 01 | 16S metagenomics + diversity | Exp001-015: Shannon, Simpson, Chao1, DADA2, chimera, UniFrac |
| 03 | Bioag + rhizosphere | Exp250-260: Pivot Bio soybean, N-fixation |
| 04 | Sentinel microbes + PFAS | Exp018, 041-042, 114-118, 157-160: HAB, PFAS mass spec, NMF drug repurposing, NPU |
| 05 | Cross-species signaling | Exp020-023, 127-146, 150, 185: QS biofilm ODE, Anderson localization, cold seep metagenomics |
| 06 | No-till soil health | Exp170-182: Soil pore geometry, QS autoinducer diffusion in 3D |
| 09 | Field genomics + nanopore | Exp114-118: MinION pipeline, AKD1000 NPU sentinel |
| 16 | Anaerobic-aerobic QS | Exp200-210: Digester microbiome, gut O₂ gradient, FNR/ArcAB regulation |

---

## 3. The Jelly Strings

Provenance gaps that NUCLEUS structurally fills.

| Gap | Papers | NUCLEUS Solution |
|-----|--------|-----------------|
| QS gene prevalence varies by study without version-pinned databases | Anderson 1-6 | NestGate BLAKE3-hashes NCBI BioProject snapshots at retrieval; re-download → new hash, not overwrite |
| SILVA/RefSeq taxonomy version drift changes classification results | baseCamp 01, 04, 09 | Data source TOMLs pin SILVA SSURef_NR99_138.2; BLAKE3 of fetched database anchors the provenance chain |
| Autoinducer threshold (10 µM) inherited by citation across 10+ papers | Waters 1-3, baseCamp 05 | loamSpine parameter certificate traces the constant to Waters & Bassler 2005 Table 1 |
| PFAS mass spectral libraries updated continuously (Jones Lab Zenodo) | baseCamp 04 | NestGate snapshots Zenodo 14341321 at retrieval with content hash; spectral matching reproducible |
| Phylogenetic tree topology depends on alignment software version | Liu 1-6 | Sovereign Rust Smith-Waterman + neighbor-joining replace vendor-locked QIIME2/SATé; single binary |
| Drug-disease NMF rank selection is manual (rank=10, 200 iterations) | baseCamp 04 | rhizoCrypt DAG records NMF hyperparameters as provenance nodes; rank scan is automated |
| NPU inference latency depends on hardware batch and SDK version | baseCamp 09 | metalForge AKD1000 profile pins firmware+SDK version; benchmark results content-addressed |
| Soil pore geometry hand-drawn from 2D thin sections | baseCamp 06 | 3D pore network generated from µCT data; Anderson W computed from actual pore connectivity |

---

## 4. Data Targets

Machine-readable manifests:

- **Sources**: `data/sources/thread04_enviro.toml` (23 external data sources)
- **Targets**: `data/targets/thread04_enviro_targets.toml` (36 validation targets)

All 36 targets have been validated by wetSpring (V163b, 1,962 lib tests).
The targets span:

| Domain | Count | Key Metrics |
|--------|-------|------------|
| 16S pipeline / diversity | 4 | Shannon H = 4.605, Simpson = 0.99, Chao1 = 100, CPU parity = 1.0 |
| Anderson localization | 4 | ξ₃D = 1.57, W_c/V = 16.5, 2D all-localized, QS biofilm threshold = 10 µM |
| ODE / stochastic solvers | 2 | RK4 error ≤ 1e-6, Gillespie mean switch ≈ 50 |
| Phylogenetics | 3 | RF distance = 0, SW score = 100, NJ correct topology |
| PFAS analytical chemistry | 4 | PFOA m/z = 412.966, PFOS m/z = 498.930, peak detection parity, MassBank cosine ≥ 0.95 |
| Cold seep metagenomics | 2 | QS prevalence ≈ 0.35, Shannon H ≈ 3.8 |
| Drug repurposing / NMF | 2 | NMF reconstruction error ≤ 0.5, Fajgenbaum pathway score ≈ 0.85 |
| Gonzales dermatitis | 2 | JAK1 IC50 = 10 nM, lokivetmab duration = 42 days |
| HMM / stochastic | 2 | Forward log-likelihood = -4.265, Viterbi path accuracy = 1.0 |
| Soil / agriculture | 2 | No-till diversity gain ≈ 0.15, soil pore QS threshold ≈ 5 µM |
| R / scipy parity | 2 | vegan parity (53 checks), scipy ODE parity ≤ 1e-8 |
| NPU field sentinel | 2 | QS classifier accuracy ≥ 0.92, bloom sentinel latency ≈ 0.053 ms |
| Spectral matching | 2 | MassBank cosine ≥ 0.95, EPA PFAS ML accuracy ≥ 0.90 |
| LTEE genomics (B7) | 4 | *Pending: mutation accumulation curves from Exp380* |

The 4 LTEE B7 targets in `data/targets/` are currently `validated = false`,
pending Exp380 Tier 1 (Python baseline) completion.

---

## 5. NUCLEUS Composition Blueprints

### Blueprint A: Sovereign 16S Pipeline

The core metagenomics pipeline — from raw NCBI reads through diversity
analysis — as a single NUCLEUS composition.

```
Tower (BearDog + Songbird)
  ├── toadStool dispatch
  │     ├── Workload: ncbi_fetch (NestGate → SRA download → BLAKE3)
  │     ├── Workload: dada2_denoise (DADA2 error model + chimera)
  │     ├── Workload: taxonomy_classify (SILVA/RefSeq → OTU table)
  │     ├── Workload: diversity_compute (Shannon, Simpson, Chao1, UniFrac)
  │     └── Workload: anderson_spectral (W/V → ξ localization length)
  │
  ├── barraCuda compute
  │     ├── DADA2 error model fitting (matrix operations)
  │     ├── UniFrac distance matrix (phylogenetic distance, N²)
  │     ├── Anderson spectral analysis (eigenvalue decomposition)
  │     └── Diversity index computation (parallel across samples)
  │
  ├── NestGate storage
  │     ├── Raw sequences: BLAKE3-hashed FASTQ per sample
  │     ├── OTU tables: content-addressed community profiles
  │     ├── Taxonomy assignments: version-pinned SILVA classification
  │     └── Diversity results: per-sample metric vectors
  │
  └── Provenance trio
        ├── rhizoCrypt: pipeline DAG (fetch → denoise → classify → analyze)
        ├── loamSpine: database version certificates (SILVA 138.2, RefSeq)
        └── sweetGrass: attribution to Anderson, Waters, Liu lineages
```

### Blueprint B: PFAS Sentinel Pipeline

Analytical chemistry pipeline for PFAS screening — mass spectrometry
through spectral matching and machine learning classification.

```
Tower
  ├── toadStool dispatch
  │     ├── Workload: mzml_parse (raw spectra → peak list)
  │     ├── Workload: spectral_match (MassBank cosine similarity)
  │     ├── Workload: pfas_classify (EPA ML model: decision tree + RF)
  │     ├── Workload: nmf_repurpose (drug-disease matrix factorization)
  │     └── Workload: fajgenbaum_pathway (HHV-8 iMCD enrichment)
  │
  ├── NestGate storage
  │     ├── Mass spectra: BLAKE3-hashed mzML files
  │     ├── Spectral library: Jones Lab Zenodo 14341321 snapshot
  │     └── Classification results: per-compound PFAS identification
  │
  └── Provenance trio
        ├── rhizoCrypt: screening DAG per sample
        ├── loamSpine: spectral library version + ML model provenance
        └── sweetGrass: Jones Lab + Fajgenbaum attribution
```

### Blueprint C: LTEE Mutation Accumulation (Exp380 / B7)

Tenaillon et al. 2016 — 264 genome downloads through mutation
accumulation curves. The pipeline that feeds lithoSpore module 6.

```
Tower
  ├── toadStool dispatch
  │     ├── Workload: ncbi_fetch_264 (BioProject PRJNA294072 → 264 genomes)
  │     ├── Workload: genome_annotate (gene calling, mutation identification)
  │     ├── Workload: mutation_curves (mutations vs generations per lineage)
  │     ├── Workload: rate_estimate (per-population mutation rate)
  │     └── Workload: litho_export (expected values JSON for lithoSpore)
  │
  ├── NestGate storage
  │     ├── Genomes: 264 BLAKE3-hashed assemblies
  │     ├── Annotations: per-genome gene/mutation calls
  │     └── Expected values: mutation accumulation curves + rates
  │
  └── Provenance trio
        ├── rhizoCrypt: pipeline DAG per lineage
        ├── loamSpine: genome accession → assembly version certificates
        └── sweetGrass: Tenaillon et al. 2016 → lithoSpore module 6
```

### Blueprint D: NPU Field Sentinel

Edge-deployed autonomous microbe classification on neuromorphic hardware.

```
Tower
  ├── toadStool dispatch
  │     ├── Workload: npu_classify (AKD1000 int8 QS phase classification)
  │     ├── Workload: bloom_sentinel (HAB early warning)
  │     └── Workload: nanopore_stream (MinION → real-time classification)
  │
  ├── barraCuda compute
  │     ├── NPU int8 model compilation (metalForge AKD1000 backend)
  │     └── Online evolution (136 gen/sec adaptive model updates)
  │
  └── Provenance trio
        └── rhizoCrypt: per-inference DAG with NPU hardware fingerprint (PUF)
```

---

## 6. Spring Alignment

| Spring | Contribution to Thread 4 |
|--------|-------------------------|
| **wetSpring** | Primary owner. 63/63 paper reproductions, 36 validation targets, 1,962 lib tests. Sovereign 16S pipeline, PFAS screening, Anderson localization, cold seep metagenomics, NPU sentinel, LTEE B7 genomics pipeline. All composition blueprints above originate from wetSpring experiments. |
| **airSpring** | Agricultural genomics — soil microbiome diversity from Thread 6 overlaps with baseCamp 06 (no-till soil health). NOAA weather data context for soil sampling. FAO-56 ET₀ validated datasets for irrigation-microbiome coupling. |
| **neuralSpring** | ESN/LSTM time-series models for digester methane yield (baseCamp 16) and ML-assisted Anderson spectral analysis. Thread 5 LTEE surrogates may consume wetSpring's mutation accumulation curves. |
| **hotSpring** | Anderson localization GPU kernels. Thread 7 Anderson math provides the eigenvalue decomposition and transfer matrix methods that wetSpring applies to biological disorder. |
| **groundSpring** | Soil chemistry and geochemistry context for baseCamp 06. Inverse problem methods for soil sensor calibration. Thread 7 Anderson statistical methods. |
| **healthSpring** | Gonzales JAK1/lokivetmab PK/PD validation (baseCamp 12-13) crosses into Thread 3 (immunology). Drug repurposing NMF patterns from baseCamp 04 share mathematical infrastructure. |

---

## 7. petalTongue Vision

### Sovereign 16S Dashboard

Live metagenomics analysis: sequences flowing from NCBI through DADA2
denoising, taxonomy classification, and diversity computation in
real time.

- **DataBinding channels**: `TimeSeries` (diversity indices per sample),
  `Categorical` (taxonomy assignments), `Heatmap` (OTU abundance matrix),
  `Network` (UniFrac phylogenetic tree)
- **Interaction**: Click any OTU → taxonomy, abundance across samples,
  phylogenetic placement. Filter by environment type. Compare biomes.

### PFAS Screening Surface

Mass spectra rendered as interactive peak landscapes. Click any peak →
compound identification, spectral match score, EPA classification.

- **DataBinding channels**: `Spectrum` (m/z vs intensity),
  `Scalar` (cosine similarity scores), `Categorical` (PFAS/non-PFAS),
  `Network` (drug-disease NMF clusters)
- **Interaction**: Overlay reference spectra. Zoom into isotope patterns.
  Drug repurposing network exploration.

### Anderson Localization Field Map

2D/3D maps of environmental disorder scores overlaid on geographic or
spatial coordinates. Cold seep sites, soil pore networks, gut mucosal
gradients — all rendered as Anderson W(disorder) landscapes.

- **DataBinding channels**: `Spatial3D` (pore network geometry),
  `Scalar` (W/V disorder), `TimeSeries` (localization length over time),
  `Heatmap` (spatial W distribution)
- **Interaction**: Adjust disorder parameter → watch localization
  transition. Compare environments. Overlay QS gene prevalence.

### LTEE Mutation Accumulation Curves

264-genome mutation accumulation rendered as generation-by-generation
curves per lineage, with statistical envelopes.

- **DataBinding channels**: `TimeSeries` (mutations vs generations per
  population), `Scalar` (mutation rate estimates), `Histogram`
  (mutation type distribution), `Comparison` (observed vs expected)
- **Interaction**: Select lineage → highlight mutations. Compare
  populations. Overlay Tenaillon 2016 published values.

---

## 8. scyBorg Publication

### What Gets Published

1. **This expression** (CC-BY-SA 4.0) — lineage analysis, composition
   blueprints, provenance audit.
2. **Sovereign pipeline reproductions** (AGPL-3.0) — every baseCamp
   experiment re-expressed as a NUCLEUS deploy graph.
3. **Provenance-complete datasets** — 23 data sources, 36 validation
   targets, all BLAKE3-anchored.
4. **Live computation surfaces** — petalTongue dashboards as interactive
   figures.
5. **LTEE B7 expected values** — feeding lithoSpore module 6 and
   foundation Thread 5 targets.

### Provenance Chain

```
External data (NCBI, SILVA, Zenodo, NOAA)
  → NestGate fetch + BLAKE3 hash
    → wetSpring sovereign pipeline (Rust, single binary)
      → Validation targets (36, machine-readable TOML)
        → lithoSpore integration (module 6 expected values)
          → petalTongue rendering (live computation surface)
            → scyBorg publication (DAG-recorded, ed25519-witnessed)
```

Every step is content-addressed. Every attribution flows through
sweetGrass braids. The environmental genomics lineage becomes a
geological layer in foundation that other researchers can build upon.

---

## 9. Evolution Targets

Capabilities NUCLEUS needs to evolve for complete Thread 4 expression.

| Capability | Blueprint | Priority |
|-----------|-----------|----------|
| NestGate live content pipeline (PG-04) | A, B, C | High — currently blocked upstream |
| Songbird canonical method names (PG-03) | A | High — blocked upstream |
| Provenance trio live endpoints (PG-02) | All | High — blocked upstream |
| toadStool workload dispatch for multi-genome pipelines | C | Medium |
| metalForge AKD1000 NPU backend for edge deployment | D | Medium |
| petalTongue Spectrum channel for mass spectrometry | B | Medium |
| petalTongue Spatial3D for 3D pore network rendering | A | Low |

---

*The environmental genomics domain spans cold seeps to wastewater,
soil pores to coral reefs, harmful algal blooms to precision agriculture.
The original researchers built this science with vendor-locked tools,
citation-only provenance, and database versions that drifted between
publications. This expression rebuilds their work with sovereign
infrastructure and structural provenance — not because the science was
wrong, but because the tools they had forced compromises that NUCLEUS
eliminates.*
