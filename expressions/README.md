# expressions/

Domain thread expression documents. Each expression zooms into one thread
from the unified lineage with detailed paper-by-paper analysis, NUCLEUS
composition blueprints, data target inventories, and petalTongue
visualization maps.

**Last reviewed**: Wave 72. Expression content (scientific lineage, paper
coverage, validation targets) remains valid. Infrastructure capabilities
added since Wave 63 (L4 routing, virtual relay, topology affinity) affect
NUCLEUS composition deployment but not foundation-layer data relationships.
Expressions reference primals by capability, not routing implementation.
Product mappings (helixVision, blueFish, esotericWebb) describe which
domain threads feed which product — unchanged by infrastructure evolution.

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
| `PLASMA_QCD_SOVEREIGN_GPU.md` | Thread 2: Plasma Physics / QCD | Active |
| `IMMUNO_DRUG_DISCOVERY.md` | Thread 3: Immunology / Drug Discovery | Active |
| `ENVIRONMENTAL_GENOMICS.md` | Thread 4: Environmental Genomics & Field Science | Active |
| `LTEE_EVOLUTIONARY_DYNAMICS.md` | Thread 5: Evolutionary Biology / LTEE | Active |
| `ML_SURROGATES.md` | Thread 5: LTEE ML Surrogates | Active |
| `MEASUREMENT_SCIENCE.md` | Thread 6: Agricultural Science + Thread 7: Anderson Mathematics | Active |
| `SOVEREIGN_HEALTH.md` | Thread 8: Human Health / Clinical | Active |
| `GAMING_CREATIVE_SCIENCE.md` | Thread 9: Gaming / Creative | Active |
| `PROVENANCE_ECONOMICS.md` | Thread 10: Provenance / Economics | Active |
| `LTEE_EVOLUTION.md` | Thread 5: LTEE (initial seed) | Superseded by `LTEE_EVOLUTIONARY_DYNAMICS.md` |

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
