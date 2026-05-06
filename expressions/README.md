# expressions/

Domain thread expression documents. Each expression zooms into one thread
from the unified lineage with detailed paper-by-paper analysis, NUCLEUS
composition blueprints, data target inventories, and petalTongue
visualization maps.

## What Is an Expression?

The unified lineage (`lineage/THE_UNIFIED_LINEAGE.md`) maps the complete
foundation as a single interconnected graph. An expression document is a
**subgraph view** — it focuses on one domain thread with enough detail
to drive actual NUCLEUS validation runs.

Expressions are not independent projects. They inherit context from the
unified lineage and reference cross-thread connections. A reader should
always read `THE_UNIFIED_LINEAGE.md` first, then drill into the expression
for the domain they care about.

## Active Expressions

| Expression | Thread | Status |
|-----------|--------|--------|
| `ABG_WHOLE_CELL_REBUILD.md` | Thread 1: Whole-Cell Modeling | Active |

## How to Author a New Expression

See `specs/EXPRESSION_AUTHORING_GUIDE.md` for the template and requirements.
The short version:

1. Identify the thread from `lineage/THREAD_INDEX.toml`
2. Collect the external paper lineage for the domain
3. Map each paper to NUCLEUS composition blueprints
4. Create stub data source and target TOMLs in `data/`
5. Define the petalTongue visualization vision
6. Note cross-thread connections explicitly
7. Submit as a new file in this directory

## Naming Convention

`<DOMAIN_SHORT>_<DESCRIPTOR>.md` — e.g. `ABG_WHOLE_CELL_REBUILD.md`,
`MURILLO_PLASMA_TRANSPORT.md`, `GONZALES_JAK_SERIES.md`.
