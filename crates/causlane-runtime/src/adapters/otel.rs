//! Optional OpenTelemetry adapter for derived trace spans.
//!
//! The adapter consumes [`TraceSpan`] values only. Audit-event classification
//! stays in `causlane-core`, so this module cannot drift into a second event
//! taxonomy.

use std::{convert::Infallible, error::Error, fmt, time::Duration};

use causlane_core::{AuthzDecision, TraceAttribute, TraceSpan, TraceSpanKind};
use opentelemetry::{
    logs::{AnyValue, LogRecord, Logger, LoggerProvider, Severity},
    metrics::{Counter, MeterProvider},
    trace::{Span, SpanKind, Status, Tracer, TracerProvider},
    KeyValue,
};
use opentelemetry_otlp::{
    ExporterBuildError, LogExporter, MetricExporter, Protocol, SpanExporter, WithExportConfig,
};
use opentelemetry_sdk::{
    error::OTelSdkError,
    logs::{SdkLogger, SdkLoggerProvider},
    metrics::SdkMeterProvider,
    trace::{SdkTracer, SdkTracerProvider},
};

use super::tracing::TraceSinkPort;

/// OpenTelemetry instrumentation scope used by this adapter.
pub const CAUSLANE_OTEL_SCOPE: &str = "causlane.runtime";

/// Counter name incremented for each emitted `TraceSpan`.
pub const CAUSLANE_TRACE_SPANS_TOTAL: &str = "causlane.trace_spans";

const TRACE_SPAN_EVENT_NAME: &str = "causlane.trace_span";

/// Programmatic OTLP/HTTP exporter configuration.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OtlpHttpConfig {
    /// Optional common OTLP endpoint override, for example `http://localhost:4318`.
    ///
    /// When unset, the OpenTelemetry environment/default endpoint resolution is
    /// used. Signal-specific `OTEL_EXPORTER_OTLP_*_ENDPOINT` variables still
    /// apply inside the OpenTelemetry exporter.
    pub endpoint: Option<String>,
    /// Optional export timeout override for all three signals.
    ///
    /// When unset, the OpenTelemetry environment/default timeout resolution is
    /// used.
    pub timeout: Option<Duration>,
}

/// Error returned when an explicit adapter flush fails.
#[derive(Debug)]
pub enum OpenTelemetryFlushError {
    /// Trace provider flush failed.
    Traces(OTelSdkError),
    /// Meter provider flush failed.
    Metrics(OTelSdkError),
    /// Logger provider flush failed.
    Logs(OTelSdkError),
}

impl fmt::Display for OpenTelemetryFlushError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Traces(error) => write!(f, "OpenTelemetry trace flush failed: {error}"),
            Self::Metrics(error) => write!(f, "OpenTelemetry metric flush failed: {error}"),
            Self::Logs(error) => write!(f, "OpenTelemetry log flush failed: {error}"),
        }
    }
}

impl Error for OpenTelemetryFlushError {}

/// OpenTelemetry sink for derived causlane spans.
///
/// The sink owns local SDK providers and does not install global OpenTelemetry
/// providers. Recording is telemetry-only and implements [`TraceSinkPort`] with
/// an infallible error type.
#[derive(Debug)]
pub struct OpenTelemetryTraceSink {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
    logger_provider: SdkLoggerProvider,
    tracer: SdkTracer,
    logger: SdkLogger,
    span_counter: Counter<u64>,
}

impl OpenTelemetryTraceSink {
    /// Build a sink from already configured SDK providers.
    #[must_use]
    pub fn from_providers(
        tracer_provider: SdkTracerProvider,
        meter_provider: SdkMeterProvider,
        logger_provider: SdkLoggerProvider,
    ) -> Self {
        let tracer = tracer_provider.tracer(CAUSLANE_OTEL_SCOPE);
        let meter = meter_provider.meter(CAUSLANE_OTEL_SCOPE);
        let logger = logger_provider.logger(CAUSLANE_OTEL_SCOPE);
        let span_counter = meter
            .u64_counter(CAUSLANE_TRACE_SPANS_TOTAL)
            .with_description("Count of causlane TraceSpan telemetry records")
            .build();

        Self {
            tracer_provider,
            meter_provider,
            logger_provider,
            tracer,
            logger,
            span_counter,
        }
    }

    /// Build a local OTLP/HTTP sink using protobuf over a blocking HTTP client.
    ///
    /// The feature intentionally does not enable tonic/grpc or async runtime
    /// integration. Exporter endpoint, timeout, headers and compression still
    /// follow the OpenTelemetry environment variables unless overridden by
    /// [`OtlpHttpConfig`].
    #[must_use = "building OTLP telemetry can fail and the caller must handle that"]
    pub fn from_otlp_http(config: &OtlpHttpConfig) -> Result<Self, ExporterBuildError> {
        let span_exporter =
            configure_otlp_http(SpanExporter::builder().with_http(), config).build()?;
        let metric_exporter =
            configure_otlp_http(MetricExporter::builder().with_http(), config).build()?;
        let log_exporter =
            configure_otlp_http(LogExporter::builder().with_http(), config).build()?;

        let tracer_provider = SdkTracerProvider::builder()
            .with_batch_exporter(span_exporter)
            .build();
        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(metric_exporter)
            .build();
        let logger_provider = SdkLoggerProvider::builder()
            .with_batch_exporter(log_exporter)
            .build();

        Ok(Self::from_providers(
            tracer_provider,
            meter_provider,
            logger_provider,
        ))
    }

    /// Force all owned providers to flush buffered telemetry.
    #[must_use = "flush failures indicate telemetry loss and should be observed"]
    pub fn force_flush(&self) -> Result<(), OpenTelemetryFlushError> {
        self.tracer_provider
            .force_flush()
            .map_err(OpenTelemetryFlushError::Traces)?;
        self.meter_provider
            .force_flush()
            .map_err(OpenTelemetryFlushError::Metrics)?;
        self.logger_provider
            .force_flush()
            .map_err(OpenTelemetryFlushError::Logs)?;
        Ok(())
    }

    fn emit(&mut self, span: &TraceSpan) {
        let attributes = otel_attributes(span);
        self.emit_trace_span(span, &attributes);
        self.emit_metric(span, &attributes);
        self.emit_log_record(span, &attributes);
    }

    fn emit_trace_span(&self, span: &TraceSpan, attributes: &[OtelAttribute]) {
        let mut otel_span = self
            .tracer
            .span_builder(span_name(span.span_kind))
            .with_kind(SpanKind::Internal)
            .with_attributes(attributes.iter().map(OtelAttribute::to_key_value))
            .start(&self.tracer);
        if span.span_kind == TraceSpanKind::Violation {
            otel_span.set_status(Status::error("causlane violation"));
        }
        otel_span.end();
    }

    fn emit_metric(&self, span: &TraceSpan, attributes: &[OtelAttribute]) {
        let metric_attributes = metric_attributes(span, attributes);
        self.span_counter.add(1, &metric_attributes);
    }

    fn emit_log_record(&self, span: &TraceSpan, attributes: &[OtelAttribute]) {
        let severity = severity(span.span_kind);
        let mut record = self.logger.create_log_record();
        record.set_event_name(TRACE_SPAN_EVENT_NAME);
        record.set_target(CAUSLANE_OTEL_SCOPE);
        record.set_severity_number(severity);
        record.set_severity_text(severity.name());
        record.set_body(AnyValue::from(TRACE_SPAN_EVENT_NAME));
        record.add_attributes(attributes.iter().map(OtelAttribute::to_log_attribute));
        self.logger.emit(record);
    }
}

impl TraceSinkPort for OpenTelemetryTraceSink {
    type Error = Infallible;

    fn record(&mut self, span: TraceSpan) -> Result<(), Self::Error> {
        self.emit(&span);
        Ok(())
    }
}

fn configure_otlp_http<B>(builder: B, config: &OtlpHttpConfig) -> B
where
    B: WithExportConfig,
{
    let builder = builder.with_protocol(Protocol::HttpBinary);
    let builder = match &config.endpoint {
        Some(endpoint) => builder.with_endpoint(endpoint.clone()),
        None => builder,
    };
    match config.timeout {
        Some(timeout) => builder.with_timeout(timeout),
        None => builder,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct OtelAttribute {
    key: &'static str,
    value: OtelAttributeValue,
}

impl OtelAttribute {
    fn bool(key: &'static str, value: bool) -> Self {
        Self {
            key,
            value: OtelAttributeValue::Bool(value),
        }
    }

    fn int(key: &'static str, value: i64) -> Self {
        Self {
            key,
            value: OtelAttributeValue::Int(value),
        }
    }

    fn string(key: &'static str, value: impl Into<String>) -> Self {
        Self {
            key,
            value: OtelAttributeValue::String(value.into()),
        }
    }

    fn to_key_value(&self) -> KeyValue {
        match &self.value {
            OtelAttributeValue::Bool(value) => KeyValue::new(self.key, *value),
            OtelAttributeValue::Int(value) => KeyValue::new(self.key, *value),
            OtelAttributeValue::String(value) => KeyValue::new(self.key, value.clone()),
        }
    }

    fn to_log_attribute(&self) -> (&'static str, AnyValue) {
        let value = match &self.value {
            OtelAttributeValue::Bool(value) => AnyValue::from(*value),
            OtelAttributeValue::Int(value) => AnyValue::from(*value),
            OtelAttributeValue::String(value) => AnyValue::from(value.clone()),
        };
        (self.key, value)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum OtelAttributeValue {
    Bool(bool),
    Int(i64),
    String(String),
}

fn otel_attributes(span: &TraceSpan) -> Vec<OtelAttribute> {
    let mut attributes = vec![
        OtelAttribute::string("causlane.trace_id", span.trace_id.0.clone()),
        OtelAttribute::string("causlane.span_id", span.span_id.0.clone()),
        OtelAttribute::string("causlane.action_id", span.action_id.0.clone()),
        OtelAttribute::string("causlane.event_id", span.event_id.0.clone()),
        OtelAttribute::string("causlane.event_kind", format!("{:?}", span.event_kind)),
        OtelAttribute::string("causlane.span_kind", span_kind_label(span.span_kind)),
    ];

    if let Some(parent_span_id) = &span.parent_span_id {
        attributes.push(OtelAttribute::string(
            "causlane.parent_span_id",
            parent_span_id.0.clone(),
        ));
    }
    if let Some(plan_hash) = &span.plan_hash {
        attributes.push(OtelAttribute::string(
            "causlane.plan_hash",
            plan_hash.as_str(),
        ));
    }
    if let Some(occurred_at) = span.occurred_at {
        push_u64(&mut attributes, "causlane.occurred_at_ms", occurred_at.0);
    }

    for attribute in &span.attributes {
        push_trace_attribute(&mut attributes, attribute);
    }

    attributes
}

fn push_trace_attribute(attributes: &mut Vec<OtelAttribute>, attribute: &TraceAttribute) {
    match attribute {
        TraceAttribute::EventIndex(value) => {
            push_u64(attributes, "causlane.event_index", *value);
        }
        TraceAttribute::WitnessCount(value) => {
            push_usize(attributes, "causlane.witness_count", *value);
        }
        TraceAttribute::WitnessRefCount(value) => {
            push_usize(attributes, "causlane.witness_ref_count", *value);
        }
        TraceAttribute::TruthAnchorCount(value) => {
            push_usize(attributes, "causlane.truth_anchor_count", *value);
        }
        TraceAttribute::LeaseCount(value) => {
            push_usize(attributes, "causlane.lease_count", *value);
        }
        TraceAttribute::ImpactSetHash(value) => {
            attributes.push(OtelAttribute::string(
                "causlane.impact_set_hash",
                value.0.clone(),
            ));
        }
        TraceAttribute::DrainFenceScope(value) => {
            attributes.push(OtelAttribute::string(
                "causlane.drain_fence_scope",
                value.0.clone(),
            ));
        }
        TraceAttribute::HasExecutionBarrier => {
            attributes.push(OtelAttribute::bool("causlane.has_execution_barrier", true));
        }
        TraceAttribute::HasAuthzDecision => {
            attributes.push(OtelAttribute::bool("causlane.has_authz_decision", true));
        }
        TraceAttribute::AuthzDecision(decision) => {
            attributes.push(OtelAttribute::string(
                "causlane.authz_decision",
                authz_decision_label(*decision),
            ));
        }
        TraceAttribute::HasExecutionCapability => {
            attributes.push(OtelAttribute::bool(
                "causlane.has_execution_capability",
                true,
            ));
        }
        TraceAttribute::ExecutionCapabilityOpIndex(value) => {
            attributes.push(OtelAttribute::int(
                "causlane.execution_capability_op_index",
                i64::from(*value),
            ));
        }
        TraceAttribute::ExecutionCapabilityLeaseCount(value) => {
            push_usize(
                attributes,
                "causlane.execution_capability_lease_count",
                *value,
            );
        }
        TraceAttribute::HasAttestedFact => {
            attributes.push(OtelAttribute::bool("causlane.has_attested_fact", true));
        }
    }
}

fn metric_attributes(span: &TraceSpan, attributes: &[OtelAttribute]) -> Vec<KeyValue> {
    attributes
        .iter()
        .filter(|attribute| metric_attribute_is_low_cardinality(attribute.key))
        .map(OtelAttribute::to_key_value)
        .chain([KeyValue::new(
            "causlane.metric_signal",
            span_kind_label(span.span_kind),
        )])
        .collect()
}

fn metric_attribute_is_low_cardinality(key: &str) -> bool {
    matches!(
        key,
        "causlane.event_kind" | "causlane.span_kind" | "causlane.authz_decision"
    )
}

fn push_usize(attributes: &mut Vec<OtelAttribute>, key: &'static str, value: usize) {
    match i64::try_from(value) {
        Ok(converted) => attributes.push(OtelAttribute::int(key, converted)),
        Err(_) => attributes.push(OtelAttribute::string(key, value.to_string())),
    }
}

fn push_u64(attributes: &mut Vec<OtelAttribute>, key: &'static str, value: u64) {
    match i64::try_from(value) {
        Ok(converted) => attributes.push(OtelAttribute::int(key, converted)),
        Err(_) => attributes.push(OtelAttribute::string(key, value.to_string())),
    }
}

fn span_name(kind: TraceSpanKind) -> String {
    format!("causlane.{}", span_kind_label(kind))
}

fn span_kind_label(kind: TraceSpanKind) -> &'static str {
    match kind {
        TraceSpanKind::Admission => "admission",
        TraceSpanKind::Planning => "planning",
        TraceSpanKind::Dispatch => "dispatch",
        TraceSpanKind::Barrier => "barrier",
        TraceSpanKind::Execution => "execution",
        TraceSpanKind::Observation => "observation",
        TraceSpanKind::Projection => "projection",
        TraceSpanKind::Lifecycle => "lifecycle",
        TraceSpanKind::Gate => "gate",
        TraceSpanKind::Constraint => "constraint",
        TraceSpanKind::Authorization => "authorization",
        TraceSpanKind::Drain => "drain",
        TraceSpanKind::Violation => "violation",
    }
}

fn authz_decision_label(decision: AuthzDecision) -> &'static str {
    match decision {
        AuthzDecision::Allow => "allow",
        AuthzDecision::Deny => "deny",
    }
}

fn severity(kind: TraceSpanKind) -> Severity {
    match kind {
        TraceSpanKind::Violation => Severity::Error,
        _ => Severity::Info,
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fmt};

    use super::{
        metric_attributes, otel_attributes, span_kind_label, span_name, OpenTelemetryTraceSink,
        CAUSLANE_TRACE_SPANS_TOTAL, TRACE_SPAN_EVENT_NAME,
    };
    use crate::adapters::tracing::TraceSinkPort;
    use causlane_core::{
        ActionId, AuditEventId, AuditEventKind, AuthzDecision, CorrelationId, ImpactSetHash,
        PlanHash, TraceAttribute, TraceSpan, TraceSpanId, TraceSpanKind,
    };
    use opentelemetry::{logs::AnyValue, trace::Status, Key, Value};
    use opentelemetry_sdk::{
        logs::{InMemoryLogExporter, SdkLoggerProvider},
        metrics::{data::AggregatedMetrics, InMemoryMetricExporter, SdkMeterProvider},
        trace::{InMemorySpanExporter, SdkTracerProvider},
    };

    #[derive(Debug)]
    struct TestError(String);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl Error for TestError {}

    fn test_error(message: impl Into<String>) -> Box<dyn Error> {
        Box::new(TestError(message.into()))
    }

    fn trace_span(kind: TraceSpanKind) -> Result<TraceSpan, Box<dyn Error>> {
        let plan_hash = PlanHash::new(format!("sha256:{}", "a".repeat(PlanHash::DIGEST_LEN)))
            .map_err(|error| test_error(format!("test plan hash rejected: {error:?}")))?;

        Ok(TraceSpan {
            span_id: TraceSpanId("audit:event-1".to_owned()),
            parent_span_id: Some(TraceSpanId("audit:parent-1".to_owned())),
            trace_id: CorrelationId("corr-1".to_owned()),
            action_id: ActionId("action-1".to_owned()),
            event_id: AuditEventId("event-1".to_owned()),
            event_kind: AuditEventKind::AuthzDecisionRecorded,
            span_kind: kind,
            plan_hash: Some(plan_hash),
            occurred_at: Some(causlane_core::Timestamp(123)),
            attributes: vec![
                TraceAttribute::EventIndex(7),
                TraceAttribute::WitnessCount(2),
                TraceAttribute::ImpactSetHash(ImpactSetHash("impact-1".to_owned())),
                TraceAttribute::HasAuthzDecision,
                TraceAttribute::AuthzDecision(AuthzDecision::Deny),
            ],
        })
    }

    #[test]
    fn attributes_preserve_causlane_identity_and_typed_values() -> Result<(), Box<dyn Error>> {
        let span = trace_span(TraceSpanKind::Authorization)?;

        let attributes = otel_attributes(&span);

        assert!(attributes.iter().any(|attribute| {
            attribute.to_key_value() == opentelemetry::KeyValue::new("causlane.trace_id", "corr-1")
        }));
        assert!(attributes.iter().any(|attribute| {
            attribute.to_key_value()
                == opentelemetry::KeyValue::new("causlane.parent_span_id", "audit:parent-1")
        }));
        assert!(attributes.iter().any(|attribute| {
            attribute.to_key_value() == opentelemetry::KeyValue::new("causlane.event_index", 7_i64)
        }));
        assert!(attributes.iter().any(|attribute| {
            attribute.to_key_value()
                == opentelemetry::KeyValue::new("causlane.authz_decision", "deny")
        }));
        assert!(attributes.iter().any(|attribute| {
            attribute.to_key_value()
                == opentelemetry::KeyValue::new("causlane.impact_set_hash", "impact-1")
        }));

        Ok(())
    }

    #[test]
    fn metric_attributes_keep_only_low_cardinality_dimensions() -> Result<(), Box<dyn Error>> {
        let span = trace_span(TraceSpanKind::Authorization)?;
        let attributes = otel_attributes(&span);

        let metric_attributes = metric_attributes(&span, &attributes);

        assert!(metric_attributes
            .iter()
            .any(|attribute| attribute.key == Key::from_static_str("causlane.span_kind")));
        assert!(metric_attributes
            .iter()
            .any(|attribute| attribute.key == Key::from_static_str("causlane.authz_decision")));
        assert!(!metric_attributes
            .iter()
            .any(|attribute| attribute.key == Key::from_static_str("causlane.event_id")));
        assert!(metric_attributes
            .iter()
            .any(|attribute| attribute.key == Key::from_static_str("causlane.metric_signal")));

        Ok(())
    }

    #[test]
    fn records_span_log_and_metric_with_in_memory_providers() -> Result<(), Box<dyn Error>> {
        let span_exporter = InMemorySpanExporter::default();
        let log_exporter = InMemoryLogExporter::default();
        let metric_exporter = InMemoryMetricExporter::default();

        let tracer_provider = SdkTracerProvider::builder()
            .with_simple_exporter(span_exporter.clone())
            .build();
        let logger_provider = SdkLoggerProvider::builder()
            .with_simple_exporter(log_exporter.clone())
            .build();
        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(metric_exporter.clone())
            .build();

        let mut sink = OpenTelemetryTraceSink::from_providers(
            tracer_provider,
            meter_provider,
            logger_provider,
        );
        match sink.record(trace_span(TraceSpanKind::Authorization)?) {
            Ok(()) => {}
            Err(error) => match error {},
        }
        sink.force_flush()?;

        let spans = span_exporter.get_finished_spans()?;
        let [recorded_span] = spans.as_slice() else {
            return Err(test_error(format!(
                "expected one span, got {}",
                spans.len()
            )));
        };
        assert_eq!(recorded_span.name, span_name(TraceSpanKind::Authorization));
        assert!(recorded_span.attributes.iter().any(|attribute| {
            attribute.key == Key::from_static_str("causlane.span_kind")
                && attribute.value == Value::from(span_kind_label(TraceSpanKind::Authorization))
        }));

        let logs = log_exporter.get_emitted_logs()?;
        let [recorded_log] = logs.as_slice() else {
            return Err(test_error(format!("expected one log, got {}", logs.len())));
        };
        assert_eq!(
            recorded_log.record.event_name(),
            Some(TRACE_SPAN_EVENT_NAME)
        );
        assert!(recorded_log.record.attributes_iter().any(|(key, value)| {
            *key == Key::from_static_str("causlane.authz_decision")
                && *value == AnyValue::from("deny")
        }));

        let metrics = metric_exporter.get_finished_metrics()?;
        assert!(metrics.iter().any(|resource_metrics| {
            resource_metrics.scope_metrics().any(|scope_metrics| {
                scope_metrics.metrics().any(|metric| {
                    metric.name() == CAUSLANE_TRACE_SPANS_TOTAL
                        && matches!(metric.data(), AggregatedMetrics::U64(_))
                })
            })
        }));

        Ok(())
    }

    #[test]
    fn violation_spans_are_marked_error() -> Result<(), Box<dyn Error>> {
        let span_exporter = InMemorySpanExporter::default();
        let tracer_provider = SdkTracerProvider::builder()
            .with_simple_exporter(span_exporter.clone())
            .build();
        let meter_provider = SdkMeterProvider::builder().build();
        let logger_provider = SdkLoggerProvider::builder().build();
        let mut sink = OpenTelemetryTraceSink::from_providers(
            tracer_provider,
            meter_provider,
            logger_provider,
        );

        match sink.record(trace_span(TraceSpanKind::Violation)?) {
            Ok(()) => {}
            Err(error) => match error {},
        }
        sink.force_flush()?;

        let spans = span_exporter.get_finished_spans()?;
        let [recorded_span] = spans.as_slice() else {
            return Err(test_error(format!(
                "expected one span, got {}",
                spans.len()
            )));
        };
        assert_eq!(
            recorded_span.status,
            Status::error("causlane violation"),
            "violation telemetry should be visible to tracing backends"
        );

        Ok(())
    }
}
