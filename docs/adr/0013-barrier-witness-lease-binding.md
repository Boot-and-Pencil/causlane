# ADR-0013: Execution barrier binds witnesses, leases and impact set

- Status: accepted
- Date: 2026-06-05
- Supersedes: -
- Superseded by: -

## Context

A hard effect changes the world. Without durable, write-ahead evidence, replay cannot prove the effect was legal at the moment it ran, and execution can drift from the plan, the resolved facts and the granted leases that justified it.

We already bind plans and impact via `impact_set_hash`/`plan_hash` (ADR-0009), require fresh authz (ADR-0011) and arbitrate leases against conflicts (ADR-0012). What is missing is a single durable artifact that ties concrete evidence to the act of execution. See also invariants I-001/I-006/I-009.

## Decision

A hard effect MUST NOT execute without a durable, write-ahead `ExecutionBarrier` that binds concrete evidence. The executor MUST receive a scoped `ExecutionCapability` derived from a valid barrier, never a raw job payload.

`ExecutionBarrier` fields:

```text
barrier_id
action_id
plan_hash
op_indexes
impact_set_hash
witnesses          (Vec<WitnessRef>)
leases             (Vec<LeaseRef>)
constraint_snapshot_id   (optional)
authz_decision_refs
```

`WitnessRef`:

```text
event_id
witness_kind       (ObservedFact | GateApproval | AuthzDecision | ConstraintDecision | ExternalEvidence)
requirement_id
selector_id
subject_scope
fact_kind
binding            { action_id, plan_hash, op_index, impact_set_hash }
event_hash         (optional)
```

`LeaseRef`:

```text
lease_id
resource
scope
mode
amount
holder_action_id
holder_plan_hash
holder_op_index
epoch              (ConstraintEpoch)
expires_at
lease_event_id
```

Barrier flow:

```text
1  plan declares required witnesses + resource claims.
2  dispatcher resolves witness selectors against the audit view.
3  dispatcher evaluates constraints/authz.
4  lease manager grants leases.
5  lease grants recorded as audit events or in the same durable transaction.
6  ExecutionBarrierLogged references witness refs + lease refs + impact_set_hash.
7  executor gets a scoped capability derived from the barrier.
8  replay validates witness existence, causal order, plan binding, impact binding, lease coverage, and absence of active conflicts.
```

Normative rule: `ExecutionStarted(action_id, plan_hash)` is invalid unless a prior `ExecutionBarrierLogged` exists with the same `action_id` + `plan_hash` and all required witnesses/leases were valid at barrier time.

## Consequences

Easier:

- Replay can prove side-effect legality.
- Capability-scoped execution.

Harder:

- Richer event payload.
- A barrier validator is required.

## Enforcement

- Runtime constructs `ExecutionCapability` only from a valid barrier.
- Replay enforces the normative rule, plan/impact binding and lease coverage, and rejects execution with no prior `ExecutionBarrierLogged`.
- Replay rejects barriers carrying stale witnesses or expired/conflicting leases (ADR-0011, ADR-0012).
- Tests cover execution-without-barrier and stale-witness/lease.
