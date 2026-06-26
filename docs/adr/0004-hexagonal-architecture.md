# ADR-0004: Use hexagonal architecture

- Status: accepted
- Date: 2026-06-05

## Context

The dispatcher kernel must remain testable, replayable and formally modelable without real databases, queues, workers or network calls.

## Decision

Use hexagonal architecture:

```text
Adapters -> Application -> Domain
```

The domain/application kernel lives in `causlane-core`. Runtime integrations live in separate adapter crates.

## Consequences

The core can remain mostly sync, deterministic and dependency-light. Runtime ergonomics require extra adapter wiring.

## Enforcement

- `causlane-core` must not depend on Tokio, SQL, HTTP, tracing, policy engines or workflow engines.
- Ports live inward; adapters live outward.
- Adapter certification tests verify protocol compliance.
