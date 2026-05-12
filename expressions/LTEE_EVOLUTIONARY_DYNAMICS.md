# LTEE Evolutionary Dynamics — Thread 5 Expression

**Springs**: groundSpring (primary), neuralSpring, wetSpring, hotSpring
**Thread**: 5 (Evolutionary Biology / LTEE)
**Last Updated**: May 11, 2026
**Status**: groundSpring B1-B3 COMPLETE, hotSpring B2 COMPLETE, neuralSpring B1 Python DONE, wetSpring B7 STARTED

## The Core Question

Can we reproduce 75,000+ generations of bacterial evolution computationally,
matching the quantitative results of the longest-running evolution experiment
in history?

The Long-Term Evolution Experiment (LTEE), started by Richard Lenski in 1988,
tracks 12 populations of *E. coli* through continuous serial transfer. It has
produced definitive evidence for:

- Fitness trajectories that follow power-law dynamics (Wiser et al. 2013)
- Neutral mutation rates consistent with molecular clock models (Barrick et al. 2009)
- Clonal interference shaping adaptive dynamics (Good et al. 2017)
- Genomic parallelism across replicate populations (Tenaillon et al. 2016)

## Reproduction Papers (Spring Assignments)

| ID | Paper | Key Result | Spring | Status |
|----|-------|-----------|--------|--------|
| B1 | Barrick et al. 2009 | Neutral mutation accumulation rate | groundSpring | **COMPLETE** (Py 8/8 + Rust 8/8) |
| B2 | Wiser et al. 2013 | Power-law fitness trajectory | groundSpring, hotSpring | **COMPLETE** (Py 9/9 + Rust 10/10; hotSpring Tier 1+2) |
| B3 | Good et al. 2017 | Clonal interference dynamics | groundSpring | **COMPLETE** |
| B7 | Tenaillon et al. 2016 | 264-genome mutation accumulation | wetSpring | **STARTED** (sovereign pipeline) |
| B1-ML | Barrick mutation (ML surrogate) | LSTM/ESN/HMM mutation forecasting | neuralSpring | **Python baseline DONE** (8/8) |

## Five Dimensions of LTEE Validation

| Dimension | What It Measures | Key Experiments |
|-----------|-----------------|-----------------|
| **Fitness dynamics** | Population-level adaptation rate | B2 power-law fits, Malthusian relative fitness |
| **Mutation accumulation** | Molecular clock neutrality | B1 synonymous vs nonsynonymous rates |
| **Clonal dynamics** | Competition between lineages | B3 interference coefficients |
| **Genomic parallelism** | Convergent evolution across replicates | B7 264-genome alignment |
| **Predictability** | ML forecasting of evolutionary trajectories | B1-ML surrogate models |

## Data Flow

```
Dryad/NCBI accessions
    │
    ├── groundSpring: fetch → BLAKE3 hash → Python baseline → Rust validation
    │     └── control/ltee_*/expected_values.json
    │
    ├── hotSpring: Anderson RMT analysis → Rust scenario → notebook
    │     └── experiments/results/ltee/ltee_b2_anderson_expected.json
    │
    ├── neuralSpring: LSTM/ESN/HMM training → Python baseline → (Rust next)
    │     └── control/ltee_mutation_accumulation/
    │
    └── wetSpring: 264-genome sovereign pipeline → mutation curves
          └── experiments/ltee/ (in progress)
```

## Connection to lithoSpore

lithoSpore modules 1-3 and 6-7 directly consume Thread 5 outputs:

| lithoSpore Module | Thread 5 Input | Source Spring |
|-------------------|---------------|---------------|
| ltee-mutation (1) | B1 expected_values.json | groundSpring |
| ltee-fitness (2) | B2 expected_values.json | groundSpring |
| ltee-clonal (3) | B3 outputs | groundSpring |
| ltee-genomics (6) | B7 264-genome pipeline | wetSpring |
| ltee-anderson (7) | B2 Anderson RMT analysis | hotSpring |

## What Remains

- **wetSpring B7**: 264-genome pipeline in progress — completion feeds module 6
- **neuralSpring Rust binary**: Python baseline done, Rust validation binary not yet
- **Data anchoring**: spring outputs need BLAKE3 hashing into NestGate content storage
- **Foundation targets**: validation targets below need `validated = true` + BLAKE3 hashes as springs complete
