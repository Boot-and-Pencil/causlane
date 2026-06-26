# Tracing connector

M07.3 exposes structured spans as a derived observability projection from the
audit/event journal. The audit journal remains the only authority for observed
truth; spans are diagnostic views over already-recorded events.

## Projection

Every `AuditEvent` maps to one `TraceSpan` in `causlane-core`. The span id is
derived from the audit event id, the parent span is derived from `causation_id`,
and the trace id is derived from `correlation_id`. Optional audit fields become
typed attributes rather than a string-keyed metadata map.

## Runtime adapter

`TraceProjectingAuditLog` wraps any `AuditLogPort` and `TraceSinkPort`.

1. Append the audit event to the authoritative audit log.
2. If the append succeeds, project the event with `trace_span_from_audit_event`.
3. Record the span in the sink.
4. Ignore sink errors because telemetry is fail-open.

If the audit append fails, no span is recorded.

## Exporters

The runtime adapter does not add an OpenTelemetry exporter and does not depend
on the Rust `tracing` ecosystem. M07.4 can serialize the existing `TraceSpan`
model into OTLP without duplicating event-kind classification.
