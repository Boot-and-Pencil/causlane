# ADR-0010: Truth anchor is distinct from witness

- Status: accepted
- Date: 2026-06-05
- Supersedes: -
- Superseded by: -

## Context

A witness answers "why is this transition allowed?" — it carries causal or authorization evidence. An anchor answers "from which observed truth is this projection derived?". Reusing the `witnesses` vector for both roles conflates distinct semantics and makes projection provenance impossible to validate precisely during replay. Per ADR-0003, every projection must anchor to observed truth; that anchor must be explicit and typed, not inferred from witnesses.

## Decision

A projection's truth anchor MUST be a separate, typed field `anchors: Vec<TruthAnchor>` on `AuditEvent`. It MUST NOT be reused from the `witnesses` vector. An anchor MAY coincidentally also be a causal witness, but the two fields MUST remain separate.

`TruthAnchor` fields:

```text
event_id;
action_id;
plan_hash;
fact_kind (optional);
scope (optional);
event_hash (optional).
```

Rules:

- `ProjectionEmitted` MUST have `anchors.len() >= 1`.
- Each anchor MUST refer to a PRIOR `ObservedTruthCommitted` event.
- Anchors MUST match the `action_id` / `plan_hash` / `fact` / `scope` constraints where the projection policy specifies them.
- `witnesses` MAY include the same event, but the anchor stays explicit.
- `ExecutionBarrierLogged` uses `witnesses` / leases, NOT `anchors`.
- `ObservedTruthCommitted` normally has no anchors, unless it is itself derived observed truth.

## Consequences

Replay can validate projection provenance precisely, and the semantics of each field are clearer. The event model and fixtures must now carry both fields.

## Enforcement

- Replay MUST stop using `event.witnesses.is_empty()` as the projection-anchor check; it MUST instead require `anchors` referencing a prior `ObservedTruthCommitted` of matching `action_id` / `plan_hash`.
- Formal models assert that every `ProjectionEmitted` carries an anchor to a prior matching `ObservedTruthCommitted` (see ADR-0003, invariant I-003).
- Runtime projection API requires `anchors`.
- The `release_promote.trace.json` fixture is updated to use an `anchors` array.
- Tests cover projection-without-anchor and anchor-to-wrong-kind.
