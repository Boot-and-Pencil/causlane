# ADR-0007: Integrate with existing Rust ecosystem instead of replacing it

- Status: accepted
- Date: 2026-06-05

## Context

There are existing tools for jobs, workflows, policy, persistence and observability. Rebuilding them would be a bad NIH outcome.

## Decision

`causlane` will integrate through adapters rather than replacing mature runtime primitives.

Candidate adapters:

```text
Jobs: Apalis, Fang.
Durable workflows: Restate, Temporal, Dapr, Conductor.
AuthZ: Cedar, Casbin, AuthZEN, OpenFGA, SpiceDB, OPA.
Observability: tracing, OpenTelemetry.
Persistence: SQLite/Postgres audit adapters.
```

## Consequences

The project stays focused on semantic dispatch. Users keep choice of execution backend.

## Enforcement

Core crates must not import execution backend dependencies. Adapter crates may.
