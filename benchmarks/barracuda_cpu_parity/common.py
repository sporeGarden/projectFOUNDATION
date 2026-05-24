#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Shared utilities for barraCuda CPU parity benchmarks."""

import datetime
import json
import platform
import subprocess
import sys
from pathlib import Path

import numpy as np


def provenance_header(*, caller_file: str | None = None,
                      extra_versions: dict | None = None) -> dict:
    """Build a provenance dict for benchmark result JSON files.

    Uses repo-relative paths so committed results are portable.
    """
    commit = "unknown"
    try:
        commit = subprocess.check_output(
            ["git", "rev-parse", "--short", "HEAD"],
            stderr=subprocess.DEVNULL,
        ).decode().strip()
    except Exception:
        pass

    src = caller_file or __file__
    try:
        repo_root = Path(subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            stderr=subprocess.DEVNULL,
        ).decode().strip())
        command = f"python3 {Path(src).resolve().relative_to(repo_root).as_posix()}"
    except Exception:
        command = f"python3 {Path(src).name}"

    versions = {
        "python": platform.python_version(),
        "numpy": np.__version__,
    }
    if extra_versions:
        versions.update(extra_versions)

    return {
        "provenance": {
            "generated": datetime.datetime.now(datetime.timezone.utc).isoformat(),
            **versions,
            "platform": platform.platform(),
            "git_commit": commit,
            "command": command,
        }
    }


def _numpy_json_default(x):
    """Handle numpy scalars that json.dump can't serialize natively."""
    if isinstance(x, np.bool_):
        return bool(x)
    if isinstance(x, (np.integer,)):
        return int(x)
    if isinstance(x, (np.floating,)):
        return float(x)
    raise TypeError(f"Object of type {type(x)} is not JSON serializable")


def write_results(results: list, out_path: str, *,
                  caller_file: str | None = None,
                  provenance_kw: dict | None = None) -> None:
    """Write benchmark results + provenance to JSON."""
    kw = dict(provenance_kw or {})
    if caller_file:
        kw["caller_file"] = caller_file
    output = {"results": results}
    output.update(provenance_header(**kw))

    try:
        with open(out_path, "w") as f:
            json.dump(output, f, indent=2, default=_numpy_json_default)
        print(f"\nResults written to {out_path}")
    except OSError:
        pass
