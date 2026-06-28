# Examples

The current repository keeps examples intentionally small for the pre-alpha
track. Runnable examples are checked by `python3 tools/examples-check`; planned
entries describe intended coverage and are not production integration claims.

Current examples:

```text
simple-local
  runnable: one action, in-memory audit, barrier/capability, observed truth,
  projection anchor and replay.

approval-gate
  runnable: hard effect waits for approval bound to action_id + plan_hash +
  impact_set_hash, with step-up, separation-of-duties and bundle replay.

consequence-parallelism
  planned:
  safe frontier selection for independent consequences.

why-not-parallel
  planned:
  explanation for conflict vs dependency.
```
