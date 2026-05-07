# Measurement Science — Threads 6 & 7 Expression

**Springs**: groundSpring (primary), airSpring, wetSpring, neuralSpring, hotSpring
**Threads**: 6 (Agricultural Science), 7 (Anderson Mathematics)
**Last Updated**: May 7, 2026
**Status**: groundSpring complete (first spring done) — 395/395 checks, 29 notebooks

## The Core Question

How do things actually look, and why is it different from what we expected?

Every measurement is a sum of signal and noise. groundSpring decomposes
that sum across 10 scientific domains — from soil moisture sensors to
Anderson localization to quorum sensing — and proves that the decomposition
is faithful, deterministic, and GPU-acceleratable.

## Five Pillars

| Pillar | What it measures | Key experiments |
|--------|-----------------|-----------------|
| **Signal vs Noise** | Bias-variance decomposition | 001, 002, 003, 015, 024 |
| **Inverse Problems** | Observations → causes | 005, 019, 020, 021 |
| **Sensing Systems** | Instrument distortion | 006, 010, 011, 028 |
| **Temporal Dynamics** | System drift over time | 014, 016, 017, 033 |
| **Spatial Propagation** | Signal through media | 008, 009, 012, 018 |

## The Anderson Thread (Cross-Cutting)

Anderson localization — how disorder traps waves — is the mathematical
backbone. It connects:

- **Condensed matter** (Exp 008, 009, 012, 018): Kachkovskiy (MSU Math)
- **Measurement science** (Exp 015, 022): Uncertainty bridge, ET₀→ξ
- **Immunology** (Exp 033): Gonzales — cytokine signaling through inflamed skin
- **Neuromorphic** (Exp 028): BrainChip AKD1000 regime classification
- **Soil science** (Exp 024): Aggregate stability as disorder

The disorder parameter W maps to:
- Soil heterogeneity (Thread 6)
- Lattice impurities (Thread 2)
- Signal interference (Thread 4)
- Cytokine scattering (Thread 3)

## Validated Targets (29/29)

groundSpring is the **first spring with all baselines complete**:

| Domain | Experiments | Checks | Speedup range |
|--------|-------------|--------|---------------|
| Measurement | 001, 002, 003, 015, 022, 024 | 87/87 | 8.2–14.1× |
| Biological | 004, 006, 010, 011, 014, 016, 017 | 81/81 | 6.9–46.2× |
| Condensed Matter | 008, 009, 012, 018 | 44/44 | 12.3–49.5× |
| Statistics | 007, 013 | 19/19 | 5.8–7.3× |
| Inverse Problems | 019, 020, 021 | 25/25 | 3.8–7.6× |
| Geophysics | 005, 032 | 21/21 | 4.2× |
| Hydrology | 003, 035 | 34/34 | 9.4× |
| Soil Science | 023, 024 | 15/15 | 6.1–9.3× |
| WDM/Numerical | 025, 026, 027 | 21/21 | 2.1–4.5× |
| Neuromorphic | 028 | 9/9 | N/A |

**Total**: 395/395 checks, 29/29 math parity proven, 965 Rust tests passing.

## Faculty Network

| Faculty | Institution | Experiments | Thread |
|---------|------------|-------------|--------|
| Alexei Bazavov | CMSE + Physics, MSU | 019, 020, 021 | 2, 7 |
| Christopher Waters | MMG, MSU | 006, 010, 011 | 4, 5 |
| Kevin Liu | CMSE, MSU | 007 | 6 |
| Ilya Kachkovskiy | Math, MSU | 008, 009, 012, 018 | 7 |
| Rika Anderson | Biology, Carleton | 014, 016 | 4, 5 |
| Emily Dolson | CSE, MSU | 017 | 5 |
| Andrea J. Gonzales | Pharm/Tox, MSU | Paper 12 (008, 012, 015, 018) | 3, 7 |

## Primal Composition

groundSpring consumes 5 primals via IPC:

| Primal | Role | Capabilities |
|--------|------|-------------|
| beardog | Security | crypto.sign, crypto.verify |
| songbird | Discovery | discovery.find_primals, discovery.query |
| toadstool | Compute | compute.execute, compute.submit |
| nestgate | Data + Storage | storage.put/get, data.ncbi/noaa/iris |
| barracuda | Math | 110 delegations (67 CPU + 43 GPU) |

Provides 16 `measurement.*` capabilities to the ecosystem.

## Workloads

| Workload | What it runs |
|----------|-------------|
| `gs-validate-all` | All 29 Rust validators (395 checks) |
| `gs-guidestone` | guideStone Level 3 (5 bare + 6 IPC) |
| `gs-bench-gpu` | Three-mode GPU benchmark |
| `gs-python-baselines` | All 29 Python baselines for provenance |

## Sediment Contribution

groundSpring deposits the **measurement noise layer** — the floor that
all other springs build on. When airSpring computes ET₀, it inherits
groundSpring's error propagation bounds. When wetSpring classifies
microbial communities, it uses groundSpring's rarefaction noise floor.
When hotSpring solves inverse problems, it uses groundSpring's jackknife
and spectral reconstruction methods.

## Notebooks

29 publication-grade Python baseline notebooks in `notebooks/baselines/`
plus 5 sporePrint summary notebooks in `notebooks/`.

## What's Next

- **Thread 6 data sources**: Wire NOAA GHCN-Daily + NCBI SRA + IRIS FDSN
  into `fetch_sources.sh` with `--thread ag`
- **Thread 7 Anderson sources**: Literature references already mapped;
  numerical targets are all validated
- **Sediment Layer 2**: Execute `foundation_validate.sh --thread ag` once
  NUCLEUS composition is deployed on ironGate
- **Cross-thread metamorphism**: Anderson results overlap Threads 2, 3, 4,
  5 — meta-validation when those springs complete their baselines
