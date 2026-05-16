# Thread 1 (Whole-Cell Modeling) Validation Summary

**Run ID:** `wcm-20260509`  
**Date:** 2026-05-09 (America/Chicago session)  
**Repository:** `ecoPrimals/gardens/projectFOUNDATION`  
**Expression:** `expressions/ABG_WHOLE_CELL_REBUILD.md`  
**Machine-readable sources:** `data/sources/thread01_wcm.toml`  
**Numerical targets (not executed in this run):** `data/targets/thread01_wcm_targets.toml`

This was a **diagnostic end-to-end attempt** against `deploy/foundation_validate.sh --thread wcm` plus independent verification of public data anchors and workload TOMLs.

---

## 1. What Was Validated

| Area | Method |
|------|--------|
| NUCLEUS primal liveness | `foundation_validate.sh` Phase 1 (BearDog, Songbird, ToadStool, NestGate, rhizoCrypt, loamSpine, sweetGrass) |
| Public data fetch | `deploy/fetch_sources.sh --thread wcm` → `.data/` |
| Content addressing | BLAKE3 of all files under `.data/` → `DATA_BLAKE3_MANIFEST.tsv` |
| Genome / assembly / bioproject artifacts | Fetch log + manual hash listing |
| UniProt proteome sizes | Logic from `wcm-proteome-validation.toml`, run from repo root |
| KEGG pathway list files | Logic from `wcm-kegg-pathway-check.toml`, run from repo root |
| Workloads via ToadStool | `toadstool execute` on each `workloads/thread01_wcm/*.toml` |
| Full provenance pipeline | Intended: `foundation_validate.sh` Phases 2–7 (rhizoCrypt + loamSpine + sweetGrass + NestGate registration) |

---

## 2. Results Overview

### 2.1 Passed

- **Phase 1 health checks (7/7):** All configured primals responded to the script’s probes on default ports (9100, 9200, 9400, 9500, 9700, 9850); **rhizoCrypt** was reported **[OK]** via process liveness (`pgrep`), not via a successful DAG RPC (see gaps).
- **`fetch_sources.sh --thread wcm`:** Completed with **0 fetch failures** on this host. Most objects were **already cached** from prior runs; one UniProt object was re-downloaded but is **structurally invalid** (see failures).
- **NCBI nucleotide GenBank:** `NC_000908.2`, `CP016816.2`, `NC_000913.3` present with non-trivial sizes and stable BLAKE3 anchors (see manifest).
- **NCBI assembly report JSON:** `GCA_000027325.1`, `GCA_900015295.1` present.
- **NCBI BioProject:** `PRJNA357500.xml` present.
- **KEGG pathway lists (`mge`, `eco`, `mmc`):** All three files present; line counts meet workload minima (**58**, **137**, **145** pathways respectively). Full output: `wcm-kegg-pathway-check.manual.stdout`.
- **UniProt proteomes (2/3):**
  - **UP000000807** (*M. genitalium*): **483** sequences — within scripted band 400–600.
  - **UP000000625** (*E. coli* K-12): **4403** sequences — within 4000–5000.
- **Manual replay of workload shell from repository root:** Genome fetch + hash script completed; proteome + KEGG checks ran with correct paths (see `*.manual.stdout`). This establishes that **the bash logic is sound when `cwd` and paths are correct**.

### 2.2 Failed or Degraded

- **`foundation_validate.sh`:** Stopped at **Phase 2 — Create Provenance Session**. No DAG session ID returned; see `foundation_validate_attempt.log`.
- **ToadStool execution of packaged workloads:** All three `execute` runs **mis-executed relative to the TOML intent:**
  - Native runtime **ignored** `working_dir` (`foundation` not in trusted directories) and used a **temp cwd**, so `${FOUNDATION_ROOT:-.}/.data` did not resolve to the repo’s `.data`, and `deploy/fetch_sources.sh` was not found.
  - Captured logs: `wcm-*.stdout` (machine output includes ToadStool JSON logs + stderr).
- **UniProt UP000018174 (*M. mycoides* / syn3A-related anchor in manifest):** File on disk is **20 bytes**, `gzip` reports **“compressed data, truncated”**, and sequence count is **0**. The REST stream URL returns **HTTP 200** with effectively **empty payload** for `compressed=true` (UniProt metadata still describes the proteome, but `proteomeType` is **Excluded** in the JSON — **the automated FASTA stream is unusable for validation as written**).
- **`thread01_wcm_targets.toml`:** No automated pass/fail comparison was run (the validation script does not ingest this file yet).

### 2.3 Provenance Artifacts From This Run

| Artifact | Present? |
|----------|----------|
| `DATA_BLAKE3_MANIFEST.tsv` | Yes — full BLAKE3 list for `.data/` |
| `VALIDATION_REPORT.md` (from `foundation_validate.sh`) | **No** — script exited before report phase |
| `braid.json` (sweetGrass) | **No** |
| rhizoCrypt Merkle / loamSpine spine IDs | **No** (session creation failed) |

---

## 3. Step Counts (rollup)

| Category | Count | Notes |
|----------|------:|-------|
| Primal health sub-checks **passed** | **7** | Ports + rhizoCrypt PID path |
| `fetch_sources` thread-wcm **completed without curl failures** | **1** run | 12 logical sources; **1** delivers unusable UniProt gzip |
| Workload logic checks **passed** (manual, repo root) | **2** of 3 | Proteome: Mgen + E.coli; KEGG: 3/3 |
| **foundation_validate** phases **completed** | **1** of ~8 | Only Phase 1 |
| **ToadStool workload runs matching author intent** | **0** of 3 | Working-dir isolation |

---

## 4. Gaps and Feedback (projectNUCLEUS / Upstream)

### 4.1 `deploy/foundation_validate.sh` ↔ **rhizoCrypt**

1. **Phase 2 RPC:** `dag.session.create` returned an empty response in this environment. Separate probing showed **no usable newline-JSON response on TCP `127.0.0.1:9601`** with plain `nc` while the process listens — likely **wire-format / authentication (BTSP)** mismatch between the script’s `printf … | nc` transport and the server’s expectations (rhizoCrypt implements newline JSON-RPC for some transports; **TCPPlain may differ**).
2. **Parameter schema:** The script passes `"params":{"name":"…"}`. Current rhizoCrypt tests and handlers in-tree use **`session_type`** (e.g. `"General"`) for `dag.session.create`, not `name`. Even with transport fixed, params may need alignment.

### 4.2 **ToadStool** (plasmidBin / NUCLEUS deploy)

- Workloads declare `working_dir` = foundation repo and relative `deploy/...` paths. With **`Standard` isolation**, ToadStool logged **`working_dir not in trusted_directories`** and executed under **`temp_dir`**, breaking all three WCM workloads.

**Feedback:** Either add foundation path to trusted directories for validation compositions, or change workloads to absolute paths / embed data-dir via environment injected by the runner.

### 4.3 **`deploy/fetch_sources.sh`**

- **No post-download validation:** A **20-byte “.fasta.gz”** was treated as success (`curl -sf`). Add `gzip -t` or minimum size check before counting a fetch as OK.
- **UniProt `UP000018174`:** The manifest still points at this accession for *M. mycoides*; **UniProt currently exposes an excluded proteome with an empty FASTA stream** for the REST query used. Pick a **non-excluded** reference proteome + update `fetch_sources.sh` + workload bounds, or fetch from **genome-derived protein FASTA** (e.g. linked assembly) instead.

### 4.4 **Paper / manifest consistency (informational)**

- In `ABG_WHOLE_CELL_REBUILD.md`, the NCBI anchor table lists **CP016816.2** under “Papers D, E, F, G”; **Paper D** in the same doc is *E. coli* (**NC_000913.3**). Consider correcting the table row for **CP016816.2** to **E–G** only to avoid provenance confusion.

---

## 5. Dependencies on NUCLEUS Primals

| Primal | Role in scripted pipeline | Observed state |
|--------|---------------------------|----------------|
| BearDog | Tower coordination | Responded (`health.liveness`) |
| Songbird | HTTP health | **OK** |
| ToadStool | `execute` workloads | Binary present; **workload path/cwd issue** |
| NestGate | Storage registration | Responded |
| rhizoCrypt | DAG session + events | Process alive; **DAG create not verified over TCP** |
| loamSpine | Spine entries | HTTP JSON-RPC responded |
| sweetGrass | Braid creation | Reached in script only after Phase 2 — **not invoked** in this run |

---

## 6. Files in This Directory

| File | Description |
|------|-------------|
| `VALIDATION_SUMMARY.md` | This document |
| `DATA_BLAKE3_MANIFEST.tsv` | BLAKE3 for every file under `foundation/.data/` at validation time |
| `foundation_validate_attempt.log` |Stdout/stderr from `foundation_validate.sh --thread wcm --skip-fetch` |
| `wcm-*.stdout` | Raw **ToadStool** execution logs (failed workload intent) |
| `wcm-*.manual.stdout` | Outputs from **manual** replay of workload bash from repo root |

---

## 7. Conclusion

**Public reference anchors for WCM (NCBI genomes, KEGG lists, two of three UniProt proteomes) are in good shape** and are captured with **BLAKE3** in `DATA_BLAKE3_MANIFEST.tsv`. The **end-to-end foundation validation pipeline did not reach data registration, workload execution under ToadStool-as-orchestrator, or sweetGrass closure**, due to **rhizoCrypt session RPC** and **ToadStool working-directory trust** issues, plus **one upstream UniProt stream** that does not currently yield a valid FASTA.

These items are suitable **handback inputs** to projectNUCLEUS for the next integration pass.
