# Tracing connector

M07.3 exposes structured spans as a derived observability projection from the
causal protocol history. The causal protocol history remains the only authority
for observed truth; spans are diagnostic views over already-recorded events.

## Projection

Every `CausalProtocolEvent` maps to one `TraceSpan` in `causlane-core`. The span
id is derived from the causal protocol event id, the parent span is derived from
`causation_id`, and the trace id is derived from `correlation_id`. Optional
protocol-history fields become typed attributes rather than a string-keyed
metadata map.

## Runtime adapter

`TraceProjectingCausalProtocolHistory` wraps any `CausalProtocolHistoryPort` and `TraceSinkPort`.

1. Append the causal protocol event to the authoritative causal protocol history.
2. If the append succeeds, project the event with `trace_span_from_causal_protocol_event`.
3. Record the span in the sink.
4. Ignore sink errors because telemetry is fail-open.

If the protocol-history append fails, no span is recorded.

## Exporters

The runtime adapter does not add an OpenTelemetry exporter and does not depend
on the Rust `tracing` ecosystem. M07.4 can serialize the existing `TraceSpan`
model into OTLP without duplicating event-kind classification.
