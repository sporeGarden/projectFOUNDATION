# barraCuda CPU Parity Baselines — Tier 1→2 Parity Proofs

Python (scipy/numpy) reference implementations for operations that
barraCuda implements in pure Rust + WGSL GPU compute.

These baselines are **Tier 1→2 parity proofs** per the ecosystem
validation tier model (see `primalSpring/docs/VALIDATION_TIERS.md`):

```
Tier 1: Python script → expected results (this directory)
Tier 2: Rust validator → compare against Python expected results
Tier 3: Primal composition → verify computation chain via provenance trio
```

Parity verification rules:
- Rust CPU results must match Python within documented tolerances
- GPU results must match Rust CPU within IEEE 754 bounds
- Any deviation must be named, justified, and minimal
- Results are exported as JSON for consumption by lithoSpore's
  `ParityReport` format (ecosystem standard for cross-tier parity)

## Operations Covered

| Baseline | barraCuda equivalent | Parity standard | Cases |
|----------|---------------------|-----------------|-------|
| `stats_variance.py` | `VarianceF64` kernel | rel_err < 1e-6 (Welford vs two-pass) | 6 |
| `stats_mean.py` | `stats.mean` capability | rel_err < 1e-14 (Kahan compensated) | 6 |
| `md_velocity_verlet.py` | `velocity_verlet_split_f64.wgsl` | relative energy drift < 5% | 3 |
| `spectral_eigenvalues.py` | Anderson localization eigensolver | max eigenvalue diff < 1e-10 | 5 |
| `linalg_solve.py` | `linalg.solve` capability | relative residual < 1e-10 | 6 |
| `matmul.py` | `tensor.matmul` capability | element-wise rel_err < 1e-10 | 6 |

**Total**: 32 test cases across 6 baselines, all executed in CI.

## Remaining Coverage Gaps

Operations referenced in expressions but not yet baselined:

| Missing baseline | Referenced in | barraCuda surface |
|-----------------|---------------|-------------------|
| FFT / spectral | PLASMA_QCD_SOVEREIGN_GPU.md | `spectral.*` |
| Sigmoid / activation | GAMING_CREATIVE_SCIENCE.md | `math.sigmoid` |
| Random number generation | GAMING_CREATIVE_SCIENCE.md | `rng.*` |

These are covered by barraCuda's internal test suite but lack independent
Python reference baselines in projectFOUNDATION.

## GPU Parity Status

No GPU parity benchmarks exist in projectFOUNDATION. GPU benchmarks live
in the barraCuda repo:

| Bench | Location | Compares against |
|-------|----------|-----------------|
| `kokkos_parity.rs` | `primals/barraCuda/crates/barracuda/benches/` | VarianceF64 GPU timing vs Kokkos reference numbers |
| `lammps_parity.rs` | same | LJ + Yukawa at LAMMPS-scale N vs published throughput |
| `scipy_parity.rs` | same | SumReduce, Variance, cdist vs SciPy timing notes |

**Galaxy benchmarks**: Not present in either repo. barraCuda STATUS.md
confirms no Galaxy benchmarks planned.

**Kokkos hardware parity**: barraCuda bench comments reference Kokkos/CUDA
published numbers but do not run Kokkos binaries side-by-side. Full
hardware-matched Kokkos parity is tracked as future work awaiting matching
GPU hardware.

## Setup

```bash
pip install -r benchmarks/barracuda_cpu_parity/requirements.txt
```

## Running

```bash
python3 benchmarks/barracuda_cpu_parity/stats_mean.py
python3 benchmarks/barracuda_cpu_parity/stats_variance.py
python3 benchmarks/barracuda_cpu_parity/matmul.py
python3 benchmarks/barracuda_cpu_parity/linalg_solve.py
python3 benchmarks/barracuda_cpu_parity/md_velocity_verlet.py
python3 benchmarks/barracuda_cpu_parity/spectral_eigenvalues.py
```

Each script writes a `*_results.json` file with full numerical outputs,
provenance metadata, and pass/fail status.

## Industry References

- **Kokkos**: Parallel dispatch model (barraCuda uses wgpu compute equivalent)
- **LAMMPS**: MD force/integration parity (Sarkas OCP Yukawa regime)
- **SciPy**: Statistical functions, eigenvalue decomposition (LAPACK dstev/dsyev)
- **cuBLAS**: BLAS operation semantics (not FFI — pure Rust reimplementation)
- **Galaxy**: Not benchmarked — different domain (astrophysics N-body)
