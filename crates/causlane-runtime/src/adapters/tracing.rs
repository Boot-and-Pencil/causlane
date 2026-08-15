//! Runtime tracing adapter.
//!
//! This adapter projects causal protocol events into spans after the authoritative
//! protocol-history append succeeds. The projection is telemetry only: sink
//! failures are ignored.

use causlane_core::{
    trace_span_from_causal_protocol_event, CausalProtocolEvent, CausalProtocolEventId,
    CausalProtocolHistoryPort, TraceSpan,
};

/// Sink for derived trace spans.
pub trait TraceSinkPort {
    /// Sink-specific error type.
    type Error;

    /// Record a derived span.
    ///
    /// Errors are telemetry failures and must not affect protocol-history append semantics.
    fn record(&mut self, span: TraceSpan) -> Result<(), Self::Error>;
}

/// In-memory trace sink for tests and local composition.
#[derive(Default)]
pub struct InMemoryTraceSink {
    /// Spans recorded so far, in arrival order.
    pub spans: Vec<TraceSpan>,
}

impl TraceSinkPort for InMemoryTraceSink {
    type Error = core::convert::Infallible;

    fn record(&mut self, span: TraceSpan) -> Result<(), Self::Error> {
        self.spans.push(span);
        Ok(())
    }
}

/// Audit log wrapper that emits one derived trace span per successful append.
pub struct TraceProjectingCausalProtocolHistory<A, S> {
    protocol_history: A,
    trace_sink: S,
}

impl<A, S> TraceProjectingCausalProtocolHistory<A, S> {
    /// Create a tracing protocol-history wrapper.
    #[must_use]
    pub fn new(protocol_history: A, trace_sink: S) -> Self {
        Self {
            protocol_history,
            trace_sink,
        }
    }

    /// Borrow the wrapped causal protocol history.
    #[must_use]
    pub fn protocol_history(&self) -> &A {
        &self.protocol_history
    }

    /// Mutably borrow the wrapped causal protocol history.
    #[must_use]
    pub fn protocol_history_mut(&mut self) -> &mut A {
        &mut self.protocol_history
    }

    /// Borrow the trace sink.
    #[must_use]
    pub fn trace_sink(&self) -> &S {
        &self.trace_sink
    }

    /// Mutably borrow the trace sink.
    #[must_use]
    pub fn trace_sink_mut(&mut self) -> &mut S {
        &mut self.trace_sink
    }

    /// Split the wrapper back into its parts.
    #[must_use]
    pub fn into_parts(self) -> (A, S) {
        (self.protocol_history, self.trace_sink)
    }

    fn emit_spans(&mut self, spans: Vec<TraceSpan>)
    where
        S: TraceSinkPort,
    {
        for span in spans {
            drop(self.trace_sink.record(span));
        }
    }
}

impl<A, S> CausalProtocolHistoryPort for TraceProjectingCausalProtocolHistory<A, S>
where
    A: CausalProtocolHistoryPort,
    S: TraceSinkPort,
{
    type Error = A::Error;

    fn append_batch(
        &mut self,
        events: Vec<CausalProtocolEvent>,
    ) -> Result<Vec<CausalProtocolEventId>, Self::Error> {
        let spans = events
            .iter()
            .map(trace_span_from_causal_protocol_event)
            .collect();
        let event_ids = self.protocol_history.append_batch(events)?;
        self.emit_spans(spans);
        Ok(event_ids)
    }

    fn append(&mut self, event: CausalProtocolEvent) -> Result<CausalProtocolEventId, Self::Error> {
        let span = trace_span_from_causal_protocol_event(&event);
        let event_id = self.protocol_history.append(event)?;
        self.emit_spans(vec![span]);
        Ok(event_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{InMemoryTraceSink, TraceProjectingCausalProtocolHistory, TraceSinkPort};
    use crate::adapters::protocol_history::InMemoryCausalProtocolHistory;
    use causlane_core::{
        trace_span_from_causal_protocol_event, ActionId, CausalProtocolEvent,
        CausalProtocolEventId, CausalProtocolEventKind, CausalProtocolHistoryPort, TraceSpan,
    };

    #[derive(Debug, PartialEq, Eq)]
    struct ProtocolHistoryError;

    struct FailingCausalProtocolHistory;

    impl CausalProtocolHistoryPort for FailingCausalProtocolHistory {
        type Error = ProtocolHistoryError;

        fn append_batch(
            &mut self,
            _events: Vec<CausalProtocolEvent>,
        ) -> Result<Vec<CausalProtocolEventId>, Self::Error> {
            Err(ProtocolHistoryError)
        }

        fn append(
            &mut self,
            _event: CausalProtocolEvent,
        ) -> Result<CausalProtocolEventId, Self::Error> {
            Err(ProtocolHistoryError)
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct TelemetryError;

    #[derive(Default)]
    struct FailingTraceSink {
        attempts: usize,
    }

    impl TraceSinkPort for FailingTraceSink {
        type Error = TelemetryError;

        fn record(&mut self, _span: TraceSpan) -> Result<(), Self::Error> {
            self.attempts += 1;
            Err(TelemetryError)
        }
    }

    fn event() -> CausalProtocolEvent {
        event_kind("event-1", CausalProtocolEventKind::ExecutionStarted)
            .with_causation_id(CausalProtocolEventId("parent-event".to_owned()))
    }

    fn event_kind(id: &str, kind: CausalProtocolEventKind) -> CausalProtocolEvent {
        CausalProtocolEvent::new(
            CausalProtocolEventId(id.to_owned()),
            ActionId("action-1".to_owned()),
            kind,
        )
    }

    #[test]
    fn emits_derived_span_after_successful_protocol_history_append() {
        let source_event = event();
        let expected_span = trace_span_from_causal_protocol_event(&source_event);
        let expected_event = source_event.clone().with_event_index(0);
        let mut audit = TraceProjectingCausalProtocolHistory::new(
            InMemoryCausalProtocolHistory::default(),
            InMemoryTraceSink::default(),
        );

        let event_id = audit.append(source_event);

        assert_eq!(event_id, Ok(CausalProtocolEventId("event-1".to_owned())));
        assert_eq!(audit.protocol_history().events, vec![expected_event]);
        assert_eq!(audit.trace_sink().spans, vec![expected_span]);
    }

    #[test]
    fn does_not_emit_span_when_protocol_history_append_fails() {
        let mut protocol_history = TraceProjectingCausalProtocolHistory::new(
            FailingCausalProtocolHistory,
            InMemoryTraceSink::default(),
        );

        let result = protocol_history.append(event());

        assert_eq!(result, Err(ProtocolHistoryError));
        assert!(protocol_history.trace_sink().spans.is_empty());
    }

    #[test]
    fn telemetry_failure_is_fail_open_after_protocol_history_append() {
        let source_event = event();
        let expected_event = source_event.clone().with_event_index(0);
        let mut audit = TraceProjectingCausalProtocolHistory::new(
            InMemoryCausalProtocolHistory::default(),
            FailingTraceSink::default(),
        );

        let event_id = audit.append(source_event);

        assert_eq!(event_id, Ok(CausalProtocolEventId("event-1".to_owned())));
        assert_eq!(audit.protocol_history().events, vec![expected_event]);
        assert_eq!(audit.trace_sink().attempts, 1);
    }

    #[test]
    fn emits_derived_spans_after_successful_batch_append() {
        let barrier = event_kind("barrier", CausalProtocolEventKind::ExecutionBarrierLogged);
        let started = event_kind("started", CausalProtocolEventKind::ExecutionStarted)
            .with_causation_id(CausalProtocolEventId("barrier".to_owned()));
        let expected_spans = vec![
            trace_span_from_causal_protocol_event(&barrier),
            trace_span_from_causal_protocol_event(&started),
        ];
        let expected_events = vec![
            barrier.clone().with_event_index(0),
            started.clone().with_event_index(1),
        ];
        let mut audit = TraceProjectingCausalProtocolHistory::new(
            InMemoryCausalProtocolHistory::default(),
            InMemoryTraceSink::default(),
        );

        let event_ids = CausalProtocolHistoryPort::append_batch(&mut audit, vec![barrier, started]);

        assert_eq!(
            event_ids,
            Ok(vec![
                CausalProtocolEventId("barrier".to_owned()),
                CausalProtocolEventId("started".to_owned())
            ])
        );
        assert_eq!(audit.protocol_history().events, expected_events);
        assert_eq!(audit.trace_sink().spans, expected_spans);
    }

    #[test]
    fn does_not_emit_spans_when_batch_append_fails() {
        let mut audit = TraceProjectingCausalProtocolHistory::new(
            InMemoryCausalProtocolHistory::default(),
            InMemoryTraceSink::default(),
        );

        let result = CausalProtocolHistoryPort::append_batch(
            &mut audit,
            vec![
                event_kind("same", CausalProtocolEventKind::ExecutionBarrierLogged),
                event_kind("same", CausalProtocolEventKind::ExecutionStarted),
            ],
        );

        assert_eq!(
            result,
            Err(
                crate::adapters::protocol_history::CausalProtocolHistoryAdapterError::DuplicateEventId {
                    event_id: CausalProtocolEventId("same".to_owned())
                }
            )
        );
        assert!(audit.protocol_history().events.is_empty());
        assert!(audit.trace_sink().spans.is_empty());
    }
}
