# Causlane cookbook

This cookbook collects executable recipes for the current Causlane control
surface. Each recipe points at existing fixtures and commands; the fixtures,
replay oracle and formal gates remain the authority.

## Common setup

Compile the reference bundle and emit the success trace into a scratch
directory:

```bash
mkdir -p /tmp/causlane-cookbook
causlane bundle compile \
  --registry contracts/examples/release_promote.registry.yaml \
  --out /tmp/causlane-cookbook/release_promote.bundle.json
causlane scenario emit-trace \
  --scenario contracts/scenarios/release_promote_success.scenario.yaml \
  --bundle /tmp/causlane-cookbook/release_promote.bundle.json \
  --out /tmp/causlane-cookbook/release_promote.trace.json
```

The same bundle and trace can be reused by the replay, contract-test and support
bundle recipes below.

## Add an action

1. Add or update a predicate in a registry such as
   [`contracts/examples/release_promote.registry.yaml`](../../contracts/examples/release_promote.registry.yaml).
2. Add at least one executable scenario under
   [`contracts/scenarios/`](../../contracts/scenarios/). Use
   [`release_promote_success.scenario.yaml`](../../contracts/scenarios/release_promote_success.scenario.yaml)
   as the reference shape.
3. Validate the registry and scenario:

```bash
causlane bundle validate contracts/examples/release_promote.registry.yaml
causlane scenario validate contracts/scenarios/release_promote_success.scenario.yaml
```

4. Emit a trace and verify it against the compiled bundle:

```bash
causlane scenario emit-trace \
  --scenario contracts/scenarios/release_promote_success.scenario.yaml \
  --bundle /tmp/causlane-cookbook/release_promote.bundle.json \
  --out /tmp/causlane-cookbook/release_promote.trace.json
causlane replay verify \
  --bundle /tmp/causlane-cookbook/release_promote.bundle.json \
  --trace /tmp/causlane-cookbook/release_promote.trace.json \
  --explain
```

5. If the action changes protocol-critical behavior, add the relevant replay
   negative control and formal impact record before claiming the behavior.

## Approval and witness failures

Witness/approval evidence must bind the exact action, plan and impact set. The
existing negative scenarios exercise common mistakes:

```bash
causlane scenario emit-trace \
  --scenario contracts/scenarios/missing_witness_invalid.scenario.yaml \
  --bundle /tmp/causlane-cookbook/release_promote.bundle.json \
  --out /tmp/causlane-cookbook/missing_witness.trace.json
causlane replay verify \
  --bundle /tmp/causlane-cookbook/release_promote.bundle.json \
  --trace /tmp/causlane-cookbook/missing_witness.trace.json \
  --explain --json
```

Use `approval_wrong_plan_invalid.scenario.yaml` and
`approval_wrong_impact_invalid.scenario.yaml` for binding-mismatch controls.

## Conflict and parallelism

Replay rejects conflicting exclusive leases with
[`conflicting_leases_invalid.scenario.yaml`](../../contracts/scenarios/conflicting_leases_invalid.scenario.yaml).
For scheduler-facing explanations, save this graph snapshot as
`/tmp/causlane-cookbook/conflict.graph.yaml`:

```yaml
produced_facts: []
active_ops: [environment_update:0]
lanes:
  - lane_id: default
    capacity: unbounded
ops:
  - action_id: environment_update
    op_index: 0
    lane: default
    requires: []
    writes: [environment:staging]
  - action_id: release_promote
    op_index: 0
    lane: default
    requires: []
    writes: [environment:staging, release_candidate:rc_123]
```

```bash
causlane why-not-parallel \
  --graph /tmp/causlane-cookbook/conflict.graph.yaml \
  --op environment_update:0 \
  --with release_promote:0 \
  --json
causlane graph export \
  --graph /tmp/causlane-cookbook/conflict.graph.yaml \
  --format mermaid \
  --op release_promote:0
```

## Drain and active leases

Drain fences are diagnostic and replay-checkable through existing fixtures:

```bash
causlane scenario emit-trace \
  --scenario contracts/scenarios/drain_with_active_lease_invalid.scenario.yaml \
  --bundle /tmp/causlane-cookbook/release_promote.bundle.json \
  --out /tmp/causlane-cookbook/drain.trace.json
causlane replay verify \
  --bundle /tmp/causlane-cookbook/release_promote.bundle.json \
  --trace /tmp/causlane-cookbook/drain.trace.json \
  --explain --json
```

Expected replay code: `DrainFenceWithActiveOverlap`.

## Replay and contract tests

Use replay explain when debugging one trace:

```bash
causlane replay verify \
  --bundle /tmp/causlane-cookbook/release_promote.bundle.json \
  --trace /tmp/causlane-cookbook/release_promote.trace.json \
  --explain --json
```

Use the contract-test manifest when a CI gate should assert a scenario set:

```bash
causlane contract test --manifest contracts/contract-tests.yaml --json
```

## Authz recipes

Required authz policies use the authz registry and scenario set:

```bash
causlane bundle compile \
  --registry contracts/examples/release_promote_authz.registry.yaml \
  --out /tmp/causlane-cookbook/release_promote_authz.bundle.json
causlane scenario emit-trace \
  --scenario contracts/scenarios/authz/authz_success.scenario.yaml \
  --bundle /tmp/causlane-cookbook/release_promote_authz.bundle.json \
  --out /tmp/causlane-cookbook/authz_success.trace.json
causlane replay verify \
  --bundle /tmp/causlane-cookbook/release_promote_authz.bundle.json \
  --trace /tmp/causlane-cookbook/authz_success.trace.json \
  --explain
```

Use `authz_missing_invalid`, `authz_denied_invalid`, `authz_expired_invalid`,
`authz_stale_invalid`, `authz_wrong_policy_invalid` and
`authz_issued_after_barrier_invalid` as negative controls.

## Projection and redaction

Projection recipes use
[`projection_readonly.registry.yaml`](../../contracts/examples/projection_readonly.registry.yaml)
and the projection scenarios:

```bash
causlane bundle compile \
  --registry contracts/examples/projection_readonly.registry.yaml \
  --out /tmp/causlane-cookbook/projection.bundle.json
causlane scenario emit-trace \
  --scenario contracts/scenarios/projection_success.scenario.yaml \
  --bundle /tmp/causlane-cookbook/projection.bundle.json \
  --out /tmp/causlane-cookbook/projection.trace.json
causlane replay verify \
  --bundle /tmp/causlane-cookbook/projection.bundle.json \
  --trace /tmp/causlane-cookbook/projection.trace.json \
  --explain
```

Use `projection_without_anchor_invalid`, `projection_anchor_wrong_fact_invalid`,
`projection_anchor_wrong_plan_invalid` and `projection_anchor_wrong_scope_invalid`
to check anchor failures. Projection redaction policy is defined by the shared
redaction classes in [`../07-security-and-authz.md`](../07-security-and-authz.md).

## Support bundle

Build a sanitized support artifact from the compiled bundle, trace and graph
snapshot:

```bash
causlane support-bundle build \
  --bundle /tmp/causlane-cookbook/release_promote.bundle.json \
  --trace /tmp/causlane-cookbook/release_promote.trace.json \
  --graph /tmp/causlane-cookbook/conflict.graph.yaml \
  --op release_promote:0 \
  --out /tmp/causlane-cookbook/support-bundle.json
```

Support bundles are derived diagnostics. They do not replace the raw compiled
bundle, replay trace, graph snapshot or formal receipts.

## Adapter boundary

Runtime adapters spend already-authorized work; they do not decide semantic
authority. Before wiring an adapter, keep these checks green:

```bash
causlane replay verify \
  --bundle /tmp/causlane-cookbook/release_promote.bundle.json \
  --trace /tmp/causlane-cookbook/release_promote.trace.json
causlane formal doctor --profile base --lane local_smoke
```

Adapter-specific certification belongs to S08. The S07 cookbook stops at the
stable control surfaces: bundle, scenario, replay, graph, support bundle and
formal doctor.
