# simple-local

Pre-alpha runnable example for the smallest local flow. It is a local protocol
example, not a production runtime recipe.

Flow:

```text
ActionCall -> ActionPlan -> DispatchLogged -> Barrier -> NoopExecutor -> ObservedTruth -> Projection -> Replay
```

Run it from the repository root:

```bash
python3 tools/examples-check
```
