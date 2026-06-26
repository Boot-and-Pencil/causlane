# ADR-0016: Stable host-facing dispatch API v1

- Status: proposed
- Date: 2026-06-22
- Supersedes: none
- Related: ADR-0004, ADR-0005, ADR-0008, ADR-0011, ADR-0013, ADR-0015

## Context

External host projects need to integrate with Causlane before the full production dispatcher runtime is complete. The existing repository exposes strong internal concepts, but binding a host application directly to action planning, barriers, leases, witnesses or replay types would freeze the wrong layer and make future dispatcher evolution expensive.

One active host, Refinery/M-CR Stage 8, needs a dispatcher seam for cross-cutting concerns: linear task execution, RBAC-before-dispatch, immutable config snapshots, secret-safe payload references, session/correlation context, observability, retry, and later replacement by a richer dispatcher.

## Decision

Introduce a small, versioned host-facing API named `causlane.host-dispatch.v1` in `causlane-core` and a deterministic in-memory reference implementation in `causlane-runtime`.

The API is intentionally conservative:

```text
HostDispatchPort
HostEffectHandler
HostDispatchContext
HostTaskSpec
HostDispatcherCapabilities
HostDrainOutcome
HostDispatchError
```

The linear reference implementation is `LinearHostDispatcher`. It supports admission validation, dependency readiness, at-most-one-task drain, and idempotency-key duplicate suppression. It does not perform host effects itself; all execution goes through `HostEffectHandler`.

## Ownership boundary

Causlane owns:

```text
host API version validation;
queue admission result;
dependency readiness for the reference implementation;
drain ticket/outcome shape;
future ability to back the same API with richer internals.
```

The host owns:

```text
RBAC and hard-deny policy;
secret resolution and redaction;
config snapshots;
session lifecycle;
idempotency policy before hard effects;
actual effect execution;
operator/audit evidence;
product-specific kill switches.
```

## Consequences

Benefits:

```text
host projects can depend on a stable small seam;
Causlane internals remain free to evolve;
Refinery can ship a linear stub now and swap runtime later;
parallelism remains explicitly disabled until proven safe;
formal and replay evidence can attach behind the same boundary later.
```

Costs:

```text
one additional compatibility layer to maintain;
initial API is intentionally underpowered;
hosts must keep their own policy and effect discipline instead of assuming Causlane enforces product rules.
```

## Compatibility rule

Any breaking change to `causlane.host-dispatch.v1` requires either:

1. a new API version string, or
2. a documented adapter that preserves v1 behavior for existing host projects.

## Non-goals

This ADR does not stabilize internal planner, lease, barrier, witness, replay, projection, formal IR, or runtime partitioning APIs.
