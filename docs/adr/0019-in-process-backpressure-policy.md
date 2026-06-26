# ADR-0019: In-process backpressure policy is explicit per submit

## Status

Accepted.

## Context

M08.1 introduced bounded in-process partition queues. M09.2 added routed
admission coordination, and M09.4 needs an explicit overload policy without
turning the host-dispatch protocol into a durable scheduler or duplicating the
runtime admission path.

## Decision

`InProcessRuntime` exposes `InProcessBackpressurePolicy` with two modes:

- `Wait` waits for route admission and bounded ingress capacity;
- `FailFast` returns `RouteBusy` or `QueueFull` instead of waiting.

The existing `submit` and `submit_routed` APIs remain wait-mode entrypoints.
The existing `try_submit` and `try_submit_routed` APIs remain fail-fast
entrypoints. New `submit_with_backpressure` and
`submit_routed_with_backpressure` methods let hosts choose the same policy
explicitly per call.

All entrypoints share one internal submit helper for host validation, route
validation, route coordination, command enqueue and partition admission response
handling. Wait-mode validates the route and reserves primary ingress capacity
before acquiring route permits. Fail-fast mode acquires route permits first and
then uses immediate `try_send`, preserving its non-blocking overload surface.

## Consequences

- `QueueFull` and `RouteBusy` remain the stable runtime diagnostics for
  overload decisions.
- Wait-mode routed submit no longer couples participant route permits to a
  saturated primary ingress channel.
- `PartitionKey` is the current tenant/backpressure bucket boundary supplied by
  the host.
- `InProcessRuntimeConfig` is unchanged, so existing config construction keeps
  compiling.
- No host-dispatch v2 schema, replay trace schema, Formal IR schema, generated
  model or scenario changes are introduced.
- Priority scheduling, automatic tenant derivation, background shedding workers,
  distributed queue storage, `RateLimit` enforcement, retry policy and numeric
  operational SLO threshold enforcement remain outside M09.4.
