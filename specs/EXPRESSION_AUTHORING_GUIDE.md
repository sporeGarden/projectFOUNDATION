# Expression Authoring Guide

**Version**: 1.0.0
**Status**: Active
**License**: scyBorg triple

---

## Purpose

This guide explains how to author a new expression document for the
foundation. An expression zooms into one domain thread from the unified
lineage with enough detail to drive NUCLEUS validation runs.

---

## Prerequisites

Before authoring an expression:

1. Read `lineage/THE_UNIFIED_LINEAGE.md` to understand the full foundation
2. Identify your thread in `lineage/THREAD_INDEX.toml`
3. Collect the external paper lineage for your domain
4. Verify that the relevant springs have validation checks for the domain

---

## Required Sections

Every expression document must include:

### 1. Header

```markdown
# <Domain> <Descriptor>

**Thread N Expression — <one-line description>**

**Date**: <ISO date>
**Status**: Active — <current phase>
**License**: scyBorg triple — AGPL-3.0-or-later (code), ORC (system mechanics), CC-BY-SA 4.0 (this document)
**Thread**: N (<name>) in `lineage/THE_UNIFIED_LINEAGE.md`
**Cross-threads**: list of connected threads
```

### 2. Framing

Why this expression exists. What it validates. How it connects to the
gen3-to-gen4 evolution story.

### 3. The Paper Lineage

Per-paper analysis of the domain's published work. For each paper:
- Citation (authors, year, journal, DOI)
- Organism or system studied
- Key contribution
- Data sources used
- Key numerical results
- Parameter inheritance from prior papers

### 4. The Jelly Strings

Provenance gaps in the original work that NUCLEUS can structurally fill.
Table format:

| Gap | Papers | NUCLEUS Solution |
|-----|--------|-----------------|

### 5. Data Targets

Pointer to the machine-readable manifests:
- `data/sources/threadNN_<short>.toml`
- `data/targets/threadNN_<short>_targets.toml`

Create stub TOMLs if they don't exist yet.

### 6. NUCLEUS Composition Blueprints

Per-paper (or per-group-of-papers) composition maps:
- Which NUCLEUS atomics are required
- Which primals serve which roles
- What the deploy graph looks like

### 7. Spring Alignment

Table mapping springs to their contribution to this thread.

### 8. petalTongue Vision

How the thread's science should be visualized as live computation
surfaces. What DataBinding channels are needed. What the dashboard
looks like.

### 9. scyBorg Publication

How the expression's results will be published under the scyBorg
triple license. What the provenance chain looks like.

---

## Optional Sections

- **Cross-Thread Deep Dives**: Detailed analysis of how shared methods
  or parameters connect to other threads
- **Evolution Targets**: What the expression reveals about missing
  NUCLEUS capabilities (gaps to file back via wateringHole)

---

## File Location and Naming

Place the expression in `expressions/`:

```
expressions/<DOMAIN_SHORT>_<DESCRIPTOR>.md
```

Examples:
- `ABG_WHOLE_CELL_REBUILD.md` (Thread 1)
- `MURILLO_PLASMA_TRANSPORT.md` (Thread 2)
- `GONZALES_JAK_SERIES.md` (Thread 3)

Update `lineage/THREAD_INDEX.toml` to point to the new expression:

```toml
expression = "expressions/<FILENAME>.md"
```

---

## Data Manifest Creation

For each new expression, create or update:

1. `data/sources/threadNN_<short>.toml` — external data sources
2. `data/targets/threadNN_<short>_targets.toml` — expected results

Follow the schemas documented in `data/README.md`.

---

## Review Checklist

Before submitting a new expression:

- [ ] Header follows the template with thread number and cross-threads
- [ ] Paper lineage includes citations and key results for every paper
- [ ] Jelly strings table identifies at least 3 provenance gaps
- [ ] Data source TOML exists in `data/sources/`
- [ ] Validation target TOML exists in `data/targets/`
- [ ] NUCLEUS composition blueprints map papers to atomics
- [ ] Spring alignment table is complete
- [ ] petalTongue vision describes at least one visualization
- [ ] scyBorg publication section references the sovereign pipeline
- [ ] `THREAD_INDEX.toml` updated with expression path
- [ ] Cross-thread connections noted explicitly
