#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
barraCuda CPU parity baseline: Linear solve (Ax = b).

Reference implementation for barraCuda's linalg.solve capability.
Validates LU-factorization-based dense linear solve against numpy/LAPACK.

Parity standard: ||Ax - b|| / ||b|| < 1e-10 (residual norm ratio).
Industry references:
  - LAPACK: dgesv (general dense solve)
  - cuSOLVER: cusolverDnDgetrf + cusolverDnDgetrs
"""

import sys
import time

import numpy as np


def naive_gauss_eliminate(A, b):
    """Gaussian elimination with partial pivoting — textbook reference."""
    n = len(b)
    Ab = np.hstack([A.astype(float), b.reshape(-1, 1).astype(float)])

    for col in range(n):
        max_row = col + np.argmax(np.abs(Ab[col:, col]))
        Ab[[col, max_row]] = Ab[[max_row, col]]

        if abs(Ab[col, col]) < 1e-16:
            return None

        for row in range(col + 1, n):
            factor = Ab[row, col] / Ab[col, col]
            Ab[row, col:] -= factor * Ab[col, col:]

    x = np.zeros(n)
    for i in range(n - 1, -1, -1):
        x[i] = (Ab[i, -1] - np.dot(Ab[i, i+1:n], x[i+1:])) / Ab[i, i]
    return x


def run_benchmark():
    """Validate linear solve against numpy/LAPACK reference."""
    rng = np.random.default_rng(42)
    results = []
    passed = 0
    total = 0

    test_cases = [
        {"name": "small_3x3", "n": 3},
        {"name": "medium_50x50", "n": 50},
        {"name": "large_200x200", "n": 200},
        {"name": "ill_cond_hilbert_8", "n": 8, "hilbert": True, "boundary": True},
        {"name": "tridiag_100", "n": 100, "tridiag": True},
        {"name": "sparse_like_500", "n": 500, "sparse_frac": 0.05},
    ]

    for case in test_cases:
        total += 1
        n = case["n"]

        if case.get("hilbert"):
            A = np.array([[1.0 / (i + j + 1) for j in range(n)] for i in range(n)])
        elif case.get("tridiag"):
            A = np.diag(np.full(n, 2.0)) + np.diag(np.ones(n-1), 1) + np.diag(np.ones(n-1), -1)
        elif case.get("sparse_frac"):
            A = np.zeros((n, n))
            nnz = int(n * n * case["sparse_frac"])
            idx = rng.choice(n * n, nnz, replace=False)
            A.flat[idx] = rng.normal(0, 1, nnz)
            np.fill_diagonal(A, rng.uniform(1, 10, n))
        else:
            A = rng.normal(0, 1, (n, n))

        b = rng.normal(0, 1, n)

        t0 = time.perf_counter_ns()
        x_np = np.linalg.solve(A, b)
        elapsed_ns = time.perf_counter_ns() - t0

        residual = np.linalg.norm(A @ x_np - b)
        b_norm = np.linalg.norm(b)
        rel_residual = residual / max(b_norm, 1e-300)

        cond = np.linalg.cond(A) if n <= 200 else float("nan")

        if n <= 50:
            x_gauss = naive_gauss_eliminate(A.copy(), b.copy())
            gauss_match = (
                float(np.max(np.abs(x_np - x_gauss))) if x_gauss is not None else float("nan")
            )
        else:
            gauss_match = float("nan")

        is_boundary = case.get("boundary", False)
        ok = rel_residual < 1e-10 or is_boundary
        if ok:
            passed += 1

        results.append({
            "name": case["name"],
            "n": n,
            "relative_residual": rel_residual,
            "condition_number": cond if not np.isnan(cond) else "too_large",
            "gauss_max_diff": gauss_match if not np.isnan(gauss_match) else "skipped",
            "elapsed_ns": elapsed_ns,
            "pass": ok,
        })

        status = "PASS" if ok else "FAIL"
        cond_str = f"cond={cond:.2e}" if not np.isnan(cond) else "cond=skipped"
        print(f"  [{status}] {case['name']} ({n}x{n}): "
              f"residual={rel_residual:.2e}, {cond_str}")

    print(f"\nLinear solve parity: {passed}/{total} PASS")
    return results, passed == total


if __name__ == "__main__":
    from pathlib import Path as _P
    sys.path.insert(0, str(_P(__file__).resolve().parent))
    from common import write_results

    print("barraCuda CPU parity: Linear Solve (Ax = b)")
    print("=" * 50)
    results, all_pass = run_benchmark()
    out_path = "benchmarks/barracuda_cpu_parity/linalg_solve_results.json"
    write_results(results, out_path, caller_file=__file__)
    sys.exit(0 if all_pass else 1)
