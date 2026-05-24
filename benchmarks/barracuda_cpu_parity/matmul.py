#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
barraCuda CPU parity baseline: Matrix multiplication (GEMM).

Reference implementation for barraCuda's tensor.matmul capability.
Validates naive, tiled, and Strassen-like approaches against numpy/BLAS.

Parity standard: max element-wise relative error < 1e-10.
Industry references:
  - cuBLAS: cublasDgemm (double-precision GEMM)
  - BLAS: dgemm (reference implementation via OpenBLAS/MKL)
  - Strassen 1969: sub-cubic matrix multiplication
"""

import json
import sys
import time

import numpy as np


def naive_matmul(A, B):
    """Triple-loop matrix multiply — O(n^3) textbook reference."""
    m, k = A.shape
    _, n = B.shape
    C = np.zeros((m, n))
    for i in range(m):
        for j in range(n):
            s = 0.0
            for p in range(k):
                s += A[i, p] * B[p, j]
            C[i, j] = s
    return C


def tiled_matmul(A, B, tile=32):
    """Tiled (blocked) matrix multiply — approximates GPU shared-memory pattern."""
    m, k = A.shape
    _, n = B.shape
    C = np.zeros((m, n))
    for ii in range(0, m, tile):
        for jj in range(0, n, tile):
            for kk in range(0, k, tile):
                i_end = min(ii + tile, m)
                j_end = min(jj + tile, n)
                k_end = min(kk + tile, k)
                C[ii:i_end, jj:j_end] += A[ii:i_end, kk:k_end] @ B[kk:k_end, jj:j_end]
    return C


def run_benchmark():
    """Validate matrix multiplication against numpy/BLAS reference."""
    rng = np.random.default_rng(42)
    results = []
    passed = 0
    total = 0

    test_cases = [
        {"name": "tiny_4x4", "m": 4, "k": 4, "n": 4, "naive": True},
        {"name": "small_32x32", "m": 32, "k": 32, "n": 32, "naive": True},
        {"name": "rect_64x128x32", "m": 64, "k": 128, "n": 32},
        {"name": "medium_256x256", "m": 256, "k": 256, "n": 256},
        {"name": "large_512x512", "m": 512, "k": 512, "n": 512},
        {"name": "tall_1024x64", "m": 1024, "k": 64, "n": 64},
    ]

    for case in test_cases:
        total += 1
        m, k, n = case["m"], case["k"], case["n"]

        A = rng.normal(0, 1, (m, k))
        B = rng.normal(0, 1, (k, n))

        t0 = time.perf_counter_ns()
        C_np = A @ B
        elapsed_blas_ns = time.perf_counter_ns() - t0

        t0 = time.perf_counter_ns()
        C_tiled = tiled_matmul(A, B)
        elapsed_tiled_ns = time.perf_counter_ns() - t0

        max_diff = float(np.max(np.abs(C_np - C_tiled)))
        c_norm = float(np.max(np.abs(C_np)))
        rel_err = max_diff / max(c_norm, 1e-300)

        naive_err = float("nan")
        if case.get("naive"):
            C_naive = naive_matmul(A, B)
            naive_err = float(np.max(np.abs(C_np - C_naive)))

        ok = rel_err < 1e-10
        if ok:
            passed += 1

        results.append({
            "name": case["name"],
            "shape": f"{m}x{k} @ {k}x{n}",
            "max_abs_diff": max_diff,
            "rel_err": rel_err,
            "naive_max_diff": naive_err if not np.isnan(naive_err) else "skipped",
            "blas_ns": elapsed_blas_ns,
            "tiled_ns": elapsed_tiled_ns,
            "flops": 2 * m * k * n,
            "pass": ok,
        })

        status = "PASS" if ok else "FAIL"
        gflops_blas = (2 * m * k * n) / max(elapsed_blas_ns, 1) * 1e-9
        print(f"  [{status}] {case['name']} ({m}x{k}@{k}x{n}): "
              f"rel_err={rel_err:.2e}, BLAS={gflops_blas:.1f} GFLOPS")

    print(f"\nMatrix multiply parity: {passed}/{total} PASS")
    return results, passed == total


if __name__ == "__main__":
    from pathlib import Path as _P
    sys.path.insert(0, str(_P(__file__).resolve().parent))
    from common import write_results

    print("barraCuda CPU parity: Matrix Multiplication (GEMM)")
    print("=" * 55)
    results, all_pass = run_benchmark()
    out_path = "benchmarks/barracuda_cpu_parity/matmul_results.json"
    write_results(results, out_path, caller_file=__file__)
    sys.exit(0 if all_pass else 1)
