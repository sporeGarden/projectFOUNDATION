# graphs/

Foundation-specific deploy graphs for validation runs.

## Relationship to projectNUCLEUS

projectFOUNDATION does not own the canonical primal deploy graphs. Those live
in projectNUCLEUS (curated from primalSpring). These graphs are
**validation compositions** — they specify which NUCLEUS atomics are
required to validate a particular domain thread.

For primal graphs, see:
- `projectNUCLEUS/graphs/` — curated deploy graphs for gate deployment
- `primalSpring/graphs/` — canonical source of truth for graph schemas

## Available Graphs

| Graph | Purpose | Base Atomic |
|-------|---------|-------------|
| `foundation_validation.toml` | General foundation validation (provenance-heavy) | Nest Atomic + Node Atomic |

## Usage

```bash
# Deploy via projectNUCLEUS infrastructure
cd ../../projectNUCLEUS/deploy
bash deploy.sh --composition nest --gate irongate

# Execute foundation validation through toadStool
toadstool execute graphs/foundation_validation.toml
```

As thread expressions mature, thread-specific validation graphs may be
added (e.g. `thread01_wcm_validation.toml` for whole-cell modeling runs
requiring full Node Atomic with GPU dispatch).

## Alignment Notes (Wave 46, primalSpring v0.9.27)

The local graph is a **validation superset** of the primalSpring canonical
`compositions/foundation_validation.toml`. Known intentional deltas:

| Aspect | Local | primalSpring canonical |
|--------|-------|----------------------|
| `skunkbat` node | Present (defense layer for validation security audit) | Absent |
| `by_capability` strings | `crypto`, `storage`, `spine`, `braid`, `orchestration`, `discovery`, `defense`, `compute`, `math`, `dag`, `visualization`, `ai` | `security`, `content`, `ledger`, `attribution` |
| `bonding_policy` | Not specified (defaults) | Covalent/metallic/weak tiers |
| `fallback = "skip"` on optionals | Present on coralreef, petaltongue, squirrel | Present on coralreef, petaltongue, squirrel |
| Per-node `security_model` | Graph-level only (`btsp_enforced`) | Per-node |

The `by_capability` drift is cosmetic — toadStool resolves by songbird
registry, not string matching. The graph will converge with Rust elevation
(Phase B) when `CompositionContext` replaces the bash discovery layer.

**Signal graphs** (`nest.store`, `nest.commit`) are not referenced locally.
The bash pipeline uses 4-call RPC sequences as documented in the elevation
review. Signal adoption targets Phase C (Rust UniBin).
