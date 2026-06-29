# Migration And Shadow Adoption

M12.3 documents how a host project can adopt Causlane incrementally without a
rewrite. The path starts with replayable examples and diagnostic observation,
then moves toward guarded execution only after the host has evidence that its
expectations match actual runtime behavior.

This guide is intentionally bounded. It does not add a new protocol state, does
not change replay schemas, and does not make shadow comparison an enforcement
authority.

## Replay-first baseline

Start outside production traffic:

1. Model one narrow host workflow as an action graph or runnable example.
2. Keep the host's existing execution path as the authority.
3. Use Causlane replay, examples, or bounded runtime tests to describe the
   expected action lifecycle.
4. Record limitations next to the example instead of turning missing production
   behavior into an implied guarantee.

The first useful adoption outcome is a deterministic baseline: maintainers can
describe what should happen before any adapter is allowed to affect production
work.

## Shadow diagnostics

After a baseline exists, host integrations can observe runtime behavior without
enforcing it. The `tokio-runtime` shadow API exposes:

```text
InProcessRuntimeEvent
ShadowExpectation
ShadowExpectationKind
ShadowComparison
compare_shadow_events
```

The host provides `ShadowExpectation` values and compares them with already
emitted `InProcessRuntimeEvent` values. The result is a `ShadowComparison` with
matched, mismatched and unexpected observations.

Shadow comparison is useful for rollout questions such as:

- did this task get accepted, rejected, blocked, executed or failed as expected;
- did the observed partition match the expected partition;
- did a rejected or failed task carry the expected `HostDispatchError`;
- did the runtime emit extra events that the host did not expect.

## Diagnostic-only boundary

Shadow comparison is not a scheduler or policy decision point. A
`ShadowComparison` must not:

- admit new work;
- block, cancel, retry or reorder work;
- create observed truth;
- override authz, capability, lease or witness checks;
- publish crates, create tags or mutate release state.

Mismatches are data for operators, tests and rollout review. They do not feed
back into runtime execution.

## Incremental rollout

Use the following progression for a host integration:

1. **Document the current behavior.** Pick one bounded workflow and write the
   expected task outcomes.
2. **Run in observation mode.** Subscribe to runtime events and compare them
   with expectations, while the existing host path remains authoritative.
3. **Classify mismatches.** Treat missing, mismatched and unexpected events as
   rollout evidence. Fix expectations, integration wiring or host behavior
   before adding enforcement.
4. **Guard a small path.** Move only one low-risk path to guarded execution
   after diagnostics consistently match.
5. **Expand by surface.** Repeat the same baseline, observation and guarded
   execution cycle per workflow or adapter surface.

This keeps Causlane as a small semantic dispatch layer. It avoids replacing a
host system's queue, workflow engine, scheduler, release process or incident
runbook.

## Negative controls

Existing runtime tests cover the important failure mode for this milestone:
shadow mismatch remains diagnostic-only. Negative controls include missing
expected events, unexpected actual events, produced-ref mismatch, keyed versus
unkeyed rejection mismatch, and an in-process runtime test where execution still
completes even when the shadow expectation is wrong.

Those controls are evidence for the adoption guide, not a production rollout
certificate.

## Deferred production claims

The following are intentionally out of scope for M12.3:

- durable migration workers;
- automatic rollback from shadow mismatch;
- semver compatibility policy for future stable releases;
- provider-specific CI or deployment integration;
- production enforcement driven by shadow diagnostics.

These remain later adapter, compatibility or operational-readiness work.
