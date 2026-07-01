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

/// Builder for [`HostDispatchContext`] values accepted by the host API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDispatchContextBuilder {
    actor_ref: String,
    trace_id: String,
    correlation_id: String,
    request_id: Option<String>,
    config_snapshot_ref: String,
    idempotency_key: Option<String>,
    runtime_profile: HostRuntimeProfile,
    created_at: Timestamp,
}

impl HostDispatchContextBuilder {
    /// Start a host dispatch context with all required references.
    #[must_use]
    pub fn new(
        actor_ref: impl Into<String>,
        trace_id: impl Into<String>,
        correlation_id: impl Into<String>,
        config_snapshot_ref: impl Into<String>,
        created_at: Timestamp,
    ) -> Self {
        Self {
            actor_ref: actor_ref.into(),
            trace_id: trace_id.into(),
            correlation_id: correlation_id.into(),
            request_id: None,
            config_snapshot_ref: config_snapshot_ref.into(),
            idempotency_key: None,
            runtime_profile: HostRuntimeProfile::LinearOnly,
            created_at,
        }
    }

    /// Attach a host request id.
    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Attach a host idempotency key.
    #[must_use]
    pub fn with_idempotency_key(mut self, idempotency_key: impl Into<String>) -> Self {
        self.idempotency_key = Some(idempotency_key.into());
        self
    }

    /// Select the runtime profile advertised for this dispatch path.
    #[must_use]
    pub fn with_runtime_profile(mut self, runtime_profile: HostRuntimeProfile) -> Self {
        self.runtime_profile = runtime_profile;
        self
    }

    /// Build and validate the context.
    ///
    /// # Errors
    /// Returns [`HostDispatchError`] when any required host reference is empty.
    #[must_use = "host dispatch context builder result must be handled"]
    pub fn build(self) -> Result<HostDispatchContext, HostDispatchError> {
        let context = HostDispatchContext {
            actor_ref: self.actor_ref,
            trace_id: self.trace_id,
            correlation_id: self.correlation_id,
            request_id: self.request_id,
            config_snapshot_ref: self.config_snapshot_ref,
            idempotency_key: self.idempotency_key,
            runtime_profile: self.runtime_profile,
            created_at: self.created_at,
        };
        validate_host_context(&context)?;
        Ok(context)
    }
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

/// Builder for [`HostTaskSpec`] values accepted by the host API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostTaskSpecBuilder {
    task_id: String,
    action_id: ActionId,
    predicate_id: PredicateId,
    subject_ref: String,
    plan_hash: Option<PlanHash>,
    effect_class: HostEffectClass,
    payload_ref: Option<String>,
    dependencies: Vec<String>,
    idempotency_key: Option<String>,
    partition_route: PartitionRoute,
}

impl HostTaskSpecBuilder {
    /// Start a host task with all required semantic fields.
    #[must_use]
    pub fn new(
        task_id: impl Into<String>,
        action_id: ActionId,
        predicate_id: PredicateId,
        subject_ref: impl Into<String>,
        effect_class: HostEffectClass,
        partition_route: PartitionRoute,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            action_id,
            predicate_id,
            subject_ref: subject_ref.into(),
            plan_hash: None,
            effect_class,
            payload_ref: None,
            dependencies: Vec::new(),
            idempotency_key: None,
            partition_route,
        }
    }

    /// Attach a precomputed plan hash.
    #[must_use]
    pub fn with_plan_hash(mut self, plan_hash: PlanHash) -> Self {
        self.plan_hash = Some(plan_hash);
        self
    }

    /// Attach a host-owned payload or object reference.
    #[must_use]
    pub fn with_payload_ref(mut self, payload_ref: impl Into<String>) -> Self {
        self.payload_ref = Some(payload_ref.into());
        self
    }

    /// Replace the dependency task id list.
    #[must_use]
    pub fn with_dependencies<I, S>(mut self, dependencies: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.dependencies = dependencies.into_iter().map(Into::into).collect();
        self
    }

    /// Attach a host task idempotency key.
    #[must_use]
    pub fn with_idempotency_key(mut self, idempotency_key: impl Into<String>) -> Self {
        self.idempotency_key = Some(idempotency_key.into());
        self
    }

    /// Build and validate the task.
    ///
    /// # Errors
    /// Returns [`HostDispatchError`] when the task does not satisfy the current
    /// host API contract.
    #[must_use = "host task spec builder result must be handled"]
    pub fn build(self) -> Result<HostTaskSpec, HostDispatchError> {
        let task = HostTaskSpec {
            task_id: self.task_id,
            action_id: self.action_id,
            predicate_id: self.predicate_id,
            subject_ref: self.subject_ref,
            plan_hash: self.plan_hash,
            effect_class: self.effect_class,
            payload_ref: self.payload_ref,
            dependencies: self.dependencies,
            idempotency_key: self.idempotency_key,
            partition_route: self.partition_route,
            host_api_version: CAUSLANE_HOST_API_VERSION.to_owned(),
        };
        validate_host_task(&task)?;
        Ok(task)
    }
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
    /// The supplied dispatch context is malformed.
    InvalidContext {
        /// Invalid context field name.
        field: &'static str,
        /// Human-readable reason.
        reason: String,
    },
    /// The supplied host task is malformed.
    InvalidTask {
        /// Rejected task id, when one was supplied.
        task_id: Option<String>,
        /// Invalid task field name.
        field: &'static str,
        /// Human-readable reason.
        reason: String,
    },
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

/// Validate a host dispatch context against the versioned host API contract.
///
/// # Errors
/// Returns [`HostDispatchError`] if the context is missing required host refs.
#[must_use = "host context validation result must be handled"]
pub fn validate_host_context(ctx: &HostDispatchContext) -> Result<(), HostDispatchError> {
    validate_context_ref("actor_ref", &ctx.actor_ref)?;
    validate_context_ref("trace_id", &ctx.trace_id)?;
    validate_context_ref("correlation_id", &ctx.correlation_id)?;
    validate_context_ref("config_snapshot_ref", &ctx.config_snapshot_ref)?;
    validate_optional_context_ref("request_id", ctx.request_id.as_ref())?;
    validate_optional_context_ref("idempotency_key", ctx.idempotency_key.as_ref())?;
    Ok(())
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
    validate_task_ref(task, "action_id", &task.action_id.0)?;
    validate_task_ref(task, "predicate_id", &task.predicate_id.0)?;
    validate_task_ref(task, "subject_ref", &task.subject_ref)?;
    validate_optional_task_ref(task, "payload_ref", task.payload_ref.as_ref())?;
    validate_optional_task_ref(task, "idempotency_key", task.idempotency_key.as_ref())?;
    validate_task_dependencies(task)?;
    if task.effect_class == HostEffectClass::Forbidden {
        return Err(HostDispatchError::ForbiddenEffect {
            task_id: task.task_id.clone(),
        });
    }
    if task.effect_class == HostEffectClass::HardEffect && task.idempotency_key.is_none() {
        return Err(invalid_task(
            task,
            "idempotency_key",
            "hard effects require a host task idempotency key",
        ));
    }
    if let Some(reason) = task.partition_route.invalid_reason() {
        return Err(HostDispatchError::InvalidPartitionRoute {
            task_id: task.task_id.clone(),
            reason,
        });
    }
    Ok(())
}

/// Validate a context and task as one host dispatch submission.
///
/// # Errors
/// Returns [`HostDispatchError`] if either the context or task is malformed.
#[must_use = "host submission validation result must be handled"]
pub fn validate_host_submission(
    ctx: &HostDispatchContext,
    task: &HostTaskSpec,
) -> Result<(), HostDispatchError> {
    validate_host_context(ctx)?;
    validate_host_task(task)
}

fn validate_context_ref(field: &'static str, value: &str) -> Result<(), HostDispatchError> {
    if value.is_empty() {
        return Err(invalid_context(field, "must be non-empty"));
    }
    Ok(())
}

fn validate_optional_context_ref(
    field: &'static str,
    value: Option<&String>,
) -> Result<(), HostDispatchError> {
    if value.is_some_and(String::is_empty) {
        return Err(invalid_context(field, "must be non-empty when supplied"));
    }
    Ok(())
}

fn validate_task_ref(
    task: &HostTaskSpec,
    field: &'static str,
    value: &str,
) -> Result<(), HostDispatchError> {
    if value.is_empty() {
        return Err(invalid_task(task, field, "must be non-empty"));
    }
    Ok(())
}

fn validate_optional_task_ref(
    task: &HostTaskSpec,
    field: &'static str,
    value: Option<&String>,
) -> Result<(), HostDispatchError> {
    if value.is_some_and(String::is_empty) {
        return Err(invalid_task(task, field, "must be non-empty when supplied"));
    }
    Ok(())
}

fn validate_task_dependencies(task: &HostTaskSpec) -> Result<(), HostDispatchError> {
    let mut seen = BTreeSet::new();
    for dependency in &task.dependencies {
        if dependency.is_empty() {
            return Err(invalid_task(
                task,
                "dependencies",
                "dependency task id must be non-empty",
            ));
        }
        if dependency == &task.task_id {
            return Err(invalid_task(
                task,
                "dependencies",
                "task cannot depend on itself",
            ));
        }
        if !seen.insert(dependency.as_str()) {
            return Err(invalid_task(
                task,
                "dependencies",
                &format!("duplicate dependency task id {dependency}"),
            ));
        }
    }
    Ok(())
}

fn invalid_context(field: &'static str, reason: &str) -> HostDispatchError {
    HostDispatchError::InvalidContext {
        field,
        reason: reason.to_owned(),
    }
}

fn invalid_task(task: &HostTaskSpec, field: &'static str, reason: &str) -> HostDispatchError {
    HostDispatchError::InvalidTask {
        task_id: (!task.task_id.is_empty()).then(|| task.task_id.clone()),
        field,
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
mod tests;
