# ADR-0035: Causal protocol history is not operational audit

- Status: accepted
- Date: 2026-08-16
- Supersedes: N/A
- Superseded by: N/A

## Context

Causlane needs an append-only, replayable record of protocol transitions,
barriers, witnesses, leases and observed truth. Calling that record a generic
"audit log" obscures its bounded responsibility and can be mistaken for a
product-wide operational-audit authority.

The durable event representation, stable kind tokens, replay rules and storage
history are already authoritative protocol data. This decision changes the
public responsibility vocabulary without changing those persisted semantics.

## Decision

Causlane owns causal protocol history through
`CausalProtocolEvent`, `CausalProtocolEventKind` and
`CausalProtocolHistoryPort`. Runtime adapters live under
`causlane_runtime::adapters::protocol_history`.

Causlane does not own product operational-audit records, telemetry contracts,
custody evidence or reporting contracts. Hosts may reference those external
records through host-facing references such as `audit_ref`, but Causlane does
not define their payloads or storage.

This is a pre-release clean break. The superseded Rust names, module paths and
feature flags are removed without aliases or compatibility features. Existing
stable event-kind tokens, replay payloads and the `causlane_audit_events` SQL
table remain byte- and history-compatible.

## Consequences

- Public names state the protocol-local responsibility directly.
- Product operational audit remains owned by the host's canonical contracts and
  services.
- Existing database history and replay evidence do not require migration.
- Downstream source consumers must update to the new API before the first public
  compatibility window.

## Alternatives considered

Keeping the old names with documentation was rejected because the public API
would continue to imply global audit ownership. Type aliases and duplicate
feature flags were rejected because they would preserve two names for one
authority and extend the ambiguity indefinitely.

## Verification / enforcement

```text
Docs:    architecture, glossary, runtime and release documentation name causal protocol history.
Formal:  generated targets continue to derive from the unchanged protocol event tokens and replay inputs.
Replay:  existing scenarios and stable kind tokens remain unchanged and pass the replay suite.
Runtime: in-memory, SQLite and PostgreSQL adapters preserve append order and the existing SQL table.
Tests:   public API, adapter certification, feature-matrix and stale-generation checks reject old Rust names.
```
