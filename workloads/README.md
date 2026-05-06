# workloads/

toadStool-executable workload definitions for foundation validation.

## Structure

```
workloads/
  thread01_wcm/       Whole-Cell Modeling validation workloads
  thread04_enviro/    Environmental Genomics (future)
  thread03_immuno/    Immunology & Drug Discovery (future)
  thread08_health/    Human Health (future)
```

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
isolation_level = "None"
```

## Available Workloads

### Thread 1: Whole-Cell Modeling

| Workload | Validates |
|----------|-----------|
| `wcm-genome-fetch-hash` | NCBI genome downloads + BLAKE3 anchoring |
| `wcm-proteome-validation` | UniProt proteome sizes match published gene counts |
| `wcm-kegg-pathway-check` | KEGG metabolic pathway data for WCM organisms |

## Execution

Workloads are executed via toadStool dispatch:

```bash
toadstool execute workloads/thread01_wcm/wcm-genome-fetch-hash.toml
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
