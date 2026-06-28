# consequence-parallelism

Pre-alpha runnable example for safe frontier selection over independent
consequences. It is a local protocol example, not a production runtime recipe.

Flow:

```text
GraphIndex -> select_frontier -> conflict-free antichain -> bundle replay
```

Run it from the repository root:

```bash
python3 tools/examples-check
```
