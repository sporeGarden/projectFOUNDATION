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
