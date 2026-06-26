# ADR-0018: Batched audit durability stays on the audit port

## Status

Accepted.

## Context

M08.2 introduced append-only in-memory, SQLite and Postgres audit adapters with
adapter-specific batch helpers. M09.3 needs group commit for audit/barrier
writes without creating a second journal, a second audit API or a replay payload
storage schema.

## Decision

`AuditLogPort` is the single audit boundary and now exposes two write shapes:

- `append` for immediate single-event writes;
- `append_batch` for all-or-nothing ordered batch writes.

Adapters must validate the full batch before mutating state. The existing
`AuditAppendState::prepare_batch` remains the shared admission implementation
for unique event ids, monotonic event indexes and overflow checks. SQLite and
Postgres persist prepared batches in one transaction.

Tracing is derived from successful audit writes only. `TraceProjectingAuditLog`
may precompute spans from the caller's events, but it emits spans only after the
inner audit append or batch append succeeds.

## Consequences

- Write-ahead order is explicit: callers place barrier events before execution
  events in the same batch when batching hard-effect audit writes.
- Batch failure is fail-closed and leaves adapter state unchanged.
- No replay trace schema, Formal IR schema, generated model or durable event
  payload schema changes in M09.3.
- Distributed group commit coordination, retry policy and numeric SLO threshold
  enforcement remain outside M09.3.
