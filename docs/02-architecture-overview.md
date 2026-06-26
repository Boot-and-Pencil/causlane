# Architecture overview

## Core chain

```text
Input event / request / command
  -> semantic normalization
  -> ActionCall
  -> ActionPlan
  -> dispatch admission
  -> consequence profile
  -> route + lifecycle
  -> dispatch log
  -> execution barrier, if needed
  -> execution / projection / overlay
  -> observed truth / projection / closure
  -> replay / audit / UI
```

## Single source of observed truth

Only the audit/event journal may be the authority for observed truth.

The following are derived and must not become truth:

- UI state;
- worker-local memory;
- scheduler cache;
- logs;
- metrics;
- execution graph;
- dashboards;
- projections.

## Dispatcher kernel

The dispatcher kernel is intentionally small. It knows:

```text
consequence profiles;
lifecycle classes;
transition guards;
required barriers;
truth/projection rules;
overlay admissibility;
forbidden states;
conflict/write-scope exclusion;
constraint/witness/lease requirements.
```

It does not know product-specific business semantics.

## Core invariant families

```text
No downstream action without dispatch log.
No hard execution without execution barrier.
No observed truth without execution.
No projection without observed-truth anchor.
No overlay may weaken base obligations.
No conflicting mutable frontier without merge protocol.
No graph/projection/log may become runtime truth.
```

## Consequence profiles

### RuntimeExecution

Execution-bearing action that can produce hard effects.

Requires:

```text
admission;
planning;
dispatch log;
execution barrier;
execution;
observed truth commit;
lifecycle closure.
```

### ProjectionRead

Derived read/projection over existing truth.

Requires:

```text
admission;
planning;
dispatch log;
observed-truth anchor;
projection;
closure.
```

Must not commit observed truth.

### OversightMeta

Approval, review, escalation, pause/replan request.

Can attach obligations. Must not perform hard effects directly.

### TopologyMeta

Scheduling/topology constraint: conflict domain, lease, drain, budget, host/resource adaptation.

Can constrain frontier. Must not become scheduler-authoritative truth.

### EvidenceMeta

Evidence, witness, proof target, replay binding, route rationale.

Can bind facts. Must not execute.

## Lifecycle

For execution-bearing actions:

```text
admitted
  -> planned
  -> dispatch_logged
  -> execution_barrier_logged
  -> executing
  -> observed
  -> projected
  -> closed
```

For projection-only actions:

```text
admitted
  -> planned
  -> dispatch_logged
  -> projected
  -> closed
```

## Execution graph

The execution graph is a derived causal projection:

```text
audit facts + compiled contracts + active plans + snapshots
  -> execution graph
```

It is used for:

- safe parallelism;
- dependency resolution;
- conflict detection;
- drain;
- cancellation;
- supersession;
- explanation;
- replay;
- formal checks.

It is not an authority surface.

## Contract layer (implemented)

The chain above is backed by a typed, content-addressed contract layer (see the
[contract hardening plan](11-contract-hardening-plan.md) and ADR-0009…0014):

```text
registry.yaml -> CompiledDispatchBundle (+ bundle_hash)      [causlane-contracts]
ActionPlan    -> PlanHashMaterial -> plan_hash, impact_set_hash [ADR-0009]
AuditEvent    -> witnesses + typed truth anchors + leases     [causlane-core, ADR-0010/0013]
ReplayTrace   -> verify (I-001/I-002/I-003/I-006/I-008)        [causlane-replay]
```

The kernel (`causlane-core`) stays pure; parsing and hashing are boundary
concerns in `causlane-contracts`. Formal-model projections of this layer are
generated from the bundle, not hand-written (ADR-0014), and are deferred until
the contract-hardening gate passes.
