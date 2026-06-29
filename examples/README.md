# Examples

The current repository keeps examples intentionally small for the pre-alpha
track. Runnable examples are checked by `python3 tools/examples-check` and are
not production integration claims.

Current examples:

```text
simple-local
  runnable: one action, in-memory audit, barrier/capability, observed truth,
  projection anchor and replay.

approval-gate
  runnable: hard effect waits for approval bound to action_id + plan_hash +
  impact_set_hash, with step-up, separation-of-duties and bundle replay.

consequence-parallelism
  runnable: safe frontier selection for independent consequences, conflicting
  writes and lane capacity, with conflict-free parallel replay.

contracts-boundary-ergonomics
  runnable: public contracts boundary for registry compilation, bundle artifact
  verification, template resolution, plan hashing and impact hashing.

contracts-registry-bundle-workflow
  runnable: near-real multi-predicate registry compilation, bundle reload,
  plan-template cache identity, template resolution and fail-closed controls for
  the M12.5 API validation loop.

facade-kernel-ergonomics
  runnable: downstream facade-only admission, barrier policy and frontier
  selection through the `causlane` crate.

replay-diagnostics
  runnable: public replay explain diagnostics for accepted, invariant rejected
  and structural rejected release-promotion traces.

why-not-parallel
  runnable: machine-readable explanations for parallelizable ops, pending write
  conflicts, dependency blockers and active writer blockers.

reference-integration
  runnable: API submission, deterministic worker drain, runtime audit adapter
  append and guarded projection redaction for the M12.1 public alpha API story.

runtime-guarded-audit-projection
  runnable: authz-guarded execution, runtime audit append, trace projection,
  guarded projection redaction and negative controls for the M12.5 API
  validation loop.

runtime-operator-workflow
  runnable: multi-operation runtime host workflow with guarded execution,
  append-only audit trace projection, guarded dashboard projection redaction and
  negative controls for the M12.5 API validation loop.

release-orchestration
  runnable: CI gates, package-list review, publish dry-run planning and
  downstream smoke planning for the M12.2 release orchestration story.
```
