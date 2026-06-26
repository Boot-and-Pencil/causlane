# ADR-0009: Plan hash canonicalization

- Status: accepted
- Date: 2026-06-05
- Supersedes: -
- Superseded by: -

## Context

An `ActionPlan` must be identified by a stable digest so that planning, dispatch, barriers, execution and observed results all refer to the same compiled intent, and so replay can detect drift. Hashing the raw `ActionPlan` is unsafe: it mixes compile-time intent with runtime-only data (granted leases, selected witness event ids, observed results, timestamps, worker ids, route rationale text), which makes the digest unstable and breaks deterministic replay. Approval/gate binding (invariant I-009) also needs a stable anchor to the set of hard consequences, not to every technical plan detail.

## Decision

The plan hash MUST be:

```text
plan_hash = "sha256:" + lowercase_hex(SHA-256(canonical_serialization(PlanHashMaterial)))
```

`PlanHashMaterial` MUST be a separate, stable struct (never the raw `ActionPlan`). It MUST contain:

```text
hash_schema_version;
bundle_id / bundle_version / bundle_hash;
planner_id / planner_version / planner_fingerprint;
action_id;
predicate + predicate_version;
subject_fingerprint (ContentHash);
circumstance_fingerprint (ContentHash);
consequence_profile;
lifecycle_class;
route_id;
ordered ops (CanonicalOp);
planned_impacts (CanonicalImpact);
required_witnesses;
required_claims;
barrier_policy;
projection_policy.
```

The digest MUST include: `action_id`; predicate + version; bundle hash/version; planner fingerprint/version; subject and circumstance fingerprints; route; consequence profile; ordered ops; effect signatures; planned impacts; required witnesses; required claims; barrier/projection policy. If a constraint snapshot/fact influences COMPILE (not just dispatch), its stable reference `snapshot_id` + `snapshot_hash` MUST be included (see ADR-0005).

The digest MUST NOT include: the `plan_hash` itself; audit event ids; timestamps/wall-clock; leases actually granted; witness event ids actually selected; observed results; runtime route rationale text; logs/telemetry; worker ids; a dispatch-only constraint snapshot.

Order of operations MUST be:

```text
1. planner builds ActionPlan without plan_hash;
2. runtime builds PlanHashMaterial;
3. canonical serialization;
4. SHA-256 -> plan_hash;
5. ActionPlanned / DispatchLogged / Barrier / Execution / Observed events all carry the same plan_hash;
6. replay recomputes and is fail-closed on mismatch.
```

Approval binding (I-009): an approval/gate MUST bind to `action_id` + `plan_hash` + `impact_set_hash`, where `impact_set_hash` is a separate SHA-256 digest of the canonical planned impacts, so approval tracks the set of hard consequences rather than every technical plan detail. The barrier carries `impact_set_hash` (see ADR-0013), and `bundle_hash` feeds the material (see ADR-0014).

## Consequences

Easier:

```text
deterministic replay;
stable approval binding;
drift detection.
```

Harder:

```text
must maintain a canonical serialization;
must maintain a frozen include/exclude list.
```

## Enforcement

- Docs: this ADR is the frozen include/exclude list; changing it requires bumping `hash_schema_version`.
- Formal: models assert `plan_hash` is a function of `PlanHashMaterial` only, and that runtime-only fields are not reachable from the digest.
- Replay: recomputes `plan_hash` and `impact_set_hash` and is fail-closed on mismatch; the `sha256:TODO` placeholder fixture is rejected.
- Runtime: forbids constructing a barrier/execution event whose `plan_hash` differs from the planned one; observed-truth events anchor to the same `plan_hash` (see ADR-0003).
- Tests: assert determinism (same material -> same hash) and sensitivity (op/effect/impact/witness/claim change -> hash change; runtime-only id change -> no change).
