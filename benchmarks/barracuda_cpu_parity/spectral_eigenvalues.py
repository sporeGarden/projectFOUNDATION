#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
barraCuda CPU parity baseline: Spectral eigenvalue computation.

Reference implementation for barraCuda's Anderson localization
eigensolver (tridiagonal + dense paths). Validates against scipy's
LAPACK-backed routines.

Parity standard: 1e-10 absolute error (LAPACK dstev/dsyev reference).
Industry references:
  - LAPACK: dstev (tridiagonal), dsyev (dense symmetric)
  - Anderson 1958: tight-binding 1D model eigenstates
  - groundSpring Exp 008: Lyapunov exponent validation
"""

import sys
import time

import numpy as np
import scipy
from scipy import linalg


def anderson_hamiltonian_1d(n, disorder_w, seed=42):
    """1D tight-binding Anderson Hamiltonian with uniform disorder."""
    rng = np.random.default_rng(seed)
    diagonal = rng.uniform(-disorder_w / 2, disorder_w / 2, n)
    off_diag = np.ones(n - 1)
    return np.diag(diagonal) + np.diag(off_diag, 1) + np.diag(off_diag, -1)


def tridiagonal_eigenvalues(diagonal, off_diagonal):
    """Tridiagonal eigenvalue computation (scipy LAPACK wrapper)."""
    return linalg.eigh_tridiagonal(diagonal, off_diagonal, eigvals_only=True)


def ipr(eigvec):
    """Inverse Participation Ratio — measures localization."""
    return float(np.sum(eigvec**4))


def lyapunov_exponent(hamiltonian, energy=0.0):
    """Transfer matrix Lyapunov exponent at given energy."""
    n = hamiltonian.shape[0]
    gamma_sum = 0.0
    v_prev, v_curr = np.array([0.0, 1.0]), np.array([1.0, 0.0])

    for i in range(n):
        eps_i = hamiltonian[i, i]
        transfer = np.array([[energy - eps_i, -1.0], [1.0, 0.0]])
        v_next = transfer @ v_curr
        norm = np.linalg.norm(v_next)
        if norm > 0:
            gamma_sum += np.log(norm)
            v_next /= norm
        v_prev, v_curr = v_curr, v_next

    return gamma_sum / n if n > 0 else 0.0


def run_benchmark():
    """Validate spectral computations against LAPACK reference."""
    results = []
    passed = 0
    total = 0

    test_cases = [
        {"name": "1d_w2_n100", "n": 100, "w": 2.0, "seed": 42},
        {"name": "1d_w4_n500", "n": 500, "w": 4.0, "seed": 123},
        {"name": "1d_w8_n1000", "n": 1000, "w": 8.0, "seed": 7},
        {"name": "1d_w16_n200", "n": 200, "w": 16.0, "seed": 99},
    ]

    for case in test_cases:
        total += 1
        n, w, seed = case["n"], case["w"], case["seed"]

        t0 = time.perf_counter_ns()
        H = anderson_hamiltonian_1d(n, w, seed)

        eigenvalues_dense = np.sort(np.linalg.eigvalsh(H))
        diagonal = np.diag(H)
        off_diagonal = np.diag(H, 1)
        eigenvalues_tri = tridiagonal_eigenvalues(diagonal, off_diagonal)
        elapsed_ns = time.perf_counter_ns() - t0

        max_diff = float(np.max(np.abs(eigenvalues_dense - eigenvalues_tri)))

        _, eigvecs = np.linalg.eigh(H)
        mid_idx = n // 2
        mid_ipr = ipr(eigvecs[:, mid_idx])

        gamma = lyapunov_exponent(H, energy=0.0)

        ok = max_diff < 1e-10
        if ok:
            passed += 1

        results.append({
            "name": case["name"],
            "n": n,
            "disorder_w": w,
            "max_eigenvalue_diff": max_diff,
            "band_center_ipr": mid_ipr,
            "lyapunov_exponent": gamma,
            "elapsed_ns": elapsed_ns,
            "pass": ok,
        })

        status = "PASS" if ok else "FAIL"
        print(f"  [{status}] {case['name']}: max_diff={max_diff:.2e}, "
              f"γ={gamma:.4f}, IPR={mid_ipr:.4f}")

    # Additional: verify Lyapunov scaling (γ ~ W² for small W)
    total += 1
    gammas = []
    ws = [0.5, 1.0, 2.0, 4.0]
    for w_val in ws:
        H = anderson_hamiltonian_1d(2000, w_val, seed=42)
        gammas.append(lyapunov_exponent(H, 0.0))

    log_w = np.log(ws)
    log_g = np.log(np.abs(gammas))
    slope = float(np.polyfit(log_w, log_g, 1)[0])
    scaling_ok = 1.5 < slope < 2.5
    if scaling_ok:
        passed += 1

    results.append({
        "name": "lyapunov_scaling",
        "ws": ws,
        "gammas": gammas,
        "fitted_exponent": slope,
        "expected_range": [1.5, 2.5],
        "pass": scaling_ok,
    })
    print(f"  [{'PASS' if scaling_ok else 'FAIL'}] Lyapunov scaling: "
          f"exponent={slope:.3f} (expected ~2.0)")

    print(f"\nSpectral eigenvalue parity: {passed}/{total} PASS")
    return results, passed == total


if __name__ == "__main__":
    from pathlib import Path as _P
    sys.path.insert(0, str(_P(__file__).resolve().parent))
    from common import write_results

    print("barraCuda CPU parity: Spectral Eigenvalues (Anderson Localization)")
    print("=" * 65)
    results, all_pass = run_benchmark()
    out_path = "benchmarks/barracuda_cpu_parity/spectral_eigenvalues_results.json"
    write_results(results, out_path, caller_file=__file__,
                  provenance_kw={"extra_versions": {"scipy": scipy.__version__}})
    sys.exit(0 if all_pass else 1)
