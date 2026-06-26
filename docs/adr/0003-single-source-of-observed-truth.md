# ADR-0003: Observed truth has one authority

- Status: accepted
- Date: 2026-06-05

## Context

Complex systems often allow UI state, worker memory, logs, queue state or projection tables to become de facto truth. This breaks replay and incident investigation.

## Decision

Only the append-only audit/event journal may be the authority for observed truth.

Derived surfaces include:

```text
execution graph;
UI projections;
logs;
metrics;
dashboards;
worker-local status;
scheduler cache.
```

## Consequences

All projections must anchor to observed-truth events. Runtime code must commit observed results through the audit path.

## Enforcement

- Replay rejects projection without anchor.
- Formal models assert no projection without observed-truth anchor.
- Runtime projection API requires anchor.
- Observability connector is read-only derived sink.
