# ADR-0008: Observability is derived, not authoritative

- Status: accepted
- Date: 2026-06-05

## Context

Logging/tracing is essential for operations, but logs often become accidental truth. This breaks replay and authority discipline.

## Decision

Observability connectors are derived sinks from audit/dispatch events.

```text
Audit/event journal = authority.
Logs/traces/metrics = derived observability.
```

## Consequences

Telemetry export failure must not affect correctness, except when the sink is explicitly modeled as durable audit.

## Enforcement

- Hard effects fail closed on audit append failure.
- Telemetry export failure increments health metrics / logs internal warning / drops under policy.
- Observability connectors cannot commit observed truth or mutate lifecycle.
