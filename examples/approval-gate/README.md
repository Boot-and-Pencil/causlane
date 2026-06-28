# approval-gate

Pre-alpha runnable example for a hard-effect approval gate. It is a local
protocol example, not a production runtime recipe.

Flow:

```text
ActionPlan -> GateApproved -> ApprovalRequirement -> ExecutionBarrier witness -> Bundle replay
```

Run it from the repository root:

```bash
python3 tools/examples-check
```
