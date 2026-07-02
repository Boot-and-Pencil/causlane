# Scenario: release candidate promotion

## Purpose

This is the first reference scenario. It should be represented in docs, replay fixtures, Alloy, P and Rust tests.

## Backing artifacts

This scenario is now executable end-to-end (Alloy/P projections still pending —
see [`../11-contract-hardening-plan.md`](../11-contract-hardening-plan.md)):

- registry: [`contracts/examples/release_promote.registry.yaml`](../../contracts/examples/release_promote.registry.yaml) → compiled by `causlane-contracts`;
- plan hash: `causlane_contracts::examples::release_promote_plan_material` (the single source of truth for the trace's `plan_hash`);
- trace: [`contracts/examples/release_promote.trace.json`](../../contracts/examples/release_promote.trace.json) → verified by `causlane-replay::ReplayTrace::verify` (I-001/I-002/I-003/I-006/I-008);
- scenario catalog: [`contracts/scenarios/release_promote_success.scenario.yaml`](../../contracts/scenarios/release_promote_success.scenario.yaml) plus invalid fixtures → emitted by `causlane scenario emit-trace`;
- scenario schema: [`contracts/schema/scenario.schema.json`](../../contracts/schema/scenario.schema.json);
- CLI: `causlane bundle validate …` / `causlane scenario emit-trace …` / `causlane replay verify …` / `causlane formal generate alloy --scenario …`;
- formal smoke: `just formal-smoke` regenerates the success and selected negative Alloy facts and runs them against `verification/formal-full/alloy/core/causlane_core.als`.

## ActionCall

```yaml
predicate: release.promote_candidate
subject:
  release_candidate_id: rc_123
  target_environment: staging
circumstance:
  requested_by: user_42
  source_surface: ui
  readiness_policy: release_readiness_v1
  required_evidence:
    - readiness_ok
  rollback_plan_ref: rb_987
  idempotency_key: req_abc
```

## Consequence profile

```text
RuntimeExecution
```

## Planned impacts

```yaml
planned_impacts:
  - kind: environment_mutation
    hardness: hard
    scope: environment:staging
  - kind: release_candidate_status_update
    hardness: hard
    scope: release_candidate:rc_123
```

## Required witnesses

```yaml
requires:
  - id: readiness_before_promotion
    target_stage: execution_barrier_logged
    event:
      kind: observed_truth.committed
      predicate: readiness.check
      fact: readiness_ok
      scope: release_candidate:rc_123
```

## Required constraints

```yaml
claims:
  - resource: environment_write
    scope: environment:staging
    mode: exclusive
  - resource: release_candidate_write
    scope: release_candidate:rc_123
    mode: exclusive
```

## Expected lifecycle

```text
admitted
planned
dispatch_logged
execution_barrier_logged
executing
observed
projected
closed
```

## Must fail if

- readiness witness is missing;
- approval gate is stale or bound to different plan hash;
- environment write lease is unavailable;
- execution starts before barrier;
- projection emits status without observed-truth anchor.
