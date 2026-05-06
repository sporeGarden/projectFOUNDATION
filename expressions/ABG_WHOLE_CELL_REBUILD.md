# ABG Whole-Cell Rebuild

**Thread 1 Expression — Reproducing and Validating the Complete Whole-Cell Computational Modeling Lineage (2012–2026) Through NUCLEUS Composition**

**Date**: May 6, 2026
**Status**: Active — analysis and composition mapping
**License**: scyBorg triple — AGPL-3.0-or-later (code), ORC (system mechanics), CC-BY-SA 4.0 (this document)
**Thread**: 1 (Whole-Cell Modeling) in `lineage/THE_UNIFIED_LINEAGE.md`
**Cross-threads**: 7 (Anderson math), 3 (immunology/kinetics), 4 (environmental genomics)

---

## 1. Framing

gen3 proved that sovereign Rust+GPU infrastructure computes correct science
by reproducing 70+ published papers across 8 domains. This expression asks
the next question: can NUCLEUS reproduce an **entire field** — 14 years,
7 papers, 3 organisms, 4 research groups — and fill the provenance gaps
that the original authors could not?

The whole-cell modeling lineage is the first validation target because it
represents exactly the kind of science NUCLEUS is designed to replace:
parameters borrowed across organisms with citation-only provenance,
MATLAB/Python/C++ simulations with no reproducibility chain, hand-
reconciled data between scales, and post-hoc figures disconnected from
the computations that produced them.

### The scyBorg Anchor

The rebuild and its publications are released under the scyBorg triple:

- **AGPL-3.0-or-later**: All code (NUCLEUS compositions, deploy graphs,
  spring experiments). Anyone who builds on it shares modifications back.
- **ORC**: System mechanics and composition patterns.
- **CC-BY-SA 4.0**: This document and all generated papers.

The rebuild itself becomes a sovereign publication: DAG-recorded,
BLAKE3-hashed, ed25519-witnessed, loamSpine-committed.

---

## 2. The Paper Lineage

Seven papers, three organisms, four research groups, fourteen years. Each
paper built on the last — inheriting parameters, extending methods, and
accumulating provenance debt.

### Paper A — Whole-Cell Computational Model Predicts Phenotype from Genotype

- **Citation**: Karr JR et al. (2012) Cell 150(2):389–401
- **DOI**: 10.1016/j.cell.2012.05.044
- **Organism**: *Mycoplasma genitalium* G37 (525 genes, smallest free-living cell)
- **Contribution**: First comprehensive whole-cell model — 28 coupled submodels covering metabolism, transcription, translation, DNA replication, cytokinesis
- **Toolchain**: MATLAB (>28 submodel classes), custom ODE solvers, Gillespie SSA
- **Data sources**: ~1,900 experimentally observed parameters from >900 publications
  - BRENDA enzyme kinetics (Km, kcat, Vmax)
  - EcoCyc metabolic pathway definitions
  - KEGG genome/pathway maps (org code: `mge`)
  - UniProt proteome (UP000000807)
  - NCBI genome (NC_000908.2)
- **Key results**: Cell cycle duration ~9 hours, growth rate ~0.077 h⁻¹, gene essentiality prediction accuracy 79%
- **Published code**: WholeCell GitHub repository (MATLAB)
- **Figures**: 7 main figures — growth curves, cell cycle timing, single-cell variability, gene essentiality predictions, metabolite concentrations

### Paper B — Whole-Cell Models and Simulations in Molecular Detail

- **Citation**: Feig M, Sugita Y (2019) Annu Rev Cell Dev Biol 35:191–211
- **DOI**: 10.1146/annurev-cellbio-100617-062542
- **Contribution**: Review surveying physical/atomistic approaches. Identifies the gap between kinetic models (Paper A) and all-atom structural models. Discusses integration challenges: system size (billions of atoms), timescale gaps (ns to hours), force field accuracy for crowded environments.
- **NUCLEUS relevance**: Frames the exact multi-scale composition problem that Papers E–G attempt and NUCLEUS formalizes

### Paper C — Designing Minimal Genomes Using Whole-Cell Models

- **Citation**: Rees-Garbutt J et al. (2020) Nat Commun 11:836
- **DOI**: 10.1038/s41467-020-14545-0
- **Organism**: *M. genitalium* (using Paper A model)
- **Contribution**: Algorithmic genome minimization — Minesweeper (greedy) and GAMA (genetic algorithm) systematically remove genes while maintaining viability in-silico
- **Data sources**: Paper A model + gene essentiality data
- **Key results**: GAMA explored 53,451 genomes, found designs with up to 237 genes removed (45% of genome). Minesweeper produced smaller reductions faster.
- **Parameter inheritance**: Directly inherits Paper A's 1,900+ parameters and 28 submodels. No independent parameter validation.

### Paper D — The E. coli Whole-Cell Modeling Project

- **Citation**: Sun G et al. (2020) Synth Biol (Oxf) 5(1):ysaa020
- **DOI**: 10.1093/synbio/ysaa020
- **Organism**: *Escherichia coli* K-12 MG1655 (4,623 genes)
- **Contribution**: Multi-generation *E. coli* whole-cell model through "deep curation" — systematic gene-by-gene kinetic parameter assembly
- **Toolchain**: Python (Vivarium framework), multi-process architecture
- **Data sources**:
  - EcoCyc pathway definitions
  - BRENDA enzyme kinetics
  - NCBI genome (NC_000913.3 for K-12 MG1655)
  - UniProt proteome (UP000000625)
  - ~2,000 gene products characterized
- **Key results**: Reproduces growth, division, and multi-generation lineage behavior
- **Architectural insight**: Python multi-process design parallels NUCLEUS composition — each submodel is an independent process communicating via message passing

### Paper E — Fundamental Behaviors Emerge from Simulations of a Living Minimal Cell

- **Citation**: Thornburg ZR et al. (2022) Cell 185(2):345–360
- **DOI**: 10.1016/j.cell.2021.12.025
- **Organism**: JCVI-syn3A (493 genes, synthetic minimal cell derived from *M. mycoides*)
- **Contribution**: First fully dynamical kinetic model of a minimal cell — hybrid stochastic-deterministic with 3D spatial resolution. Couples genetic information processing (stochastic, Lattice Microbes) with metabolism (deterministic, ODE).
- **Toolchain**: Lattice Microbes (C++/CUDA), custom ODE solver, Python glue
- **Data sources**:
  - NCBI genome: JCVI-syn3A (CP016816.2)
  - NCBI BioProject: PRJNA357500 (JCVI synthetic biology)
  - BRENDA/TECRdb enzyme kinetics and thermodynamics
  - Proteomics from Breuer et al. (2019) — copy numbers per protein
  - RNA-seq expression data from JCVI
  - Lipidomics and metabolomics from JCVI-syn3A cultures
- **Key results**: Emergent ribosome biogenesis (20-min doubling), spatial organization, metabolite dynamics matching experimental measurements. Cell cycle ~111 min.
- **Compute**: GPU-accelerated (CUDA) for Lattice Microbes spatial stochastic simulation

### Paper F — Molecular Dynamics Simulation of an Entire Cell

- **Citation**: Stevens JA et al. (2023) Front Chem 11:1106495
- **DOI**: 10.3389/fchem.2023.1106495
- **Organism**: JCVI-syn3A
- **Contribution**: First coarse-grained molecular dynamics simulation of an entire cell — 561 million CG beads representing the complete proteome, genome, membrane, and metabolites
- **Toolchain**: GROMACS (MD engine), Martini 3 coarse-grained force field, OpenMM for validation, custom builder scripts (Python)
- **Data sources**:
  - JCVI-syn3A proteomics for copy numbers (Breuer et al.)
  - AlphaFold2 structure predictions for all syn3A proteins
  - Martini 3 CG parameters (Marrink group, University of Groningen)
  - PDB/UniProt structural data
  - Experimental membrane composition from JCVI
- **Key results**: Stable 200 ns simulation. Protein diffusion coefficients consistent with experimental FRAP. Membrane organization and protein crowding match cryo-EM.
- **Compute**: Massive GPU requirement — ~6 billion atom-equivalents. GROMACS on HPC GPU clusters.

### Paper G — Bringing the Genetically Minimal Cell to Life (4D Cell Cycle)

- **Citation**: Thornburg ZR et al. (2026) Cell (in press)
- **Organism**: JCVI-syn3A
- **Contribution**: Full 4D (space + time) cell cycle simulation — integrates kinetic model (Paper E), structural model (Paper F), chromosome dynamics, membrane morphology, and cell division. First simulation to capture an entire cell cycle from birth through division with spatial, temporal, and molecular resolution.
- **Toolchain**: Lattice Microbes (C++/CUDA) + custom ODE + CG-MD + chromosome dynamics + morphological transformation engine
- **Data sources**: All of Papers E and F, plus new experimental data on chromosome segregation and membrane remodeling during division, cryo-ET structural data
- **Key results**: Complete cell cycle timing (~120 min), chromosome segregation dynamics, membrane constriction and scission, daughter cell asymmetry
- **Compute**: Multi-GPU, multi-scale

### The Lineage Map

```
Paper A (2012)
  M. genitalium WCM — 28 MATLAB submodels, 1,900 parameters
  │
  ├── Paper C (2020)
  │     Genome minimization ON Paper A model
  │     53,451 genomes explored, same parameters inherited
  │
  └── Conceptual bridge → different organism, similar approach
        │
Paper D (2020)
  E. coli WCM — Python multi-process, "deep curation"
  Independent parameter set, parallel methodology
        │
Paper B (2019, review)
  Identifies the kinetic↔structural gap
  Frames the multi-scale challenge
        │
        ├── Paper E (2022)
        │     JCVI-syn3A kinetic model
        │     Lattice Microbes + ODE, hybrid stochastic-deterministic
        │     Draws from BRENDA/TECRdb, proteomics, lipidomics
        │     │
        │     └── Paper G (2026)
        │           Full 4D cell cycle
        │           Integrates E + F + chromosome + morphology
        │
        └── Paper F (2023)
              JCVI-syn3A MD — 561M CG beads
              GROMACS/Martini 3, AlphaFold2 structures
              │
              └── Paper G (2026) ← merges E and F lineages
```

---

## 3. The Jelly Strings Inventory

Every point where trust is social rather than structural — where the
lineage holds together because someone said "trust me" rather than
because the system enforces integrity.

### Cross-Paper Parameter Inheritance

| From | To | What Transfers | Provenance |
|------|----|---------------|-----------|
| ~900 publications | Paper A | 1,900 experimental parameters | Citation-only. No content hashing. No version tracking. |
| Paper A (all params) | Paper C | Full 28-submodel parameter set | "We used the published model." No parameter diff. |
| BRENDA/TECRdb | Paper E | Enzyme kinetics for 493 genes | Database query with no snapshot. BRENDA updates continuously. Which version? |
| Experimental proteomics | Paper E | Initial protein copy numbers | Supplementary table, not content-addressed. No machine-readable link to raw mass spec. |
| AlphaFold2 predictions | Paper F | 3D structures for all syn3A proteins | AF2 model version not pinned. Predictions evolve with model updates. |
| Martini 3 force field | Paper F | CG parameters for all molecules | Version specified but parameter files not content-addressed. |
| Papers E + F (all) | Paper G | Complete kinetic + structural models | "We integrated the models from [refs]." No structural verification of parameter consistency. |

### Software Version Ambiguity

| Paper | Tool | Version Concern |
|-------|------|----------------|
| A | MATLAB | Version unspecified. Numeric behavior varies between releases. |
| C | MATLAB (Paper A code) | Same MATLAB version? Same codebase commit? |
| D | Python (Vivarium) | Python/numpy/scipy versions — floating-point behavior depends on all three |
| E | Lattice Microbes (C++/CUDA) | CUDA version, GPU architecture, compiler flags affect stochastic trajectories |
| F | GROMACS | 2021.x vs 2022.x have known numerical differences. Which patch? |
| G | All of the above | Multi-tool integration — which versions of each were used together? |

### Manual Data Reconciliation

| Papers | Reconciliation Point | Method |
|--------|---------------------|--------|
| A ↔ experimental | Growth rate, cell cycle timing | Manual comparison to published values |
| E ↔ proteomics | Protein concentrations | Manual mapping from mass spec to model species |
| E ↔ lipidomics | Membrane composition | Manual lipid species mapping |
| F ↔ cryo-EM | Protein distribution | Visual comparison of simulation snapshots to EM images |
| E + F → G | Model integration | Manual alignment of kinetic state variables to structural coordinates |

### Citation-Only Provenance for Critical Constants

| Constant | Paper | Source | Structural Provenance |
|----------|-------|--------|--------------------|
| Ribosome elongation rate | E | Literature estimate | None — "approximately 12 aa/s based on [refs]" |
| Membrane lipid ratios | E, F | Experimental lipidomics | Supplementary table, not content-addressed |
| Gene essentiality calls | A, C | Experimental knockouts | Literature compilation, no machine-readable source |
| Diffusion coefficients | F | Experimental FRAP measurements | Reference to published values, no raw data link |
| Division machinery parameters | G | "Estimated from literature" | No specific source for several key parameters |

### NUCLEUS Structural Solutions

| Gap | NUCLEUS Solution |
|-----|-----------------|
| Parameter provenance | NestGate content-addresses every parameter with BLAKE3. Provenance trio traces inheritance across all 7 papers. |
| Cross-organism borrowing | sweetGrass attribution braids record exactly which parameters were borrowed from which organism/paper. |
| Scale bridging | toadStool dispatches multi-scale coupled simulations. Deploy graphs encode the coupling topology. |
| Toolchain reproducibility | Sovereign Rust/WGSL pipeline replaces vendor-locked toolchains. Single binary per primal. |
| Figure-computation disconnect | petalTongue renders figures as live computation surfaces. The figure IS the computation. |
| Version pinning | NestGate stores force field versions, database snapshots, and model weights with content-addressed integrity. |
| Database drift | Data source TOMLs in `data/sources/` pin database versions at retrieval time. BLAKE3 of the retrieved dataset is the anchor. Re-retrieval produces a new hash → new ledger entry, not an overwrite. |

---

## 4. Public Data Anchors

Public data repositories are the starting points for provenance chains.
Every external dataset becomes a BLAKE3-anchored, NestGate-stored,
loamSpine-committed artifact. These are real, fetchable, versionable
data that NUCLEUS can hash and track.

### NCBI Anchors

| Accession | Database | Description | Papers |
|-----------|----------|-------------|--------|
| NC_000908.2 | NCBI Nucleotide | *M. genitalium* G37 complete genome | A, C |
| CP016816.2 | NCBI Nucleotide | JCVI-syn3A complete genome | D, E, F, G |
| NC_000913.3 | NCBI Nucleotide | *E. coli* K-12 MG1655 complete genome | D |
| PRJNA357500 | NCBI BioProject | JCVI synthetic biology project | E, G |
| GCA_000027325.1 | NCBI Assembly | *M. genitalium* G37 assembly | A, C |
| GCA_900015295.1 | NCBI Assembly | JCVI-syn3.0 assembly | E, F, G |

Each accession is a provenance chain anchor. Fetch → hash → store →
track. When BRENDA updates an enzyme constant that Paper A used, the
old hash remains in NestGate and the new hash creates a new ledger
entry. The diff is structural, not editorial.

### UniProt Anchors

| Accession | Description | Papers |
|-----------|-------------|--------|
| UP000000807 | *M. genitalium* reference proteome (484 proteins) | A, C |
| UP000000625 | *E. coli* K-12 reference proteome (4,403 proteins) | D |
| UP000018174 | *M. mycoides* subsp. capri (closest to JCVI-syn3A) | E, F, G |

### KEGG Anchors

| Org Code | Description | Papers |
|----------|-------------|--------|
| mge | *M. genitalium* metabolic network | A, C |
| eco | *E. coli* K-12 metabolic network | D |
| mmc | *M. mycoides* subsp. capri metabolic network | E |

### PDB/AlphaFold Anchors

| Resource | Description | Papers |
|----------|-------------|--------|
| AlphaFold DB (syn3A) | Structure predictions for all 493 syn3A gene products | F, G |
| PDB structures (ribosome) | Experimental ribosome structures for validation | E, F |
| EMDB cryo-ET | Electron tomography maps of syn3A cells | G |

### Force Field / Parameter Database Anchors

| Resource | Version | Description | Papers |
|----------|---------|-------------|--------|
| BRENDA | (version at retrieval) | Enzyme kinetics (Km, kcat, Vmax) | A, D, E |
| TECRdb (NIST) | (version at retrieval) | Thermodynamic equilibrium constants | E |
| CHARMM36m | July 2021 release | All-atom force field parameters | (reference for F) |
| Martini 3 | v3.0.0 (Souza et al. 2021) | Coarse-grained force field | F, G |
| EcoCyc | (version at retrieval) | Metabolic pathway definitions | A, D |

**Machine-readable manifests**: `data/sources/thread01_wcm.toml`
**Validation targets**: `data/targets/thread01_wcm_targets.toml`

The provenance chain for every constant in the rebuild starts at one of
these public anchors. sweetGrass braids flow from anchor → parameter →
computation → result → publication.

---

## 5. NUCLEUS Composition Blueprints

### Paper A → NUCLEUS: The 28-Submodel Composition

**Minimum viable atomics**: Node (toadStool + barraCuda) + Nest (NestGate + provenance trio).

```
Tower (BearDog + Songbird)
  ├── toadStool dispatch
  │     ├── Workload: metabolism_fba (flux balance, ODE)
  │     ├── Workload: transcription_stochastic (Gillespie)
  │     ├── Workload: translation_stochastic (Gillespie)
  │     ├── Workload: dna_replication (state machine)
  │     ├── Workload: protein_folding (kinetic)
  │     ├── Workload: rna_processing (kinetic)
  │     ├── ... (28 submodels total)
  │     └── Workload: cell_cycle_coordinator (master clock)
  │
  ├── barraCuda compute
  │     ├── ODE solver (metabolism — stiff system)
  │     ├── Gillespie SSA (transcription, translation)
  │     └── Matrix operations (FBA linear programming)
  │
  ├── NestGate storage
  │     ├── Parameter store: 1,900 experimental parameters (BLAKE3)
  │     ├── State snapshots: per-timestep cell state
  │     └── Results: growth curves, division events
  │
  └── Provenance trio
        ├── rhizoCrypt: simulation session DAG
        │     └── event per submodel timestep + coupling event
        ├── loamSpine: parameter certificates
        │     └── each of 1,900 parameters traced to source publication
        └── sweetGrass: attribution braid
              └── Karr et al. + 900 source publications → our reproduction
```

### Paper C → NUCLEUS: The Genome Search

```
Tower
  ├── toadStool dispatch
  │     ├── Workload: minesweeper_search (greedy, sequential)
  │     ├── Workload: gama_evolution (genetic algorithm, parallelizable)
  │     └── Workload: viability_test (Paper A model evaluation per genome)
  │
  ├── NestGate storage
  │     ├── Genome library: 53,451 genome designs (content-addressed)
  │     ├── Viability results: per-genome simulation outcomes
  │     └── Fitness landscape: topology of viable ↔ inviable boundary
  │
  └── Provenance trio
        ├── rhizoCrypt: search DAG (each genome evaluation is a node)
        ├── loamSpine: genome certificates (each design with its fitness)
        └── sweetGrass: Rees-Garbutt et al. + Paper A attribution
```

**Key pattern**: Paper C composes Paper A as a subroutine. The Paper A
NUCLEUS composition becomes a callable workload within Paper C's search.
This is NUCLEUS composition of compositions — deploy graphs consuming
deploy graphs.

### Paper E → NUCLEUS: The Hybrid Kinetic Model

```
Tower
  ├── toadStool dispatch
  │     ├── Workload: lattice_microbes (spatial stochastic, CUDA)
  │     ├── Workload: ode_metabolism (deterministic, coupled)
  │     ├── Workload: genetic_info_processing (stochastic)
  │     └── Workload: coupling_coordinator (exchanges state every dt)
  │
  ├── barraCuda compute
  │     ├── Lattice Microbes GPU kernels (reaction-diffusion on 3D grid)
  │     ├── ODE solver (metabolism)
  │     └── State exchange (spatial → bulk coupling)
  │
  ├── NestGate storage
  │     ├── 3D grid states: particle positions per timestep
  │     ├── Metabolite trajectories: concentration time series
  │     ├── Proteomics reference: experimental copy numbers (BLAKE3)
  │     └── Parameter store: BRENDA/TECRdb kinetic constants (BLAKE3)
  │
  └── Provenance trio
        ├── rhizoCrypt: simulation DAG
        │     ├── stochastic branch (Lattice Microbes events)
        │     └── deterministic branch (ODE steps)
        ├── loamSpine: parameter provenance
        │     ├── each BRENDA constant → database version + query
        │     └── each proteomics value → raw mass spec link
        └── sweetGrass: Thornburg et al. 2022 + Luthey-Schulten group
```

### Paper F → NUCLEUS: The MD Simulation

```
Tower
  ├── toadStool dispatch
  │     ├── Workload: gromacs_md (CG-MD, GPU-accelerated)
  │     ├── Workload: structure_builder (cell assembly from components)
  │     ├── Workload: analysis_pipeline (diffusion, density, RDF)
  │     └── Workload: trajectory_processing (frame extraction, filtering)
  │
  ├── barraCuda compute
  │     ├── CG force field evaluation (Martini 3 nonbonded + bonded)
  │     ├── PME electrostatics (GPU FFT)
  │     └── Trajectory analysis kernels (diffusion, spatial correlation)
  │
  ├── NestGate storage
  │     ├── Trajectory frames: XTC/TRR format, content-addressed per frame
  │     ├── Structure files: PDB/GRO with BLAKE3 hashes
  │     ├── Force field parameters: Martini 3 ITP files (BLAKE3)
  │     ├── AlphaFold2 structures: per-protein predictions (BLAKE3, AF2 version pinned)
  │     └── Experimental reference: cryo-EM tomograms, FRAP data
  │
  └── Provenance trio
        ├── rhizoCrypt: MD session DAG (build → equilibration → production)
        ├── loamSpine: structure certificates
        │     ├── each protein → AF2 prediction (model version + confidence)
        │     ├── each CG parameter → Martini 3 version
        │     └── each copy number → proteomics experiment
        └── sweetGrass: Stevens et al. 2023 + Marrink group (Martini) +
              DeepMind (AlphaFold2) + Breuer et al. (proteomics)
```

### Paper G → NUCLEUS: The Full 4D Composition

The capstone — integrating all preceding work into a single multi-scale
simulation. The most complex composition in the foundation.

```
Tower
  ├── toadStool dispatch
  │     ├── Workload: lattice_microbes (from Paper E)
  │     ├── Workload: ode_metabolism (from Paper E)
  │     ├── Workload: chromosome_dynamics (polymer + motor model)
  │     ├── Workload: membrane_morphology (shape transformations)
  │     ├── Workload: division_machinery (FtsZ ring + constriction)
  │     ├── Workload: md_structural (from Paper F, optional refinement)
  │     └── Workload: integration_coordinator (couples all scales)
  │
  ├── barraCuda compute
  │     ├── All Paper E kernels (reaction-diffusion, ODE)
  │     ├── All Paper F kernels (CG force field, PME)
  │     ├── Chromosome polymer dynamics (bead-spring + motor forces)
  │     ├── Membrane shape solver (Helfrich energy + constriction)
  │     └── Multi-scale coupling (spatial interpolation between grids)
  │
  ├── NestGate storage
  │     ├── All Paper E + F storage patterns
  │     ├── Chromosome trajectories: polymer configurations over time
  │     ├── Morphology snapshots: cell shape at each timestep
  │     ├── Division events: constriction metrics, scission timing
  │     └── Full cell state checkpoints: complete serialized state
  │
  └── Provenance trio
        ├── rhizoCrypt: cell cycle DAG
        │     └── birth → growth → DNA replication → chromosome segregation
        │         → constriction → scission → daughter cells
        ├── loamSpine: complete parameter lineage
        │     └── merges Paper E + F certificates with new division params
        └── sweetGrass: full attribution braid
              └── Thornburg 2026 + all upstream (Karr, Rees-Garbutt, Sun,
                  Thornburg 2022, Stevens, Luthey-Schulten, Marrink, JCVI)
```

---

## 6. Spring Alignment

### Alignment Matrix

| Paper | hotSpring | wetSpring | healthSpring | neuralSpring | ludoSpring |
|-------|-----------|-----------|-------------|-------------|-----------|
| A | Monte Carlo | Gene annotation | Kinetics | — | Composition |
| B | (review) | (review) | (review) | (review) | (review) |
| C | — | Genome library | — | — | Composition-of-compositions |
| D | — | Gene curation | — | ML params | Composition |
| E | ODE, particles | — | Kinetics, stiff ODE | Time series | Composition |
| F | CG-MD, GPU | — | — | Eigensolve | Composition |
| G | All of E+F | — | Stiff ODE | Time series | Full NUCLEUS composition |

### Per-Spring Contributions

| Spring | Contribution to Thread 1 |
|--------|-------------------------|
| **hotSpring** | GPU-accelerated particle dynamics (Paper F: CG-MD), ODE solvers (Paper E: metabolism), Monte Carlo sampling (Paper A: stochastic submodels), metalForge hardware abstraction (all papers). RTX 5060 sovereign dispatch and metalForge multi-backend pattern directly apply. 990+ tests, 176 experiments. |
| **wetSpring** | Genomics pipelines for gene annotation (Papers A/D), NCBI integration via NestGate NCBILiveProvider (genome fetching, accession resolution), DADA2 chimera detection (Paper D quality control), provenance pipeline pattern (BLAKE3 on all outputs). 1,902+ tests, 100% binary provenance. |
| **healthSpring** | PK/PD kinetic modeling — Hill equation, NLME FOCE+SAEM, Michaelis-Menten (Paper E enzyme kinetics share the same math). Stiff ODE integration patterns. Pioneered `primal-proof` IPC pattern. 940+ tests, 9 clinical tracks. |
| **neuralSpring** | ML-assisted parameter estimation, ESN reservoir computing for time-series analysis of simulation output, batched eigensolve for protein dynamics (Paper F). 1,403+ tests. |
| **ludoSpring** | Deploy graph validation (9 graphs, 90 checks), BYOB schema compliance, session-as-primal pattern, composition-of-compositions (Paper C composing Paper A). 791 tests, 15 IPC methods. |

---

## 7. petalTongue: The Computation IS the Presentation

Every figure in every paper becomes a live computation surface.

### Paper A → Live Submodel Dashboard

28-panel live dashboard: each submodel renders output as it computes.
Metabolism flux balance updates in real time. Transcription events appear
as gene expression pulses. DNA replication progress fills.

- **DataBinding channels**: `TimeSeries` (metabolites), `Categorical`
  (gene essentiality), `Scalar` (growth rate), `Event` (phase transitions)
- **Interaction**: Click any gene → expression, translation, essentiality
  status across all 28 submodels simultaneously

### Paper C → Genome Exploration Surface

3D fitness landscape where each point is a genome design. Color = viability.
Watch the GAMA population evolve through the 53,451-genome search space.

- **DataBinding channels**: `Grid2D` (fitness landscape), `Trajectory`
  (algorithm path), `Categorical` (gene essentiality per design)

### Paper E → 3D Living Cell

Particles move and react in real time inside a 3D cell geometry.
Ribosomes translate, mRNA diffuses, metabolites concentrate. The 3D
view IS the simulation.

- **DataBinding channels**: `Spatial3D` (particle positions), `TimeSeries`
  (metabolites), `Histogram` (molecular distributions), `Event` (reactions)
- **Interaction**: Pause/rewind. Select species → highlight all instances.
  Select region → local concentrations. Toggle scales.

### Paper F → Molecular Cinema

561 million CG beads rendered in real time (LOD). Membrane undulates.
Proteins diffuse. Ribosomes cluster near mRNA.

- **DataBinding channels**: `Spatial3D` (bead positions, LOD-streamed),
  `TimeSeries` (diffusion coefficients), `Density` (concentration fields),
  `Surface` (membrane shape)
- **Interaction**: Slice at any plane. Zoom from whole-cell to single-protein.
  Time slider across 200 ns trajectory.

### Paper G → The Cell Cycle Film

Birth → growth → DNA replication → chromosome segregation → constriction
→ division → two daughters. The entire cell cycle as a continuous,
interactive 4D rendering.

- **DataBinding channels**: All channels from Papers E and F, plus
  `Morphology` (cell shape evolution), `Polymer` (chromosome configuration),
  `Timeline` (cell cycle phase progression)
- **Interaction**: Scrub through the cycle. Compare mother and daughter.
  Overlay metabolic state on morphology. Watch FtsZ ring form in 3D.

---

## 8. Evolution Targets

What NUCLEUS needs to evolve to fully express these workflows.

### New toadStool Workload Types

| Workload | Paper | Priority |
|----------|-------|----------|
| `lattice_microbes` — C++/CUDA spatial stochastic dispatch with GPU allocation | E, G | High |
| `gromacs_md` — GROMACS as external binary with trajectory capture | F, G | High |
| `matlab_interop` — MATLAB runtime dispatch for legacy model (bridge to Rust port) | A, C | Medium |
| `genetic_algorithm` — Parallelizable genome search | C | Medium |

### New NestGate Storage Patterns

| Pattern | Paper |
|---------|-------|
| MD trajectory frames — per-frame BLAKE3 for XTC/TRR binary trajectories | F, G |
| Large simulation state — chunked, content-addressed GB-scale grid snapshots | E, G |
| Genome library — 53,451 designs, each content-addressed with viability metadata | C |
| Database snapshots — version-pinned parameter sets (BRENDA v2024.1, not just "BRENDA") | All |

### New barraCuda Kernels

| Kernel | Paper |
|--------|-------|
| CG force field evaluation (Martini 3 nonbonded + bonded on GPU, WGSL) | F, G |
| Reaction-diffusion RDME (spatial stochastic on cubic lattice) | E, G |
| FBA linear programming solver | A |
| Gillespie SSA on GPU (exact and tau-leaping) | A, E |
| Chromosome polymer dynamics (bead-spring + motor forces) | G |
| Membrane shape solver (Helfrich energy minimization) | G |

### New petalTongue Channels

| Channel | Paper |
|---------|-------|
| `Spatial3D` — 3D molecular/particle visualization with LOD streaming | E, F, G |
| `Morphology` — deformable cell shape evolution | G |
| `Polymer` — chromosome bead-spring rendering | G |
| `FitnessLandscape` — navigable 2D/3D fitness landscape | C |

---

## 9. The Publication Argument

### What Gets Published

1. **This document** (CC-BY-SA 4.0) — analysis, composition blueprints,
   provenance audit. A methods paper.
2. **The rebuild itself** (AGPL-3.0) — deploy graph TOMLs, spring
   experiments, provenance DAGs.
3. **Provenance-complete reproductions** — for each paper, every parameter
   content-addressed, every step DAG-recorded, every result in NestGate.
4. **Live computation surfaces** — petalTongue renderings as interactive
   figures. Screenshots become paper figures.
5. **The provenance audit** — machine-readable inventory of every jelly
   string with its structural solution. Independently valuable.

### The Handoff Vision

When complete, this foundation enables:

- A researcher downloads the deploy graph TOML
- They run it on their own NUCLEUS (sovereign hardware)
- The entire whole-cell modeling lineage rebuilds on their machine
- Every parameter is traceable to its source
- Every computation is reproducible
- Every figure is a live, interactive view they can explore
- They modify a parameter → provenance captures the modification
- They publish the extension as a new sweetGrass braid
- The original authors receive attribution credit automatically

The data has integrity. The provenance is structural. The science travels.

---

*The original authors built the whole-cell model on jelly strings and
trust-me data because those were the only tools available. The strings
held — the science is real, the papers are cited, the field advanced.
This foundation does not replace their work. It rebuilds it with the
structural provenance they deserved from the start, and creates a
platform where the next 14 years of whole-cell modeling can stand on
verified ground.*
