//! Runtime tracing adapter.
//!
//! This adapter projects audit events into spans after the authoritative audit
//! append succeeds. The projection is telemetry only: sink failures are ignored.

use causlane_core::{
    trace_span_from_audit_event, AuditEvent, AuditEventId, AuditLogPort, TraceSpan,
};

/// Sink for derived trace spans.
pub trait TraceSinkPort {
    /// Sink-specific error type.
    type Error;

    /// Record a derived span.
    ///
    /// Errors are telemetry failures and must not affect audit append semantics.
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
pub struct TraceProjectingAuditLog<A, S> {
    audit_log: A,
    trace_sink: S,
}

impl<A, S> TraceProjectingAuditLog<A, S> {
    /// Create a tracing audit wrapper.
    #[must_use]
    pub fn new(audit_log: A, trace_sink: S) -> Self {
        Self {
            audit_log,
            trace_sink,
        }
    }

    /// Borrow the wrapped audit log.
    #[must_use]
    pub fn audit_log(&self) -> &A {
        &self.audit_log
    }

    /// Mutably borrow the wrapped audit log.
    #[must_use]
    pub fn audit_log_mut(&mut self) -> &mut A {
        &mut self.audit_log
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
        (self.audit_log, self.trace_sink)
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

impl<A, S> AuditLogPort for TraceProjectingAuditLog<A, S>
where
    A: AuditLogPort,
    S: TraceSinkPort,
{
    type Error = A::Error;

    fn append_batch(&mut self, events: Vec<AuditEvent>) -> Result<Vec<AuditEventId>, Self::Error> {
        let spans = events.iter().map(trace_span_from_audit_event).collect();
        let event_ids = self.audit_log.append_batch(events)?;
        self.emit_spans(spans);
        Ok(event_ids)
    }

    fn append(&mut self, event: AuditEvent) -> Result<AuditEventId, Self::Error> {
        let span = trace_span_from_audit_event(&event);
        let event_id = self.audit_log.append(event)?;
        self.emit_spans(vec![span]);
        Ok(event_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{InMemoryTraceSink, TraceProjectingAuditLog, TraceSinkPort};
    use crate::adapters::audit::InMemoryAuditLog;
    use causlane_core::{
        trace_span_from_audit_event, ActionId, AuditEvent, AuditEventId, AuditEventKind,
        AuditLogPort, TraceSpan,
    };

    #[derive(Debug, PartialEq, Eq)]
    struct AuditError;

    struct FailingAuditLog;

    impl AuditLogPort for FailingAuditLog {
        type Error = AuditError;

        fn append_batch(
            &mut self,
            _events: Vec<AuditEvent>,
        ) -> Result<Vec<AuditEventId>, Self::Error> {
            Err(AuditError)
        }

        fn append(&mut self, _event: AuditEvent) -> Result<AuditEventId, Self::Error> {
            Err(AuditError)
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

    fn event() -> AuditEvent {
        event_kind("event-1", AuditEventKind::ExecutionStarted)
            .with_causation_id(AuditEventId("parent-event".to_owned()))
    }

    fn event_kind(id: &str, kind: AuditEventKind) -> AuditEvent {
        AuditEvent::new(
            AuditEventId(id.to_owned()),
            ActionId("action-1".to_owned()),
            kind,
        )
    }

    #[test]
    fn emits_derived_span_after_successful_audit_append() {
        let source_event = event();
        let expected_span = trace_span_from_audit_event(&source_event);
        let expected_event = source_event.clone().with_event_index(0);
        let mut audit =
            TraceProjectingAuditLog::new(InMemoryAuditLog::default(), InMemoryTraceSink::default());

        let event_id = audit.append(source_event);

        assert_eq!(event_id, Ok(AuditEventId("event-1".to_owned())));
        assert_eq!(audit.audit_log().events, vec![expected_event]);
        assert_eq!(audit.trace_sink().spans, vec![expected_span]);
    }

    #[test]
    fn does_not_emit_span_when_audit_append_fails() {
        let mut audit = TraceProjectingAuditLog::new(FailingAuditLog, InMemoryTraceSink::default());

        let result = audit.append(event());

        assert_eq!(result, Err(AuditError));
        assert!(audit.trace_sink().spans.is_empty());
    }

    #[test]
    fn telemetry_failure_is_fail_open_after_audit_append() {
        let source_event = event();
        let expected_event = source_event.clone().with_event_index(0);
        let mut audit =
            TraceProjectingAuditLog::new(InMemoryAuditLog::default(), FailingTraceSink::default());

        let event_id = audit.append(source_event);

        assert_eq!(event_id, Ok(AuditEventId("event-1".to_owned())));
        assert_eq!(audit.audit_log().events, vec![expected_event]);
        assert_eq!(audit.trace_sink().attempts, 1);
    }

    #[test]
    fn emits_derived_spans_after_successful_batch_append() {
        let barrier = event_kind("barrier", AuditEventKind::ExecutionBarrierLogged);
        let started = event_kind("started", AuditEventKind::ExecutionStarted)
            .with_causation_id(AuditEventId("barrier".to_owned()));
        let expected_spans = vec![
            trace_span_from_audit_event(&barrier),
            trace_span_from_audit_event(&started),
        ];
        let expected_events = vec![
            barrier.clone().with_event_index(0),
            started.clone().with_event_index(1),
        ];
        let mut audit =
            TraceProjectingAuditLog::new(InMemoryAuditLog::default(), InMemoryTraceSink::default());

        let event_ids = AuditLogPort::append_batch(&mut audit, vec![barrier, started]);

        assert_eq!(
            event_ids,
            Ok(vec![
                AuditEventId("barrier".to_owned()),
                AuditEventId("started".to_owned())
            ])
        );
        assert_eq!(audit.audit_log().events, expected_events);
        assert_eq!(audit.trace_sink().spans, expected_spans);
    }

    #[test]
    fn does_not_emit_spans_when_batch_append_fails() {
        let mut audit =
            TraceProjectingAuditLog::new(InMemoryAuditLog::default(), InMemoryTraceSink::default());

        let result = AuditLogPort::append_batch(
            &mut audit,
            vec![
                event_kind("same", AuditEventKind::ExecutionBarrierLogged),
                event_kind("same", AuditEventKind::ExecutionStarted),
            ],
        );

        assert_eq!(
            result,
            Err(
                crate::adapters::audit::AuditAdapterError::DuplicateEventId {
                    event_id: AuditEventId("same".to_owned())
                }
            )
        );
        assert!(audit.audit_log().events.is_empty());
        assert!(audit.trace_sink().spans.is_empty());
    }
}
