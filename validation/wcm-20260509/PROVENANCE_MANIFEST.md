# Provenance manifest — `wcm-20260509`

This run **did not** complete the NUCLEUS provenance chain (`rhizoCrypt` DAG session → `loamSpine` spine entries → `sweetGrass` braid). See `VALIDATION_SUMMARY.md` §2.3.

## Generated anchors (local)

| Artifact | Path (this run) |
|----------|-----------------|
| BLAKE3 manifest of `.data/` | `DATA_BLAKE3_MANIFEST.tsv` |
| Fetch + hash log (manual) | `wcm-genome-fetch-hash.manual.stdout` |

## Not generated

- `braid.json` (sweetGrass)
- `VALIDATION_REPORT.md` (from `foundation_validate.sh`)
- NestGate registration records tied to a completed validation session

When `foundation_validate.sh` reaches Phase 7, copy or symlink the canonical `VALIDATION_REPORT.md` and `braid.json` from the script’s `--results-dir` into a dated folder such as this one.
