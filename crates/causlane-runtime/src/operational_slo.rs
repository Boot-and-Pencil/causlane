//! Operational SLO measurement contract for runtime/replay readiness.
//!
//! Release profiles and host telemetry must be able to measure the surfaces in
//! this catalog. The catalog is deliberately not an enforcement gate: numeric
//! targets depend on deployment shape and remain host/release-profile policy.

/// Stable metric identifier for the operational SLO catalog.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationalSloMetricId(&'static str);

impl OperationalSloMetricId {
    /// Submit latency p50 metric id.
    pub const SUBMIT_LATENCY_P50: Self = Self("submit_latency_p50");
    /// Submit latency p95 metric id.
    pub const SUBMIT_LATENCY_P95: Self = Self("submit_latency_p95");
    /// Admission latency p50 metric id.
    pub const ADMISSION_LATENCY_P50: Self = Self("admission_latency_p50");
    /// Admission latency p95 metric id.
    pub const ADMISSION_LATENCY_P95: Self = Self("admission_latency_p95");
    /// Barrier append latency p50 metric id.
    pub const BARRIER_APPEND_LATENCY_P50: Self = Self("barrier_append_latency_p50");
    /// Barrier append latency p95 metric id.
    pub const BARRIER_APPEND_LATENCY_P95: Self = Self("barrier_append_latency_p95");
    /// Replay verify latency p50 metric id.
    pub const REPLAY_VERIFY_LATENCY_P50: Self = Self("replay_verify_latency_p50");
    /// Replay verify latency p95 metric id.
    pub const REPLAY_VERIFY_LATENCY_P95: Self = Self("replay_verify_latency_p95");
    /// Replay explain latency p50 metric id.
    pub const REPLAY_EXPLAIN_LATENCY_P50: Self = Self("replay_explain_latency_p50");
    /// Replay explain latency p95 metric id.
    pub const REPLAY_EXPLAIN_LATENCY_P95: Self = Self("replay_explain_latency_p95");
    /// Partition queue depth gauge metric id.
    pub const PARTITION_QUEUE_DEPTH: Self = Self("partition_queue_depth");
    /// Constraint snapshot stale-age gauge metric id.
    pub const CONSTRAINT_SNAPSHOT_STALE_AGE: Self = Self("constraint_snapshot_stale_age");

    /// Returns the stable token used in docs, tests and host telemetry mapping.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Runtime or replay surface measured by an operational SLO metric.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationalSloSurface {
    /// Host-facing submit call surface.
    Submit,
    /// Route/admission coordination surface.
    Admission,
    /// Execution-barrier audit append surface.
    BarrierAppend,
    /// Replay verification surface.
    ReplayVerify,
    /// Replay explanation rendering surface.
    ReplayExplain,
    /// Partition backlog surface.
    PartitionQueueDepth,
    /// Constraint snapshot freshness surface.
    ConstraintSnapshotStaleAge,
}

/// Measurement kind for an operational SLO metric.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationalSloMeasure {
    /// A duration distribution summarized by percentiles.
    Latency,
    /// A point-in-time queue depth gauge.
    QueueDepth,
    /// A point-in-time stale-age gauge.
    StaleAge,
}

/// Unit used by an operational SLO metric.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationalSloUnit {
    /// Duration measured in milliseconds.
    Milliseconds,
    /// Count measured as an integer item count.
    Count,
}

/// Percentile summarized by a latency metric.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationalSloPercentile {
    /// Median latency.
    P50,
    /// Tail latency readiness percentile.
    P95,
}

/// Source boundary that supplies or derives the operational signal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationalSloSignalSource {
    /// Runtime submit API timing.
    RuntimeSubmit,
    /// Runtime route/admission coordinator timing.
    RuntimeAdmission,
    /// Audit adapter barrier append timing.
    AuditBarrierAppend,
    /// Replay verification timing.
    ReplayVerify,
    /// Replay explanation timing.
    ReplayExplain,
    /// Runtime partition backlog observation.
    RuntimePartitionQueue,
    /// Host-owned constraint snapshot observation.
    ConstraintSnapshot,
}

/// Threshold ownership policy for a catalog metric.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationalSloThresholdPolicy {
    /// The repo defines the required measurement shape; host/release profiles own numeric targets.
    HostDefined,
}

/// One metric in the operational SLO measurement contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationalSloMetric {
    /// Stable metric id.
    pub id: OperationalSloMetricId,
    /// Runtime or replay surface.
    pub surface: OperationalSloSurface,
    /// Measurement kind.
    pub measure: OperationalSloMeasure,
    /// Measurement unit.
    pub unit: OperationalSloUnit,
    /// Percentile for latency metrics; absent for gauges.
    pub percentile: Option<OperationalSloPercentile>,
    /// Boundary that supplies or derives the signal.
    pub signal_source: OperationalSloSignalSource,
    /// Numeric-threshold ownership policy.
    pub threshold_policy: OperationalSloThresholdPolicy,
}

/// Field whose catalog shape differs from the operational SLO contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationalSloMetricField {
    /// Surface mismatch.
    Surface,
    /// Measurement kind mismatch.
    Measure,
    /// Unit mismatch.
    Unit,
    /// Percentile mismatch.
    Percentile,
    /// Signal source mismatch.
    SignalSource,
    /// Threshold policy mismatch.
    ThresholdPolicy,
}

/// Validation failure for an operational SLO catalog candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationalSloCatalogError {
    /// The same metric id appears more than once.
    DuplicateMetricId(OperationalSloMetricId),
    /// A required metric is absent.
    MissingMetric(OperationalSloMetricId),
    /// A metric id is not part of the catalog.
    UnexpectedMetric(OperationalSloMetricId),
    /// A metric exists but does not match the canonical shape.
    MetricShapeMismatch {
        /// Metric with the mismatched shape.
        id: OperationalSloMetricId,
        /// Mismatched field.
        field: OperationalSloMetricField,
    },
}

/// Authoritative operational SLO measurement catalog.
pub const OPERATIONAL_SLO_METRICS: &[OperationalSloMetric] = &[
    latency_metric(
        OperationalSloMetricId::SUBMIT_LATENCY_P50,
        OperationalSloSurface::Submit,
        OperationalSloPercentile::P50,
        OperationalSloSignalSource::RuntimeSubmit,
    ),
    latency_metric(
        OperationalSloMetricId::SUBMIT_LATENCY_P95,
        OperationalSloSurface::Submit,
        OperationalSloPercentile::P95,
        OperationalSloSignalSource::RuntimeSubmit,
    ),
    latency_metric(
        OperationalSloMetricId::ADMISSION_LATENCY_P50,
        OperationalSloSurface::Admission,
        OperationalSloPercentile::P50,
        OperationalSloSignalSource::RuntimeAdmission,
    ),
    latency_metric(
        OperationalSloMetricId::ADMISSION_LATENCY_P95,
        OperationalSloSurface::Admission,
        OperationalSloPercentile::P95,
        OperationalSloSignalSource::RuntimeAdmission,
    ),
    latency_metric(
        OperationalSloMetricId::BARRIER_APPEND_LATENCY_P50,
        OperationalSloSurface::BarrierAppend,
        OperationalSloPercentile::P50,
        OperationalSloSignalSource::AuditBarrierAppend,
    ),
    latency_metric(
        OperationalSloMetricId::BARRIER_APPEND_LATENCY_P95,
        OperationalSloSurface::BarrierAppend,
        OperationalSloPercentile::P95,
        OperationalSloSignalSource::AuditBarrierAppend,
    ),
    latency_metric(
        OperationalSloMetricId::REPLAY_VERIFY_LATENCY_P50,
        OperationalSloSurface::ReplayVerify,
        OperationalSloPercentile::P50,
        OperationalSloSignalSource::ReplayVerify,
    ),
    latency_metric(
        OperationalSloMetricId::REPLAY_VERIFY_LATENCY_P95,
        OperationalSloSurface::ReplayVerify,
        OperationalSloPercentile::P95,
        OperationalSloSignalSource::ReplayVerify,
    ),
    latency_metric(
        OperationalSloMetricId::REPLAY_EXPLAIN_LATENCY_P50,
        OperationalSloSurface::ReplayExplain,
        OperationalSloPercentile::P50,
        OperationalSloSignalSource::ReplayExplain,
    ),
    latency_metric(
        OperationalSloMetricId::REPLAY_EXPLAIN_LATENCY_P95,
        OperationalSloSurface::ReplayExplain,
        OperationalSloPercentile::P95,
        OperationalSloSignalSource::ReplayExplain,
    ),
    gauge_metric(
        OperationalSloMetricId::PARTITION_QUEUE_DEPTH,
        OperationalSloSurface::PartitionQueueDepth,
        OperationalSloMeasure::QueueDepth,
        OperationalSloUnit::Count,
        OperationalSloSignalSource::RuntimePartitionQueue,
    ),
    gauge_metric(
        OperationalSloMetricId::CONSTRAINT_SNAPSHOT_STALE_AGE,
        OperationalSloSurface::ConstraintSnapshotStaleAge,
        OperationalSloMeasure::StaleAge,
        OperationalSloUnit::Milliseconds,
        OperationalSloSignalSource::ConstraintSnapshot,
    ),
];

const fn latency_metric(
    id: OperationalSloMetricId,
    surface: OperationalSloSurface,
    percentile: OperationalSloPercentile,
    signal_source: OperationalSloSignalSource,
) -> OperationalSloMetric {
    OperationalSloMetric {
        id,
        surface,
        measure: OperationalSloMeasure::Latency,
        unit: OperationalSloUnit::Milliseconds,
        percentile: Some(percentile),
        signal_source,
        threshold_policy: OperationalSloThresholdPolicy::HostDefined,
    }
}

const fn gauge_metric(
    id: OperationalSloMetricId,
    surface: OperationalSloSurface,
    measure: OperationalSloMeasure,
    unit: OperationalSloUnit,
    signal_source: OperationalSloSignalSource,
) -> OperationalSloMetric {
    OperationalSloMetric {
        id,
        surface,
        measure,
        unit,
        percentile: None,
        signal_source,
        threshold_policy: OperationalSloThresholdPolicy::HostDefined,
    }
}

/// Returns the canonical metric shape for a stable metric id.
#[must_use]
pub fn operational_slo_metric(id: OperationalSloMetricId) -> Option<&'static OperationalSloMetric> {
    OPERATIONAL_SLO_METRICS
        .iter()
        .find(|metric| metric.id == id)
}

/// Validates a candidate catalog against the authoritative metric shapes.
///
/// This check is intentionally structural. It does not assert measured values or
/// numeric thresholds.
pub fn validate_operational_slo_catalog(
    catalog: &[OperationalSloMetric],
) -> Result<(), OperationalSloCatalogError> {
    for (index, metric) in catalog.iter().enumerate() {
        if catalog
            .iter()
            .skip(index + 1)
            .any(|candidate| candidate.id == metric.id)
        {
            return Err(OperationalSloCatalogError::DuplicateMetricId(metric.id));
        }

        let Some(expected) = operational_slo_metric(metric.id) else {
            return Err(OperationalSloCatalogError::UnexpectedMetric(metric.id));
        };
        validate_metric_shape(metric, expected)?;
    }

    for expected in OPERATIONAL_SLO_METRICS {
        if catalog.iter().all(|metric| metric.id != expected.id) {
            return Err(OperationalSloCatalogError::MissingMetric(expected.id));
        }
    }

    Ok(())
}

fn validate_metric_shape(
    metric: &OperationalSloMetric,
    expected: &OperationalSloMetric,
) -> Result<(), OperationalSloCatalogError> {
    let id = metric.id;
    if metric.surface != expected.surface {
        return Err(shape_mismatch(id, OperationalSloMetricField::Surface));
    }
    if metric.measure != expected.measure {
        return Err(shape_mismatch(id, OperationalSloMetricField::Measure));
    }
    if metric.unit != expected.unit {
        return Err(shape_mismatch(id, OperationalSloMetricField::Unit));
    }
    if metric.percentile != expected.percentile {
        return Err(shape_mismatch(id, OperationalSloMetricField::Percentile));
    }
    if metric.signal_source != expected.signal_source {
        return Err(shape_mismatch(id, OperationalSloMetricField::SignalSource));
    }
    if metric.threshold_policy != expected.threshold_policy {
        return Err(shape_mismatch(
            id,
            OperationalSloMetricField::ThresholdPolicy,
        ));
    }
    Ok(())
}

const fn shape_mismatch(
    id: OperationalSloMetricId,
    field: OperationalSloMetricField,
) -> OperationalSloCatalogError {
    OperationalSloCatalogError::MetricShapeMismatch { id, field }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_validates() {
        assert_eq!(
            validate_operational_slo_catalog(OPERATIONAL_SLO_METRICS),
            Ok(())
        );
    }

    #[test]
    fn metric_tokens_are_stable() {
        let expected = [
            (
                OperationalSloMetricId::SUBMIT_LATENCY_P50,
                "submit_latency_p50",
            ),
            (
                OperationalSloMetricId::SUBMIT_LATENCY_P95,
                "submit_latency_p95",
            ),
            (
                OperationalSloMetricId::ADMISSION_LATENCY_P50,
                "admission_latency_p50",
            ),
            (
                OperationalSloMetricId::ADMISSION_LATENCY_P95,
                "admission_latency_p95",
            ),
            (
                OperationalSloMetricId::BARRIER_APPEND_LATENCY_P50,
                "barrier_append_latency_p50",
            ),
            (
                OperationalSloMetricId::BARRIER_APPEND_LATENCY_P95,
                "barrier_append_latency_p95",
            ),
            (
                OperationalSloMetricId::REPLAY_VERIFY_LATENCY_P50,
                "replay_verify_latency_p50",
            ),
            (
                OperationalSloMetricId::REPLAY_VERIFY_LATENCY_P95,
                "replay_verify_latency_p95",
            ),
            (
                OperationalSloMetricId::REPLAY_EXPLAIN_LATENCY_P50,
                "replay_explain_latency_p50",
            ),
            (
                OperationalSloMetricId::REPLAY_EXPLAIN_LATENCY_P95,
                "replay_explain_latency_p95",
            ),
            (
                OperationalSloMetricId::PARTITION_QUEUE_DEPTH,
                "partition_queue_depth",
            ),
            (
                OperationalSloMetricId::CONSTRAINT_SNAPSHOT_STALE_AGE,
                "constraint_snapshot_stale_age",
            ),
        ];

        assert_eq!(OPERATIONAL_SLO_METRICS.len(), expected.len());
        for (id, token) in expected {
            assert_eq!(id.as_str(), token);
            assert!(operational_slo_metric(id).is_some());
        }
    }

    #[test]
    fn latency_surfaces_have_p50_and_p95() {
        for surface in [
            OperationalSloSurface::Submit,
            OperationalSloSurface::Admission,
            OperationalSloSurface::BarrierAppend,
            OperationalSloSurface::ReplayVerify,
            OperationalSloSurface::ReplayExplain,
        ] {
            let percentiles: Vec<_> = OPERATIONAL_SLO_METRICS
                .iter()
                .filter(|metric| metric.surface == surface)
                .map(|metric| metric.percentile)
                .collect();
            assert_eq!(
                percentiles,
                vec![
                    Some(OperationalSloPercentile::P50),
                    Some(OperationalSloPercentile::P95)
                ]
            );
        }
    }

    #[test]
    fn gauge_surfaces_have_no_percentile() {
        for id in [
            OperationalSloMetricId::PARTITION_QUEUE_DEPTH,
            OperationalSloMetricId::CONSTRAINT_SNAPSHOT_STALE_AGE,
        ] {
            assert_eq!(
                operational_slo_metric(id).map(|metric| metric.percentile),
                Some(None)
            );
        }
    }

    #[test]
    fn duplicate_metric_id_is_rejected() {
        let mut catalog = OPERATIONAL_SLO_METRICS.to_vec();
        if let Some(metric) = OPERATIONAL_SLO_METRICS.first() {
            catalog.push(*metric);
        }

        assert_eq!(
            validate_operational_slo_catalog(&catalog),
            Err(OperationalSloCatalogError::DuplicateMetricId(
                OperationalSloMetricId::SUBMIT_LATENCY_P50
            ))
        );
    }

    #[test]
    fn missing_metric_is_rejected() {
        let catalog: Vec<_> = OPERATIONAL_SLO_METRICS
            .iter()
            .copied()
            .filter(|metric| metric.id != OperationalSloMetricId::SUBMIT_LATENCY_P50)
            .collect();

        assert_eq!(
            validate_operational_slo_catalog(&catalog),
            Err(OperationalSloCatalogError::MissingMetric(
                OperationalSloMetricId::SUBMIT_LATENCY_P50
            ))
        );
    }

    #[test]
    fn latency_percentile_shape_is_enforced() {
        let mut catalog = OPERATIONAL_SLO_METRICS.to_vec();
        for metric in &mut catalog {
            if metric.id == OperationalSloMetricId::SUBMIT_LATENCY_P50 {
                metric.percentile = None;
            }
        }

        assert_eq!(
            validate_operational_slo_catalog(&catalog),
            Err(shape_mismatch(
                OperationalSloMetricId::SUBMIT_LATENCY_P50,
                OperationalSloMetricField::Percentile
            ))
        );
    }

    #[test]
    fn queue_depth_gauge_shape_is_enforced() {
        let mut catalog = OPERATIONAL_SLO_METRICS.to_vec();
        let mut updated = false;
        for metric in &mut catalog {
            if metric.id == OperationalSloMetricId::PARTITION_QUEUE_DEPTH {
                metric.percentile = Some(OperationalSloPercentile::P50);
                updated = true;
            }
        }
        assert!(updated);

        assert_eq!(
            validate_operational_slo_catalog(&catalog),
            Err(shape_mismatch(
                OperationalSloMetricId::PARTITION_QUEUE_DEPTH,
                OperationalSloMetricField::Percentile
            ))
        );
    }

    #[test]
    fn stale_snapshot_age_unit_shape_is_enforced() {
        let mut catalog = OPERATIONAL_SLO_METRICS.to_vec();
        let mut updated = false;
        for metric in &mut catalog {
            if metric.id == OperationalSloMetricId::CONSTRAINT_SNAPSHOT_STALE_AGE {
                metric.unit = OperationalSloUnit::Count;
                updated = true;
            }
        }
        assert!(updated);

        assert_eq!(
            validate_operational_slo_catalog(&catalog),
            Err(shape_mismatch(
                OperationalSloMetricId::CONSTRAINT_SNAPSHOT_STALE_AGE,
                OperationalSloMetricField::Unit
            ))
        );
    }
}
