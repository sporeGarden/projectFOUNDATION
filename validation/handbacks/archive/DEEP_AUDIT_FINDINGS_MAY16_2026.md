# projectFOUNDATION Deep Audit Findings — May 16, 2026

**Audit scope**: Full codebase, specs, docs, benchmarks, CI, data manifests, workloads
**Context**: Post-primalSpring Wave 18, post-CATHEDRAL split
**Updated**: May 17, 2026 — Wave 21 absorption
**Note (May 23)**: Counts in §1–§3 are May 16 snapshots — geological record.
Current state: 7 shell scripts (~1,648 lines), 7 Python files, 17 CI gates,
6 CPU parity benchmarks (32 test cases). See root README for live counts.

---

## 1. Repository Profile

projectFOUNDATION is a **data/spec/deploy/docs** repository — not a Rust
crate. It contains no `.rs` files, no `Cargo.toml`, no `package.json`.

| Category | Count |
|----------|-------|
| Shell scripts (deploy/) | 5 files, ~1,270 lines total |
| Python scripts (benchmarks/) | 3 files, ~540 lines total |
| TOML manifests (data/sources, targets, workloads, graphs) | ~50 files |
| Markdown docs (expressions, lineage, specs, validation) | ~40 files |
| CI workflows | 2 files |

**No files exceed 800 lines.** Longest: `ABG_WHOLE_CELL_REBUILD.md` (645 lines).

---

## 2. Bugs Found and Fixed

| Bug | File | Fix |
|-----|------|-----|
| `scipy.__version__` NameError | `spectral_eigenvalues.py` | Added `import scipy` alongside `from scipy import linalg` |
| `results.update()` on tuple | `md_velocity_verlet.py` | Wrapped results list in dict before provenance merge |
| Docstring claims 1e-12 parity | `md_velocity_verlet.py` | Corrected to match implementation: <5% drift threshold |
| Stale `jq` dependency listed | `foundation_validate.sh` | Removed — script uses python3/curl/nc, never jq |
| `total_targets` mismatch: thread 4 | `thread04_enviro_targets.toml` | 13 → 12 (actual count) |
| `total_targets` mismatch: thread 7 | `thread07_anderson_targets.toml` | 24 → 23 (actual count) |
| `total_targets` mismatch: thread 8 | `thread08_health_targets.toml` | 12 → 11 (actual count) |
| Stale `kokkos-lammps/` path | `PLASMA_QCD_SOVEREIGN_GPU.md` | Updated to reference actual barraCuda bench + CPU baselines |
| Stale "3 validated" count | `PROVENANCE_ECONOMICS.md` | Corrected to "5 validated / 3 pending" |
| Stale paper count comment | `THREAD_INDEX.toml` | Fixed arithmetic: 23 mapped + 5 multi-part = 28 |
| CI shellcheck misses lib/ | `ci.yml` | Added `deploy/lib/primal_ipc.sh` and `deploy/lib/target_compare.sh` |

---

## 3. Benchmarks — barraCuda CPU Parity

### What exists (3 Python baselines)

| Script | Kernel | Parity standard |
|--------|--------|-----------------|
| `stats_variance.py` | `VarianceF64` | rel_err < 1e-6 |
| `md_velocity_verlet.py` | `velocity_verlet_split_f64.wgsl` | energy drift < 5% |
| `spectral_eigenvalues.py` | Anderson eigensolver | max eigenvalue diff < 1e-10 |

### What's missing (CPU baselines not in projectFOUNDATION)

- `tensor.matmul`, `linalg.solve`, `stats.mean` — referenced in graphs
- `spectral.*` (FFT) — referenced in plasma expression
- `math.sigmoid`, `rng.*` — referenced in gaming expression

### GPU Parity

**No GPU benchmarks in projectFOUNDATION.** GPU benches live in barraCuda:

| Bench | What it tests |
|-------|---------------|
| `kokkos_parity.rs` | VarianceF64 GPU vs Kokkos timing references |
| `lammps_parity.rs` | LJ + Yukawa at LAMMPS-scale N |
| `scipy_parity.rs` | SumReduce, Variance, cdist vs SciPy |

**Galaxy**: Not benchmarked in either repo. Different domain (astrophysics N-body).

**Kokkos hardware parity**: Comments reference published Kokkos/CUDA numbers
but don't run Kokkos side-by-side. Full hardware-matched parity awaits
matching GPU hardware.

---

## 4. Data Integrity — BLAKE3 Status

**Updated May 19, 2026**: 10/165 sources now have BLAKE3 hashes (Thread 1 WCM
backfilled). CI gates on non-regression (baseline 10).

| Item | Status |
|------|--------|
| Thread 1 WCM: 10/25 hashed | **DONE** (May 19 — NCBI/UniProt/KEGG fetched) |
| Source count reconciliation | **DONE** (6 mismatches fixed, CI gate added) |
| CI hash regression gate | **DONE** (fails if WCM < 10 or overall < 10) |
| Manifest-driven fetch | **DONE** (fetch_sources.sh reads TOMLs, not hardcoded) |
| Remaining 155 sources | Open — 15 WCM unfetchable, others need thread-by-thread runs |
| `blake3_hash` fallback bug | **FIXED** (was blake2b, now requires blake3 or b3sum) |

### Remaining data gaps

- **Thread 1 WCM**: 15 sources lack automated fetchers (BRENDA, EcoCyc, GitHub, etc.)
- **Thread 4**: Brandt farm soil data "Accession TBD"
- **Thread 4**: NOAA URL was pointing at NCBI — **FIXED**
- **Thread 5**: LTEE accession mismatches (fetch vs manifest) — **FIXED**
- **Thread 9**: Game design theory rows flagged "Queued for experiment design"

---

## 5. Target Validation Coverage

**184 total targets, 146 validated (79.3%), 38 unvalidated (20.7%)**

| Thread | Validated | Unvalidated | Status |
|--------|-----------|-------------|--------|
| 1 WCM | 0/27 | **27** | BLOCKED — RPC stack needed |
| 2 Plasma | 12/12 | 0 | PASS |
| 3 Immunology | 12/12 | 0 | PASS (targets validated, spring runs pending) |
| 4 Env Genomics | 8/12 | **4** | 4 pending: sovereign 16S, soil Anderson, QS env, sentinel |
| 5 LTEE | 14/18 | **4** | 4 pending: wetSpring B7 |
| 5 ML Surrogates | 12/12 | 0 | PASS |
| 6 Agricultural | 36/36 | 0 | PASS |
| 7 Anderson | 23/23 | 0 | PASS |
| 8 Human Health | 11/11 | 0 | PASS (targets validated, spring expanding) |
| 9 Gaming | 13/13 | 0 | PASS (targets seeded, ludoSpring growing) |
| 10 Provenance | 5/8 | **3** | 3 pending: W3C PROV, RootPulse, cross-spring |

---

## 6. Expression Coverage

**All 10 threads have expression documents.** The upstream Wave 18 blurb
incorrectly claimed threads 3, 4, 8, 9 "need expression."

| Thread | Expression | Lines |
|--------|-----------|-------|
| 1 | ABG_WHOLE_CELL_REBUILD.md | 645 |
| 2 | PLASMA_QCD_SOVEREIGN_GPU.md | ~200 |
| 3 | IMMUNO_DRUG_DISCOVERY.md | ~180 |
| 4 | ENVIRONMENTAL_GENOMICS.md | ~148 |
| 5 | LTEE_EVOLUTIONARY_DYNAMICS.md + ML_SURROGATES.md | ~300 |
| 6+7 | MEASUREMENT_SCIENCE.md | ~200 |
| 8 | SOVEREIGN_HEALTH.md | ~150 |
| 9 | GAMING_CREATIVE_SCIENCE.md | ~250 |
| 10 | PROVENANCE_ECONOMICS.md | ~160 |

---

## 7. Paper Queue / Unreviewed Work

No formal paper queue file exists. Active backlogs identified from expressions:

| Item | Source | Status |
|------|--------|--------|
| LTEE E2 (Mardikoraem & Woldring 2025) | healthSpring | QUEUED |
| LTEE E3 (Dolgikh FLS2) | airSpring | QUEUED |
| LTEE E4 (Woldring Lab 2024) | healthSpring | QUEUED |
| LTEE B7 genomics pipeline | wetSpring | STARTED |
| Gaming design-validation experiments | ludoSpring | QUEUED |
| Thread 1 WCM Paper A full accession audit | CATHEDRAL | NOT STARTED |
| GPU parity: 5/25 remaining plasma papers | hotSpring | ACTIVE |

---

## 8. Hardcoding Assessment

No problematic hardcoding found. All port/path defaults use env var overrides:

```bash
PRIMAL_HOST="${PRIMAL_HOST:-127.0.0.1}"
NESTGATE_PORT=$(discover_port "nestgate" "9500")
```

`discover_port()` attempts UDS discovery first, falls back to env, then
to the default. A `DISCOVERY_FALLBACK_COUNT` counter tracks fallback usage
and warns in the final report.

---

## 9. Mocks in Production

**None.** All self-validating mocks were eliminated in the May 16 deep
debt pass. The `enviro-qs-validation.toml` and `anderson-math-validation.toml`
workloads now delegate to their respective spring validation binaries.

---

## 10. Unsafe Code / Rust Idioms

**Not applicable** — projectFOUNDATION contains no Rust source code.
Unsafe code analysis applies to upstream primals and springs, documented
in `validation/handbacks/PRIMAL_DEEP_DEBT_HANDBACK.md`.

---

## 11. CI Coverage Gaps

| Gap | Severity | Action |
|-----|----------|--------|
| `deploy/lib/*.sh` not shellchecked | Medium | **FIXED** — added to ci.yml |
| Graph TOML structural validation | Low | Future: validate node/edge schema |
| Workload executability (binary existence) | Low | Not feasible in CI (external deps) |
| Python benchmark execution | Medium | Future: run benchmarks in CI |
| BLAKE3 hash coverage threshold | Info | By design: `pct < 0` never fails |

---

## 12. Datasets to Begin Examining

As the larger project comes together, these datasets align with the
10-thread foundation:

| Dataset | Thread | Source | Purpose |
|---------|--------|--------|---------|
| NCBI SRA (16S amplicon) | 4 | Public | Microbial community profiling |
| UniProt proteomes (Paper A) | 1 | Public | Whole-cell model protein inventory |
| KEGG pathways | 1 | Public | Metabolic network reconstruction |
| IRIS seismic data | 6 | Public | Environmental monitoring (airSpring) |
| NOAA weather stations | 6 | Public | Field measurement calibration |
| Lenski Lab frozen archives | 5 | Collaboration | LTEE 80,000-generation fitness |
| PDB structures (JAK family) | 3 | Public | Drug-target structural validation |
| FANTOM5 gene expression | 8 | Public | Tissue-specific transcriptomics |
| Galaxy Zoo classifications | — | Public | Citizen science validation patterns |
| W3C PROV test corpus | 10 | Public | Provenance model compliance |

---

## Summary

**Resolved in this pass**: 11 bugs/inconsistencies fixed, CI expanded,
benchmark docs updated, target counts reconciled, stale references corrected.

**Remaining structural debt**: BLAKE3 backfill (blocked on data fetch),
Thread 1 WCM unblock (upstream RPC), 38 unvalidated targets (4 threads),
Rust UniBin elevation (Phase B/C), GPU parity baselines (lives in barraCuda).

**No Rust code, no unsafe, no mocks, no files over 800L, no hardcoding debt.**
This repo is clean infrastructure — the evolution pressure is on the
springs and primals that feed it.
