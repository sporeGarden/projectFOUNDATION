# workloads/

toadStool-executable workload definitions for foundation validation.

## Structure

```
workloads/
  thread01_wcm/          Whole-Cell Modeling validation workloads
  thread02_plasma/       Plasma Physics — thread-scoped (Sarkas MD)
  thread03_immuno/       Immunology (healthSpring PK + LTEE-B5)
  thread04_enviro/       Environmental Genomics (QS + lithoSpore Module 6)
  thread05_ltee/         LTEE (lithoSpore fitness + mutations + Anderson)
  thread06_ag/           Agricultural Science (airSpring suite)
  thread07_anderson/     Anderson Mathematics (22 targets + lithoSpore Module 7)
  thread08_health/       Health (healthSpring full validation)
  thread09_gaming/       Gaming / Creative (ludoSpring)
  thread10_provenance/   Provenance / Economics (primalSpring)
  groundspring/          groundSpring cross-cutting (29 validators, GPU bench)
  hotspring/             hotSpring cross-cutting (Chuna + Sarkas MD)
```

Thread-scoped directories (`thread02_plasma/`) contain workloads relevant to a
single domain thread. Cross-cutting directories (`hotspring/`, `groundspring/`)
contain spring-level validations that serve multiple threads.

## Workload Format

Each `.toml` follows the projectNUCLEUS workload template:

```toml
[metadata]
name = "workload-name"
description = "What this validates"
version = "0.1.0"
thread = "01"

[execution]
type = "native"
command = "/path/to/binary-or-script"
args = ["..."]
working_dir = "/path/to/working/directory"

[resources]
max_memory_bytes = 1073741824
max_cpu_percent = 80.0

[security]
isolation_level = "Standard"
trusted_directories = ["${SPRINGS_ROOT}", "${ECOPRIMALS_ROOT}"]
```

## Available Workloads

### Thread 1: Whole-Cell Modeling

| Workload | Validates |
|----------|-----------|
| `wcm-genome-fetch-hash` | NCBI genome downloads + BLAKE3 anchoring |
| `wcm-proteome-validation` | UniProt proteome sizes match published gene counts |
| `wcm-kegg-pathway-check` | KEGG metabolic pathway data for WCM organisms |

### Thread 6: Agricultural Science

| Workload | Validates |
|----------|-----------|
| `airspring-full-suite` | Complete airSpring validation suite (36/36 checks) |
| `airspring-et0-fao56` | FAO-56 reference evapotranspiration targets |
| `airspring-et0-methods` | ET0 method comparison targets |
| `airspring-soil-physics` | Soil physics and water retention targets |
| `airspring-water-balance` | Water balance and irrigation targets |
| `airspring-atlas-pipeline` | Atlas data pipeline validation |

### groundSpring (cross-cutting)

| Workload | Validates |
|----------|-----------|
| `gs-validate-all` | All 29 Rust validators (395/395 checks) |
| `gs-guidestone` | guideStone Level 3 (5 bare + 6 NUCLEUS IPC) |
| `gs-bench-gpu` | Three-mode GPU benchmark (110 delegations) |
| `gs-python-baselines` | All 29 Python baselines for provenance |

### Thread 4: Environmental Genomics

| Workload | Validates |
|----------|-----------|
| `enviro-qs-validation` | QS framework targets (7 checks) from wetSpring + airSpring |
| `litho-breseq-integration` | lithoSpore Module 6 → Thread 4 anchoring (8/8 PASS) |

### Thread 2: Plasma Physics (hotSpring)

| Workload | Validates |
|----------|-----------|
| `hs-sarkas-md` | Sarkas MD validation — thread-local (thread02_plasma/) |
| `hs-chuna-validation` | Chuna MD parity — cross-cutting (hotspring/) |
| `hs-sarkas-md-validation` | Sarkas MD validation — cross-cutting (hotspring/) |

### Thread 7: Anderson Mathematics

| Workload | Validates |
|----------|-----------|
| `anderson-math-validation` | 22 Anderson math targets across groundSpring + neuralSpring |
| `litho-anderson-integration` | lithoSpore Module 7 → Thread 7 anchoring (5/5 PASS) |

## Execution

Workloads are executed via `toadstool.validate` (preferred) or direct dispatch:

```bash
toadstool validate workloads/thread01_wcm/wcm-genome-fetch-hash.toml
```

Or through the full pipeline:

```bash
bash deploy/foundation_validate.sh --thread wcm
```

## Adding Workloads

As springs evolve to full composition, their validation binaries become
workloads here. The pattern:

1. Spring validates a published result (e.g., hotSpring reproduces Paper A cell cycle)
2. The validation binary is built and deployed to plasmidBin
3. A workload TOML is added here pointing to that binary
4. `foundation_validate.sh` picks it up and wraps it in provenance

Each new workload strengthens the sediment layer for its thread.
