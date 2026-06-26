//! Stable host-facing integration API.
//!
//! This module is intentionally smaller than the internal semantic kernel.  It
//! lets host projects such as Refinery pin a versioned dispatch seam while the
//! dispatcher remains free to evolve its lifecycle, barrier, lease, witness,
//! formal-model, and replay internals.

use std::collections::BTreeSet;

use crate::domain::{ActionId, PlanHash, PredicateId, Timestamp};

/// Stable version string for the host-facing dispatcher API.
pub const CAUSLANE_HOST_API_VERSION: &str = "causlane.host-dispatch.v2";

/// Key identifying a single-owner partition (e.g. tenant, conflict domain, or
/// root subject).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PartitionKey(pub String);

/// Partition route for one host task.
///
/// `primary` is the owning partition that receives the task. `participants`
/// names additional partitions whose admission order must be coordinated before
/// the task enters the primary queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartitionRoute {
    /// Owning partition.
    pub primary: PartitionKey,
    /// Additional partitions touched by this task.
    pub participants: Vec<PartitionKey>,
}

impl PartitionRoute {
    /// Route that touches only one owning partition.
    #[must_use]
    pub fn for_primary(primary: PartitionKey) -> Self {
        Self {
            primary,
            participants: Vec::new(),
        }
    }

    /// Route that touches one owning partition plus additional participants.
    #[must_use]
    pub fn new(primary: PartitionKey, participants: Vec<PartitionKey>) -> Self {
        Self {
            primary,
            participants,
        }
    }

    /// Deterministic lock/acquisition order for a route.
    ///
    /// The order is the deduped set of `primary + participants`, sorted by
    /// [`PartitionKey`]. Runtime code must use this instead of reimplementing
    /// cross-partition ordering.
    #[must_use]
    pub fn acquisition_order(&self) -> Vec<PartitionKey> {
        let mut ordered = BTreeSet::from([self.primary.clone()]);
        ordered.extend(self.participants.iter().cloned());
        ordered.into_iter().collect()
    }

    fn invalid_reason(&self) -> Option<String> {
        if self.primary.0.is_empty() {
            return Some("primary partition key is empty".to_owned());
        }
        self.participants
            .iter()
            .find(|participant| participant.0.is_empty())
            .map(|_participant| "participant partition key is empty".to_owned())
    }
}

/// Host-visible effect class for admission at the integration seam.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostEffectClass {
    /// The task reads or projects already committed truth only.
    ReadOnly,
    /// The task performs a non-destructive operational write.
    SoftWrite,
    /// The task may perform a hard effect after the host has authorized it.
    HardEffect,
    /// The task is forbidden at the host boundary and must be rejected.
    Forbidden,
}

/// Host runtime profile advertised to the dispatcher integration layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostRuntimeProfile {
    /// Deterministic linear execution only.
    LinearOnly,
    /// Parallel-capable host profile exists, but parallelism is disabled here.
    ParallelCapableButDisabled,
}

/// Context supplied by the host for each dispatch interaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDispatchContext {
    /// Actor or service principal reference known to the host.
    pub actor_ref: String,
    /// Trace id propagated by the host.
    pub trace_id: String,
    /// Correlation id propagated by the host.
    pub correlation_id: String,
    /// Optional request id propagated by HTTP/CLI/worker surfaces.
    pub request_id: Option<String>,
    /// Immutable host config snapshot reference.
    pub config_snapshot_ref: String,
    /// Optional host idempotency key.
    pub idempotency_key: Option<String>,
    /// Host runtime profile for this dispatch path.
    pub runtime_profile: HostRuntimeProfile,
    /// Host-observed creation timestamp.
    pub created_at: Timestamp,
}

/// Host task specification accepted by the stable dispatch seam.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostTaskSpec {
    /// Host task id.
    pub task_id: String,
    /// Causlane action id for semantic/audit mapping.
    pub action_id: ActionId,
    /// Causlane predicate id for semantic/audit mapping.
    pub predicate_id: PredicateId,
    /// Host subject reference.
    pub subject_ref: String,
    /// Optional precomputed plan hash if the host has one.
    pub plan_hash: Option<PlanHash>,
    /// Host-visible effect class.
    pub effect_class: HostEffectClass,
    /// Optional payload/object reference; raw secret values are forbidden.
    pub payload_ref: Option<String>,
    /// Dependency task ids that must complete before this task is ready.
    pub dependencies: Vec<String>,
    /// Optional idempotency key for duplicate suppression.
    pub idempotency_key: Option<String>,
    /// Partition route used by partitioned dispatchers.
    pub partition_route: PartitionRoute,
    /// Expected host API version; must equal [`CAUSLANE_HOST_API_VERSION`].
    pub host_api_version: String,
}

/// Capabilities advertised by a host dispatcher implementation.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDispatcherCapabilities {
    /// API version implemented by this dispatcher.
    pub api_version: String,
    /// Whether linear drain is supported.
    pub supports_linear_drain: bool,
    /// Whether parallelism is enabled for this implementation.
    pub supports_parallelism: bool,
    /// Whether dependency graph readiness is supported.
    pub supports_dependency_graph: bool,
    /// Whether this dispatcher coordinates multi-partition admission.
    pub supports_partition_coordination: bool,
    /// Whether the host must authorize before submit/drain.
    pub requires_external_authz: bool,
    /// Whether the host must provide idempotency before hard effects.
    pub requires_external_idempotency: bool,
}

impl HostDispatcherCapabilities {
    /// Capabilities of the deterministic reference implementation.
    #[must_use]
    pub fn linear_reference() -> Self {
        Self {
            api_version: CAUSLANE_HOST_API_VERSION.to_owned(),
            supports_linear_drain: true,
            supports_parallelism: false,
            supports_dependency_graph: true,
            supports_partition_coordination: false,
            requires_external_authz: true,
            requires_external_idempotency: true,
        }
    }
}

/// Ticket returned when the dispatcher accepts a host task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDispatchTicket {
    /// Ticket id produced by the dispatcher.
    pub ticket_id: String,
    /// Task id associated with the ticket.
    pub task_id: String,
    /// API version that accepted the task.
    pub api_version: String,
}

/// Result of one linear drain attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostDrainOutcome {
    /// A task executed and produced host-visible references.
    Executed {
        /// Executed task id.
        task_id: String,
        /// Produced fact/object references.
        produced_refs: Vec<String>,
    },
    /// No task was queued.
    Idle,
    /// Tasks exist, but all are waiting on dependencies.
    Blocked,
}

/// Outcome produced by a host effect handler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostEffectOutcome {
    /// Produced fact/object references.
    pub produced_refs: Vec<String>,
}

/// Errors returned by the host-facing dispatch API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostDispatchError {
    /// The supplied task did not name the required API version.
    UnsupportedApiVersion {
        /// Required API version.
        expected: String,
        /// Supplied API version.
        got: String,
    },
    /// The supplied effect class is forbidden.
    ForbiddenEffect {
        /// Rejected task id.
        task_id: String,
    },
    /// The supplied task id was empty.
    MissingTaskId,
    /// The task was a duplicate according to host idempotency.
    DuplicateSuppressed {
        /// Suppressed task id.
        task_id: String,
    },
    /// The supplied partition route is malformed.
    InvalidPartitionRoute {
        /// Rejected task id.
        task_id: String,
        /// Human-readable reason.
        reason: String,
    },
    /// The handler rejected or failed a task.
    HandlerRejected {
        /// Human-readable reason.
        reason: String,
    },
    /// No queued task can currently run.
    Blocked,
}

/// Host callback used by the reference dispatcher to spend one ready task.
pub trait HostEffectHandler {
    /// Execute the host effect for `task` under `ctx`.
    ///
    /// # Errors
    /// Returns [`HostDispatchError`] when the host refuses or fails the effect.
    fn execute_host_effect(
        &mut self,
        ctx: &HostDispatchContext,
        task: &HostTaskSpec,
    ) -> Result<HostEffectOutcome, HostDispatchError>;
}

/// Stable host-facing dispatcher port.
pub trait HostDispatchPort {
    /// Return the capabilities of this dispatcher implementation.
    fn capabilities(&self) -> HostDispatcherCapabilities;

    /// Submit a host task.
    ///
    /// # Errors
    /// Returns [`HostDispatchError`] if admission fails.
    fn submit(
        &mut self,
        ctx: &HostDispatchContext,
        task: HostTaskSpec,
    ) -> Result<HostDispatchTicket, HostDispatchError>;

    /// Drain at most one ready task through `handler`.
    ///
    /// # Errors
    /// Returns [`HostDispatchError`] if handler execution fails.
    fn drain_once<H: HostEffectHandler>(
        &mut self,
        ctx: &HostDispatchContext,
        handler: &mut H,
    ) -> Result<HostDrainOutcome, HostDispatchError>;
}

/// Validate a host task against the versioned host API contract.
///
/// # Errors
/// Returns [`HostDispatchError`] if the task does not satisfy the stable seam.
#[must_use = "host task validation result must be handled"]
pub fn validate_host_task(task: &HostTaskSpec) -> Result<(), HostDispatchError> {
    if task.task_id.is_empty() {
        return Err(HostDispatchError::MissingTaskId);
    }
    if task.host_api_version != CAUSLANE_HOST_API_VERSION {
        return Err(HostDispatchError::UnsupportedApiVersion {
            expected: CAUSLANE_HOST_API_VERSION.to_owned(),
            got: task.host_api_version.clone(),
        });
    }
    if task.effect_class == HostEffectClass::Forbidden {
        return Err(HostDispatchError::ForbiddenEffect {
            task_id: task.task_id.clone(),
        });
    }
    if let Some(reason) = task.partition_route.invalid_reason() {
        return Err(HostDispatchError::InvalidPartitionRoute {
            task_id: task.task_id.clone(),
            reason,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(effect_class: HostEffectClass) -> HostTaskSpec {
        HostTaskSpec {
            task_id: "task-1".to_owned(),
            action_id: ActionId("foundation.task.enqueue".to_owned()),
            predicate_id: PredicateId("foundation.task.enqueue".to_owned()),
            subject_ref: "host://subject/demo".to_owned(),
            plan_hash: None,
            effect_class,
            payload_ref: Some("object://payload/demo".to_owned()),
            dependencies: Vec::new(),
            idempotency_key: Some("idem-1".to_owned()),
            partition_route: PartitionRoute::for_primary(PartitionKey("partition-1".to_owned())),
            host_api_version: CAUSLANE_HOST_API_VERSION.to_owned(),
        }
    }

    #[test]
    fn host_api_rejects_wrong_version() {
        let mut bad = task(HostEffectClass::SoftWrite);
        bad.host_api_version = "causlane.host-dispatch.v0".to_owned();
        assert!(matches!(
            validate_host_task(&bad),
            Err(HostDispatchError::UnsupportedApiVersion { .. })
        ));
    }

    #[test]
    fn host_api_rejects_forbidden_effect() {
        assert_eq!(
            validate_host_task(&task(HostEffectClass::Forbidden)),
            Err(HostDispatchError::ForbiddenEffect {
                task_id: "task-1".to_owned(),
            })
        );
    }

    #[test]
    fn linear_reference_capability_keeps_parallelism_disabled() {
        let capabilities = HostDispatcherCapabilities::linear_reference();
        assert_eq!(capabilities.api_version, CAUSLANE_HOST_API_VERSION);
        assert!(!capabilities.supports_parallelism);
        assert!(!capabilities.supports_partition_coordination);
        assert!(capabilities.requires_external_authz);
    }

    #[test]
    fn partition_route_acquisition_order_is_sorted_and_deduped() {
        let route = PartitionRoute::new(
            PartitionKey("tenant:b".to_owned()),
            vec![
                PartitionKey("tenant:c".to_owned()),
                PartitionKey("tenant:a".to_owned()),
                PartitionKey("tenant:b".to_owned()),
            ],
        );

        assert_eq!(
            route.acquisition_order(),
            vec![
                PartitionKey("tenant:a".to_owned()),
                PartitionKey("tenant:b".to_owned()),
                PartitionKey("tenant:c".to_owned()),
            ]
        );
    }

    #[test]
    fn partition_route_rejects_empty_primary_and_participant() {
        let mut bad_primary = task(HostEffectClass::SoftWrite);
        bad_primary.partition_route = PartitionRoute::for_primary(PartitionKey(String::new()));
        assert_eq!(
            validate_host_task(&bad_primary),
            Err(HostDispatchError::InvalidPartitionRoute {
                task_id: "task-1".to_owned(),
                reason: "primary partition key is empty".to_owned(),
            })
        );

        let mut bad_participant = task(HostEffectClass::SoftWrite);
        bad_participant.partition_route = PartitionRoute::new(
            PartitionKey("partition-1".to_owned()),
            vec![PartitionKey(String::new())],
        );
        assert_eq!(
            validate_host_task(&bad_participant),
            Err(HostDispatchError::InvalidPartitionRoute {
                task_id: "task-1".to_owned(),
                reason: "participant partition key is empty".to_owned(),
            })
        );
    }
}
