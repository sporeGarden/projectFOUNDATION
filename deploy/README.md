# deploy/

Operational scripts for launching foundation validation runs.

## Quick Start

```bash
# 1. Deploy NUCLEUS composition on your gate (via projectNUCLEUS)
cd ../../projectNUCLEUS/deploy
bash deploy.sh --composition nest --gate irongate

# 2. Fetch public data sources (NCBI, UniProt, KEGG)
cd ../../projectFOUNDATION/deploy
bash fetch_sources.sh --thread all

# 3. Run full validation with provenance
bash foundation_validate.sh --thread wcm
```

## Scripts

### `fetch_sources.sh`

Retrieves datasets from public repositories and computes BLAKE3 hashes.
Data is stored in `.data/` (git-ignored). Each fetched file becomes a
provenance chain anchor.

```bash
bash fetch_sources.sh --thread wcm              # Thread 1 only
bash fetch_sources.sh --thread enviro            # Thread 4 only
bash fetch_sources.sh --thread all               # All threads
bash fetch_sources.sh --thread all --register    # + register with NestGate
```

**Supported threads**: `wcm` (1), `plasma` (2), `immuno` (3), `enviro` (4),
`ltee` (5), `ml` (5-ML), `ag` (6), `anderson` (7), `health` (8), `gaming` (9),
`provenance` (10), `all`.

The fetcher is **manifest-driven** — it reads `data/sources/*.toml` and
dispatches by `database` field + `accessions`. Thread resolution uses
`lineage/THREAD_INDEX.toml` via `lib/thread_registry.sh`.

**NCBI API key**: Set `NCBI_API_KEY` for 10 requests/sec (vs 3/sec default).

### `foundation_validate.sh`

Full validation pipeline with provenance wrapping. Requires a running
NUCLEUS composition (deployed via projectNUCLEUS).

```bash
bash foundation_validate.sh --thread wcm         # Thread 1 validation
bash foundation_validate.sh --thread all          # All threads
bash foundation_validate.sh --skip-fetch          # Skip fetch, use cached data
```

**Phases**:
1. Health-check NUCLEUS primals (7 checked; 3 required — provenance trio)
2. Create provenance session (rhizoCrypt DAG + loamSpine spine)
3. Fetch data sources (delegates to `fetch_sources.sh`)
4. Register artifacts in NestGate with BLAKE3 anchors
5. Execute workloads through toadStool
6. Compare results against validation targets (`data/targets/*.toml`)
7. Commit provenance (Merkle root + loamSpine + sweetGrass braid)
8. Write `results.json`, `provenance.toml`, and `VALIDATION_REPORT.md` to `validation/run-<timestamp>/`

## Data Flow

```
Public Repos (NCBI, UniProt, KEGG, PDB)
  │
  │ fetch_sources.sh
  ▼
.data/                      (local cache, git-ignored)
  │
  │ foundation_validate.sh
  ▼
NestGate                    (BLAKE3 content-addressed storage)
  │
  ├── rhizoCrypt            (DAG session: every step recorded)
  ├── loamSpine             (permanent ledger: Merkle root committed)
  └── sweetGrass            (attribution braid: data → computation → result)
  │
  ▼
validation/run-<timestamp>/ (human-readable report + provenance artifacts)
```

## Sediment Layers

Each validation run is a sediment layer in foundation's geological record.
The Merkle root anchors the complete chain. When springs absorb these
patterns and validate additional targets, their results flow back as new
layers — strengthening the foundation over time.

```
Layer 0: Data anchors (NCBI genomes, UniProt proteomes, KEGG pathways)
Layer 1: Structural validation (gene counts, pathway counts, format checks)
Layer 2: Computational validation (spring experiments reproducing published results)
Layer 3: Cross-thread validation (shared parameters, provenance braids across threads)
Layer 4: Product validation (helixVision, blueFish, esotericWebb consuming validated data)
```

### `backfill_hashes.sh`

Adds BLAKE3 hashes to source manifest TOMLs for files already in `.data/`.
Used to incrementally populate the `blake3` field as data is fetched.

```bash
bash backfill_hashes.sh data/sources/thread01_wcm.toml
```

## Shared Libraries (`lib/`)

| File | Purpose |
|------|---------|
| `primal_ipc.sh` | Primal discovery (env → socket → config), RPC clients, blake3_hash |
| `json_rpc.sh` | Typed JSON-RPC response parsing (`rpc_has_result`, `rpc_has_error`, `rpc_error_message`) |
| `thread_registry.sh` | Runtime thread metadata from `THREAD_INDEX.toml` |
| `target_compare.sh` | Phase 6 target comparison logic |
| `report_writer.sh` | Phase 8 report generation, provenance TOML/JSON, spring folder distribution |

## Discovery Config

`discovery_defaults.toml` — UDS-first primal discovery with TCP bootstrap fallback.

**Transport resolution (per primal):**
1. Environment: `${PRIMAL}_SOCKET` (UDS) or `${PRIMAL}_PORT` (TCP)
2. XDG discovery socket: `capability.resolve` → `result.socket` / `result.port`
3. Config: `[sockets]` (UDS, all primals) then `[bootstrap_tcp]` (dev/desktop only)

VPS deployments (`--uds-only`) resolve at step 1 or 2 and never reach TCP bootstrap.
All `rpc_*` functions try UDS first, fall back to TCP/HTTP for desktop compatibility.

## Prerequisites

- NUCLEUS composition running (`deploy.sh --composition nest --gate irongate`)
- `b3sum` (BLAKE3 hasher)
- `curl`, `nc` (netcat), `python3` (for JSON/TOML parsing)
- `toadstool` binary in plasmidBin (or PATH)
