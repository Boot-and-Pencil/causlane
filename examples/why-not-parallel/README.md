# why-not-parallel

Pre-alpha runnable example for machine-readable parallelism blockers. It is a
local protocol example, not a production scheduler recipe.

Flow:

```text
GraphIndex -> select_frontier -> pair_conflict -> why_not_parallel_from_index
```

Run it from the repository root:

```bash
python3 tools/examples-check
```
