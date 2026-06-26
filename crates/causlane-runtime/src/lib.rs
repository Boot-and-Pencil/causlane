//! Runtime composition shell.
//!
//! This crate is where async runtimes, queues, persistence and external integrations may eventually appear.

#![forbid(unsafe_code)]
#![deny(warnings)]

pub mod adapters;
pub mod authz;
pub mod guarded_executor;
#[cfg(feature = "tokio-runtime")]
pub mod in_process;
pub mod linear_host;
pub mod operational_slo;
pub mod partitions;
pub mod projection_guard;
#[cfg(feature = "tokio-runtime")]
pub mod shadow;
#[cfg(all(test, any(feature = "apalis", feature = "restate")))]
mod test_support;

#[cfg(feature = "tokio-runtime")]
pub use in_process::{
    InProcessBackpressureMode, InProcessBackpressurePolicy, InProcessEffectFuture,
    InProcessEffectHandler, InProcessRuntime, InProcessRuntimeConfig, InProcessRuntimeError,
    InProcessRuntimeEvent,
};
pub use linear_host::LinearHostDispatcher;
pub use operational_slo::{
    operational_slo_metric, validate_operational_slo_catalog, OperationalSloCatalogError,
    OperationalSloMeasure, OperationalSloMetric, OperationalSloMetricField, OperationalSloMetricId,
    OperationalSloPercentile, OperationalSloSignalSource, OperationalSloSurface,
    OperationalSloThresholdPolicy, OperationalSloUnit, OPERATIONAL_SLO_METRICS,
};
#[cfg(feature = "tokio-runtime")]
pub use shadow::{
    compare_shadow_events, ShadowComparison, ShadowExpectation, ShadowExpectationKind,
    ShadowMismatch, ShadowObservation, ShadowStatus,
};
