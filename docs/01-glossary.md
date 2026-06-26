# Glossary

## Predicate

Canonical name of a meaningful intention.

Example:

```text
release.promote_candidate
report.generate
gate.approve
pipeline.retry_stage
```

A predicate is not a surface mechanism. `POST /release/promote`, `click button`, `enqueue job` and `run worker` are not predicates by themselves.

## Subject

The object, entity, aggregate or scope the action is about.

Examples:

```text
release_candidate:rc_123
environment:staging
dataset:customers_2026
approval_request:appr_456
```

## Circumstance

Context required to interpret the action.

Examples:

```text
requested_by;
source_surface;
policy_context;
idempotency_key;
budget;
reason;
required_evidence;
```

## ActionCall

A typed invocation of `predicate(subject, circumstance)`. It is the point where an ambiguous input becomes a named, typed intent.

## ActionPlan

A pure, deterministic compilation result from `ActionCall`.

```text
compile(ActionCall, typed snapshots) -> ActionPlan
```

The plan is data. It must not perform side effects.

## Op

The smallest executable unit inside an `ActionPlan`.

## Impact

A planned or observed consequence.

Common axes:

```text
planned / observed;
hard / soft;
direct / derived;
scoped / global;
reversible / compensatable / irreversible.
```

## EffectSignature

Machine-readable consequence signature of an op or action plan:

```text
reads;
writes;
produces;
requires;
invalidates;
conflict_domains;
freshness_policy;
merge_protocol.
```

## ConsequenceProfile

Classification that determines route, lifecycle, barrier and truth obligations.

Examples:

```text
RuntimeExecution;
ProjectionRead;
OversightMeta;
TopologyMeta;
EvidenceMeta;
OutsideKernel.
```

## AuditEvent

Typed authoritative event in the run-scoped event journal.

## Observed truth

The authoritative record of what actually happened. In this architecture, observed truth has one author: the append-only audit/event journal.

## Projection

A derived view of observed truth. A projection may format, aggregate or filter truth, but must not create truth.

## Witness

A causal reference proving that a required event/fact/stage existed before a target transition.

## Execution barrier

Durable pre-execution record that authorizes a side-effecting action to enter execution.

## Constraint

A rule that can restrict frontier admission or execution.

## Claim

A declared need for a resource, scope, capacity, token, quota or exclusivity.

## Lease

A granted claim. For hard effects, relevant leases must be durable and referenced from the execution barrier.

## Tier

Semantic lifecycle stage / authority level.

Examples:

```text
admission;
planning;
dispatch;
barrier;
execution;
observation;
projection;
closure.
```

## Lane

Capacity/capability/fairness slot inside a tier. A lane does not create semantics and must not bypass lifecycle invariants.

## Overlay

Additional policy/topology/evidence/oversight constraint attached to a route. Overlays may strengthen obligations but must not weaken kernel invariants.

## ReplayTrace

Executable record that can verify runtime behavior against the same contract: action call, plan, route, lifecycle events, witnesses, barriers, observed truth and projections.

## CompiledDispatchBundle

Generated contract artifact consumed by runtime, replay, tests and formal-model projections.

## Truth anchor

The observed-truth reference a projection is derived from (ADR-0010). Distinct
from a [Witness](#witness): a witness answers *"why is this transition allowed?"*,
an anchor answers *"from which committed observed truth is this projection
built?"*. A projection MUST carry at least one anchor pointing at a prior
`ObservedTruthCommitted` event of the matching action and plan.

## Plan hash

Canonical identity of a plan: `sha256:` + 64 lowercase hex of the SHA-256 over
the canonical serialization of the [Plan hash material](#plan-hash-material)
(ADR-0009). Every plan-bearing event of an action carries the same plan hash;
replay recomputes/cross-checks it and fails closed on mismatch. A literal
`sha256:TODO` is an intentionally invalid placeholder example, not an allowed
hash value; fixtures must use `sha256:` plus 64 lowercase hex characters.

## Plan hash material

The stable, serializable projection of everything that must influence a plan's
identity (bundle hash, planner fingerprint, subject/circumstance fingerprints,
route, consequence profile, ordered ops + effect signatures, planned impacts,
required witnesses, required claims, barrier/projection policy). Runtime-only
facts (event ids, timestamps, granted leases, observed results) are excluded.

## Impact set hash

A separate SHA-256 digest over the canonical planned-impact set. Approvals/gates
bind to `action_id + plan_hash + impact_set_hash` so an approval tracks the set
of hard consequences rather than every technical plan detail (I-009).

## Bundle hash

Content hash (`sha256:...`) of a compiled dispatch bundle. Feeds the plan hash
material and lets replay/codegen detect a changed contract.

## Merge protocol

An explicit, bundle-level contract that permits concurrent overlapping mutable
writes because their results can be deterministically joined (ADR-0012). The
default is *none*: with no `Verified` merge protocol, overlapping mutable write
scopes conflict and may not run concurrently. A merge protocol is never a string
on an op — it has an id, version, status and algebra.

## Lifecycle class

The path a predicate is routed through, derived from its consequence profile:
`execution_bearing`, `projection_only`, or `meta`. Carried in the plan hash
material and the registry/bundle.

## Execution capability

A scoped token a worker receives instead of a raw job payload, derived from a
valid execution barrier (ADR-0013): action id, plan hash, op index, lease ids,
allowed impacts, write scopes, policy decision id, expiry, executor identity.
