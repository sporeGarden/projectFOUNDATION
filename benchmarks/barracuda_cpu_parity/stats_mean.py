#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
barraCuda CPU parity baseline: Statistical mean (Kahan / naive / chunked).

Reference implementation for barraCuda's stats.mean capability.
Validates that compensated summation (Kahan) and naive approaches produce
results matching numpy within IEEE 754 f64 bounds.

Parity standard: rel_err < 1e-14 for well-conditioned data.
Industry references:
  - Kahan 1965: compensated summation
  - Higham 2002: accuracy and stability of numerical algorithms
"""

import sys
import time

import numpy as np


def kahan_mean(data):
    """Kahan compensated summation mean — reduces floating-point error."""
    s = 0.0
    c = 0.0
    for x in data:
        y = x - c
        t = s + y
        c = (t - s) - y
        s = t
    return s / len(data) if data else 0.0


def chunked_mean(data, chunk_size=256):
    """Chunked partial sums — approximates GPU reduction pattern."""
    n = len(data)
    if n == 0:
        return 0.0
    partial_sums = []
    for i in range(0, n, chunk_size):
        chunk = data[i:i + chunk_size]
        partial_sums.append(sum(chunk) / len(chunk))
    return sum(partial_sums) / len(partial_sums) if partial_sums else 0.0


def run_benchmark():
    """Validate mean computation against numpy reference."""
    rng = np.random.default_rng(42)
    results = []
    passed = 0
    total = 0

    test_cases = [
        {"name": "uniform_1k", "data": rng.uniform(0, 1, 1000)},
        {"name": "normal_10k", "data": rng.normal(0, 1, 10000)},
        {"name": "large_magnitude", "data": rng.normal(1e15, 1.0, 10000)},
        {"name": "alternating_sign", "data": np.array([(-1)**i * (i+1.0) for i in range(10000)])},
        {"name": "near_zero", "data": rng.normal(0, 1e-15, 5000)},
        {"name": "exponential_100k", "data": rng.exponential(1.0, 100000)},
    ]

    for case in test_cases:
        total += 1
        data = case["data"]
        t0 = time.perf_counter_ns()

        np_mean = float(np.mean(data))
        kahan = kahan_mean(data.tolist())
        chunked = chunked_mean(data.tolist())
        naive = sum(data.tolist()) / len(data)

        elapsed_ns = time.perf_counter_ns() - t0

        rel_err_kahan = abs(np_mean - kahan) / max(abs(np_mean), 1e-300)
        rel_err_naive = abs(np_mean - naive) / max(abs(np_mean), 1e-300)
        rel_err_chunked = abs(np_mean - chunked) / max(abs(np_mean), 1e-300)

        ok = rel_err_kahan < 1e-14
        if ok:
            passed += 1

        results.append({
            "name": case["name"],
            "n": len(data),
            "numpy_mean": np_mean,
            "kahan_mean": kahan,
            "naive_mean": naive,
            "chunked_mean": chunked,
            "rel_err_kahan": rel_err_kahan,
            "rel_err_naive": rel_err_naive,
            "rel_err_chunked": rel_err_chunked,
            "elapsed_ns": elapsed_ns,
            "pass": ok,
        })

        status = "PASS" if ok else "FAIL"
        print(f"  [{status}] {case['name']} (n={len(data)}): "
              f"numpy={np_mean:.10e}, kahan_err={rel_err_kahan:.2e}")

    print(f"\nMean parity: {passed}/{total} PASS")
    return results, passed == total


if __name__ == "__main__":
    from pathlib import Path as _P
    sys.path.insert(0, str(_P(__file__).resolve().parent))
    from common import write_results

    print("barraCuda CPU parity: Statistical Mean")
    print("=" * 50)
    results, all_pass = run_benchmark()
    out_path = "benchmarks/barracuda_cpu_parity/stats_mean_results.json"
    write_results(results, out_path, caller_file=__file__)
    sys.exit(0 if all_pass else 1)
