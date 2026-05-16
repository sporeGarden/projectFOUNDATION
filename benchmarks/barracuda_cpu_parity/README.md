# barraCuda CPU Parity Baselines

Python (scipy/numpy) reference implementations for operations that
barraCuda implements in pure Rust + WGSL GPU compute.

These baselines serve as ground truth for numerical parity verification:
- Rust CPU results must match Python within documented tolerances
- GPU results must match Rust CPU within IEEE 754 bounds
- Any deviation must be named, justified, and minimal

## Operations covered

| Baseline | barraCuda equivalent | Parity standard |
|----------|---------------------|-----------------|
| `stats_variance.py` | `VarianceF64` kernel | rel_err < 1e-6 (Welford vs two-pass) |
| `md_velocity_verlet.py` | `velocity_verlet_split_f64.wgsl` | relative energy drift < 5% |
| `spectral_eigenvalues.py` | Anderson localization eigensolver | max eigenvalue diff < 1e-10 |

## Setup

```bash
pip install -r benchmarks/barracuda_cpu_parity/requirements.txt
```

## Running

```bash
# Run each baseline individually:
python3 benchmarks/barracuda_cpu_parity/stats_variance.py
python3 benchmarks/barracuda_cpu_parity/md_velocity_verlet.py
python3 benchmarks/barracuda_cpu_parity/spectral_eigenvalues.py
```

Each script writes a `*_results.json` file with full numerical outputs
and pass/fail status. Re-run to refresh results with current Python/NumPy
versions — compare JSON output to verify no baseline drift.

## Industry references

- Kokkos: parallel dispatch model (barraCuda uses wgpu compute equivalent)
- LAMMPS: MD force/integration parity (Sarkas OCP Yukawa regime)
- SciPy: statistical functions, eigenvalue decomposition
- cuBLAS: BLAS operation semantics (not FFI — pure Rust reimplementation)
