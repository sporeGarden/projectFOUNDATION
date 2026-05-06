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

---

## 2. The Paper Lineage

Seven publications spanning 2012–2026:

### Paper A — Whole-Cell Computational Model Predicts Phenotype from Genotype

- **Citation**: Karr JR et al. (2012) Cell 150(2):389–401
- **Organism**: *Mycoplasma genitalium* (525 genes)
- **Contribution**: First comprehensive whole-cell model — 28 coupled submodels
- **Data sources**: ~1,900 parameters from >900 publications (BRENDA, EcoCyc, KEGG, UniProt)
- **Key results**: Predicts cell cycle duration (~9 hours), growth rate, gene essentiality (79% match)

### Paper B — Whole-Cell Models and Simulations in Molecular Detail

- **Citation**: Feig M, Sugita Y (2019) Annu Rev Cell Dev Biol 35:191–211
- **Contribution**: Review surveying physical/atomistic approaches. Identifies the gap between kinetic models (Paper A) and all-atom structural models.
- **NUCLEUS relevance**: Frames the multi-scale composition problem that Papers E–G attempt

### Paper C — Designing Minimal Genomes Using Whole-Cell Models

- **Citation**: Rees-Garbutt J et al. (2020) Nat Commun 11:836
- **Organism**: *M. genitalium* (using Paper A model)
- **Contribution**: Algorithmic genome minimization — Minesweeper and GAMA algorithms
- **Key results**: GAMA explored 53,451 genomes, removed up to 237 genes (45%)
- **Parameter inheritance**: Directly inherits Paper A's 1,900+ parameters

### Paper D — Fundamental Behaviors Emerge from Simulations of a Living Minimal Cell

- **Citation**: Thornburg ZR et al. (2022) Cell 185:345–360
- **Organism**: *JCVI-syn3A* (493 genes)
- **Contribution**: First kinetic model of a near-minimal synthetic cell
- **Data sources**: BRENDA kinetics, genetic essentiality (Hutchison et al. 2016), proteomics (Breuer et al. 2019)
- **Key results**: DNA replication timing, cell cycle 111 min, correct macromolecular mass ratios

### Paper E — Simulating a Living Cell at Molecular Resolution

- **Citation**: Stevens JA et al. (2023) Cell (submitted/preprint)
- **Organism**: *JCVI-syn3A*
- **Contribution**: All-atom molecular dynamics — 3.2 billion atoms, entire cell as a GPU MD simulation
- **Data sources**: CG model from Paper D, PDB/UniProt structural data, CHARMM36 force field
- **Key results**: Complete metabolic network running at molecular resolution

### Paper F — Spatiotemporal Modeling of the Crowded Intracellular Environment

- **Citation**: Stevens et al. (companion to Paper E)
- **Contribution**: Multi-scale integration — coarse-grained (CG) to all-atom (AA) bridging
- **Key methods**: NAMD/GENESIS GPU-MD, CG-to-AA backmapping, reaction-diffusion coupling

### Paper G — 4D Cell Cycle Simulation

- **Citation**: Thornburg ZR et al. (2026) Cell
- **Organism**: *JCVI-syn3A*
- **Contribution**: Complete spatiotemporal cell cycle: DNA replication, membrane growth, FtsZ ring, septation, division
- **Data sources**: Integrates Papers A/D/E/F, new cryo-ET structural data, FtsZ polymerization kinetics
- **Key results**: First 4D (xyz + time) division simulation from molecular to cellular scale

---

## 3. The Jelly Strings

"Jelly strings" = provenance gaps in the original work that NUCLEUS can
structurally fill.

| Gap | Papers | NUCLEUS Solution |
|-----|--------|-----------------|
| Parameter provenance | All | NestGate content-addresses every parameter with BLAKE3. Provenance trio traces inheritance across all 7 papers. |
| Cross-organism borrowing | A → D, D → E | sweetGrass attribution braids record exactly which parameters were borrowed from which organism/paper. |
| Scale bridging | E → F → G | toadStool dispatches multi-scale coupled simulations. Deploy graphs encode the coupling topology. |
| Toolchain reproducibility | All (MATLAB → Python → C++) | Sovereign Rust/WGSL pipeline replaces vendor-locked toolchains. Single binary per primal. |
| Figure-computation disconnect | All | petalTongue renders figures as live computation surfaces. The figure IS the computation. |
| Version pinning | E (CHARMM36), G (cryo-ET structures) | NestGate stores force field versions and structural data with content-addressed integrity. |

---

## 4. Data Targets

The data sources and validation targets for this thread are maintained
in machine-readable TOML manifests:

- **Sources**: `data/sources/thread01_wcm.toml` — NCBI accessions, BRENDA
  enzyme IDs, PDB structure IDs, UniProt protein IDs
- **Targets**: `data/targets/thread01_wcm_targets.toml` — expected numerical
  results from each paper (cell cycle duration, growth rate, gene essentiality
  match percentage, macromolecular composition)

These manifests are populated as spring experiments validate each paper's
results. The manifests evolve alongside the validation work.

---

## 5. NUCLEUS Composition Blueprints

### Paper A Composition (28 Submodels)

**Minimum viable atomics**: Node (toadStool + barraCuda for ODE dispatch
and GPU math) + Nest (NestGate + provenance trio for parameter storage
and tracking).

| NUCLEUS Component | Paper A Role |
|------------------|-------------|
| toadStool | Dispatches 28 submodel steps per time increment |
| barraCuda | GPU-accelerates ODE integration, Gillespie SSA |
| NestGate | Content-addresses 1,900+ parameters, stores time-series output |
| rhizoCrypt | DAG session per simulation run |
| loamSpine | Permanent ledger entry per completed run |
| sweetGrass | Attribution braid: Karr → parameters → our reproduction |

### Papers E/F/G Composition (Multi-Scale MD)

**Minimum viable atomics**: Full NUCLEUS (Tower + Node + Nest) for
cross-gate dispatch of MD workloads.

| NUCLEUS Component | Papers E/F/G Role |
|------------------|------------------|
| toadStool | Dispatches CG and AA simulations across GPU fleet |
| barraCuda | WGSL shaders for NAMD/GENESIS-equivalent force evaluation |
| coralReef | Shader compilation for heterogeneous GPU architectures |
| NestGate | Content-addresses trajectory frames, force field versions, cryo-ET structures |
| Provenance trio | Tracks parameter flow from Paper D kinetics through CG model to AA simulation |
| petalTongue | Renders 4D cell cycle as live interactive visualization |

---

## 6. Spring Alignment

| Spring | Contribution to Thread 1 |
|--------|-------------------------|
| hotSpring | MD methods, GPU dispatch patterns, Murillo transport validation |
| wetSpring | Genomics pipelines for gene annotation (Papers A/D), NCBI integration |
| healthSpring | ODE solver patterns shared with metabolic modeling (Paper A/D kinetics) |
| neuralSpring | ML-assisted parameter estimation, reservoir computing for time-series |

---

## 7. petalTongue Vision

The whole-cell rebuild is the most compelling petalTongue demonstration:

- **Paper A**: 28 submodel state as a live dashboard — metabolite
  concentrations, gene expression, cell cycle progress
- **Papers E/G**: 4D cell rendered in real time from simulation data —
  membrane growth, FtsZ ring formation, DNA replication forks, septation
- **Cross-paper**: Parameter inheritance graph — which constants flow
  from Karr 2012 through 14 years of papers to Thornburg 2026

The computation IS the presentation. No post-hoc figure generation.

---

## 8. scyBorg Publication

The rebuild produces a scyBorg-licensed publication:
- DAG-recorded (rhizoCrypt session per paper rebuild)
- BLAKE3-hashed (every parameter, every result)
- ed25519-witnessed (loamSpine permanent ledger)
- Attribution-braided (sweetGrass traces from original authors through
  our reproduction to any downstream use)

This is the first instantiation of the sovereign publication pipeline
against external science. It proves the pipeline works for real papers
by real scientists in real journals.
