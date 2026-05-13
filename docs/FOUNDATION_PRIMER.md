# Foundation Primer

**What is foundation, and how does it relate to everything else?**

---

## The Short Version

foundation is the validated scientific lineage that makes sporeGarden
products useful. It maps 28 companion papers, 8 springs with 13,100+
quantitative checks, and 16 faculty and community contacts into one
unified knowledge graph organized by 10 domain threads. Products like
helixVision and blueFish pull specific threads from this foundation to
serve their audiences.

---

## The Ecosystem Stack

```
                    Products (who benefits)
                 helixVision  blueFish  esotericWebb
                         ↑        ↑         ↑
                         └────┬───┘─────────┘
                              │
                    foundation (what to validate)
                  lineage / data / targets / expressions
                              │
                    projectNUCLEUS (how to deploy)
                  graphs / gates / deploy / workloads
                              │
                    primals (organisms)
                  beardog, toadstool, barracuda, ...
                              │
                    springs (validation)
                  hotSpring, wetSpring, healthSpring, ...
```

**foundation** sits between the primals/springs layer and the products
layer. It defines what science needs to be validated and where the data
lives. projectNUCLEUS handles deploying the primals that do the actual
computation.

---

## Why One Foundation?

The early instinct was to create separate foundations — one for whole-cell
modeling, one for plasma physics, one for drug discovery. But the science
is deeply interconnected:

- Anderson localization (math) threads through ecology, immunology,
  agriculture, and plasma physics
- The same GPU math (barraCuda) powers lattice QCD, protein structure
  prediction, and drug screening
- The same provenance trio validates game saves, biological samples,
  and medical records
- The same ODE solvers model both cellular metabolism and pharmacokinetics

Separating the foundation would sever these connections. One unified
lineage preserves the graph structure that makes the ecosystem powerful.

---

## The 10 Domain Threads

| # | Thread | One-Line Description |
|---|--------|---------------------|
| 1 | Whole-Cell Modeling | Rebuild 14 years of computational cell biology through NUCLEUS |
| 2 | Plasma Physics / Lattice QCD | Sovereign GPU physics on consumer hardware |
| 3 | Immunology / Drug Discovery | Anderson localization predicts drug efficacy |
| 4 | Environmental Genomics | Sequence-to-publication on sovereign hardware |
| 5 | Evolutionary Biology / LTEE | Structure prediction across 75K generations |
| 6 | Agricultural Science | Precision agriculture at the field edge |
| 7 | Anderson Mathematics | The mathematical backbone connecting all domains |
| 8 | Human Health / Clinical | Per-person PK/PD on patient-controlled hardware |
| 9 | Gaming / Creative | Rigorous game science with sovereign attribution |
| 10 | Provenance / Economics | Structural provenance replaces trust-me provenance |

---

## Deploying via projectNUCLEUS

foundation never deploys primals directly. The workflow:

1. **foundation** defines the data sources and expected results
   (`data/sources/`, `data/targets/`)
2. **projectNUCLEUS** deploys the primals needed for validation
   (`deploy/deploy.sh`)
3. **toadStool** dispatches the computation workload
4. **The provenance trio** records every step
5. **Results** land in `validation/` with full provenance

The deploy graph `graphs/foundation_validation.toml` specifies the
minimum composition needed: Tower (trust), Node (compute), Nest
(storage + provenance), with optional petalTongue (visualization)
and Squirrel (AI).

---

## For Product Teams

If you're building a product on the ecoPrimals ecosystem:

1. Identify which threads your product pulls from (see the table in
   `lineage/THE_UNIFIED_LINEAGE.md`, section 5)
2. Review the expression documents for those threads to understand
   the science validation status
3. Check `data/targets/` for validated results you can cite
4. Reference the foundation in your product documentation

The foundation gives your product scientific credibility. The validation
targets in this repo are the evidence that the primals compute correctly
for your domain.

---

## For Contributors

To add a new domain thread or extend an existing one:

1. Read `lineage/THE_UNIFIED_LINEAGE.md` for the full picture
2. Read `specs/EXPRESSION_AUTHORING_GUIDE.md` for the template
3. Create an expression document in `expressions/`
4. Create data manifests in `data/sources/` and `data/targets/`
5. Update `lineage/THREAD_INDEX.toml`

---

## Related Repos

| Repo | What It Does |
|------|-------------|
| **projectNUCLEUS** | Deploys primals — the infrastructure layer |
| **plasmidBin** | Stores primal binaries — the binary depot |
| **primalSpring** | Validates compositions — the structural test suite |
| **wateringHole** | Authoritative guidance — the standards source |
| **whitePaper** | Historical record — gen4/foundations/ is the personal genesis |
| **helixVision** | Product: genomics pipeline (Threads 1, 3, 4, 5) |
| **blueFish** | Product: sovereign ETL (Thread 4 + pipeline) |
| **esotericWebb** | Product: interactive fiction (Thread 9) |
