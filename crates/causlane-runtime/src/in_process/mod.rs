//! Feature-gated Tokio in-process runtime.
//!
//! This runtime is an ephemeral adapter: it owns partition-local queues and
//! spends work only through a host-supplied effect handler. It does not create
//! policy decisions, audit truth, durable leases, or replay authority.

use std::{
    collections::{HashMap, HashSet},
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use causlane_core::{
    validate_host_submission, HostDispatchContext, HostDispatchError, HostDispatchTicket,
    HostDispatcherCapabilities, HostEffectOutcome, HostTaskSpec, CAUSLANE_HOST_API_VERSION,
};
use tokio::{
    sync::{broadcast, mpsc, oneshot, Semaphore},
    task::JoinHandle,
};

use crate::partitions::PartitionKey;

mod coordinator;
#[cfg(test)]
mod tests;
mod worker;

use coordinator::{AdmissionCoordinator, AdmissionGuards};
use worker::{partition_loop, PartitionWorkerSpawner};

/// Boxed future returned by an in-process host effect handler.
pub type InProcessEffectFuture =
    Pin<Box<dyn Future<Output = Result<HostEffectOutcome, HostDispatchError>> + Send + 'static>>;

/// Async host callback used by [`InProcessRuntime`] workers.
///
/// The handler remains the only place where host effects happen. The runtime
/// validates admission and schedules work, but it does not execute effects
/// directly or mint semantic authority.
pub trait InProcessEffectHandler: Send + Sync + 'static {
    /// Execute one admitted host task.
    fn execute_host_effect(
        &self,
        ctx: HostDispatchContext,
        task: HostTaskSpec,
    ) -> InProcessEffectFuture;
}

impl<F> InProcessEffectHandler for F
where
    F: Fn(HostDispatchContext, HostTaskSpec) -> InProcessEffectFuture + Send + Sync + 'static,
{
    fn execute_host_effect(
        &self,
        ctx: HostDispatchContext,
        task: HostTaskSpec,
    ) -> InProcessEffectFuture {
        self(ctx, task)
    }
}

/// Configuration for the feature-gated in-process runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InProcessRuntimeConfig {
    /// Maximum number of admitted, not-yet-completed tasks held by one partition.
    pub partition_queue_bound: usize,
    /// Maximum number of completed, failed and idempotency records retained per partition.
    pub partition_history_bound: usize,
    /// Maximum number of host effect handler calls running across all partitions.
    pub max_concurrent_effects: usize,
    /// Number of runtime events retained for late subscribers.
    pub event_buffer: usize,
}

impl InProcessRuntimeConfig {
    /// Create a runtime config.
    #[must_use]
    pub fn new(partition_queue_bound: usize, max_concurrent_effects: usize) -> Self {
        Self {
            partition_queue_bound,
            partition_history_bound: partition_queue_bound.saturating_mul(16).max(1),
            max_concurrent_effects,
            event_buffer: 64,
        }
    }

    fn validate(&self) -> Result<(), InProcessRuntimeError> {
        if self.partition_queue_bound == 0 {
            return Err(InProcessRuntimeError::InvalidConfig {
                field: "partition_queue_bound",
                value: 0,
            });
        }
        if self.max_concurrent_effects == 0 {
            return Err(InProcessRuntimeError::InvalidConfig {
                field: "max_concurrent_effects",
                value: 0,
            });
        }
        if self.partition_history_bound == 0 {
            return Err(InProcessRuntimeError::InvalidConfig {
                field: "partition_history_bound",
                value: 0,
            });
        }
        if self.partition_history_bound < self.partition_queue_bound {
            return Err(InProcessRuntimeError::InvalidConfig {
                field: "partition_history_bound",
                value: self.partition_history_bound,
            });
        }
        if self.event_buffer == 0 {
            return Err(InProcessRuntimeError::InvalidConfig {
                field: "event_buffer",
                value: 0,
            });
        }
        Ok(())
    }
}

impl Default for InProcessRuntimeConfig {
    fn default() -> Self {
        Self::new(64, 1)
    }
}

/// Admission behavior when a submit encounters backpressure.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InProcessBackpressureMode {
    /// Wait for route admission and ingress capacity.
    #[default]
    Wait,
    /// Return [`InProcessRuntimeError::QueueFull`] or
    /// [`InProcessRuntimeError::RouteBusy`] instead of waiting.
    FailFast,
}

/// Runtime-local backpressure policy for an explicit submit call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InProcessBackpressurePolicy {
    /// Admission behavior used by the submit call.
    pub mode: InProcessBackpressureMode,
}

impl InProcessBackpressurePolicy {
    /// Policy that preserves the default blocking submit behavior.
    #[must_use]
    pub const fn wait() -> Self {
        Self {
            mode: InProcessBackpressureMode::Wait,
        }
    }

    /// Policy that returns overload errors instead of waiting.
    #[must_use]
    pub const fn fail_fast() -> Self {
        Self {
            mode: InProcessBackpressureMode::FailFast,
        }
    }
}

impl Default for InProcessBackpressurePolicy {
    fn default() -> Self {
        Self::wait()
    }
}

/// Errors returned by [`InProcessRuntime`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InProcessRuntimeError {
    /// Runtime configuration is invalid.
    InvalidConfig {
        /// Invalid field name.
        field: &'static str,
        /// Invalid numeric value.
        value: usize,
    },
    /// The configured partition list was empty.
    NoPartitions,
    /// A partition key appeared more than once at startup.
    DuplicatePartition {
        /// Duplicate partition key.
        partition: PartitionKey,
    },
    /// A submit targeted a partition this runtime does not own.
    UnknownPartition {
        /// Unknown partition key.
        partition: PartitionKey,
        /// Rejected task id.
        task_id: String,
    },
    /// Host API validation or partition-local idempotency rejected the task.
    HostRejected {
        /// Host-facing dispatch error.
        error: HostDispatchError,
    },
    /// The bounded partition queue had no capacity.
    QueueFull {
        /// Full partition key.
        partition: PartitionKey,
        /// Rejected task id.
        task_id: String,
    },
    /// A non-blocking routed submit could not acquire route admission locks.
    RouteBusy {
        /// Busy partition key.
        partition: PartitionKey,
        /// Rejected task id.
        task_id: String,
    },
    /// The owning partition worker is closed.
    Closed {
        /// Closed partition key.
        partition: PartitionKey,
        /// Rejected task id.
        task_id: String,
    },
}

impl From<HostDispatchError> for InProcessRuntimeError {
    fn from(error: HostDispatchError) -> Self {
        Self::HostRejected { error }
    }
}

/// Runtime event stream emitted by [`InProcessRuntime`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InProcessRuntimeEvent {
    /// A task was admitted into its owning partition.
    Accepted {
        /// Owning partition.
        partition: PartitionKey,
        /// Admission ticket.
        ticket: HostDispatchTicket,
    },
    /// A task was rejected before admission.
    Rejected {
        /// Owning partition.
        partition: PartitionKey,
        /// Rejected task id, when one was supplied.
        task_id: Option<String>,
        /// Host-facing rejection error.
        error: HostDispatchError,
    },
    /// A task could not be admitted because the partition queue is full.
    QueueFull {
        /// Full partition key.
        partition: PartitionKey,
        /// Rejected task id.
        task_id: String,
    },
    /// A non-blocking routed submit could not acquire route admission locks.
    RouteBusy {
        /// Busy partition key.
        partition: PartitionKey,
        /// Rejected task id.
        task_id: String,
    },
    /// A task is waiting on dependencies that are not complete in this partition.
    Blocked {
        /// Owning partition.
        partition: PartitionKey,
        /// Blocked task id.
        task_id: String,
        /// Dependencies that are not complete in this partition.
        missing_dependencies: Vec<String>,
    },
    /// A host effect handler completed a task.
    Executed {
        /// Owning partition.
        partition: PartitionKey,
        /// Executed task id.
        task_id: String,
        /// Host-produced references.
        produced_refs: Vec<String>,
    },
    /// A host effect handler rejected or failed a task.
    Failed {
        /// Owning partition.
        partition: PartitionKey,
        /// Failed task id.
        task_id: String,
        /// Host-facing failure.
        error: HostDispatchError,
    },
}

/// Feature-gated Tokio runtime with partition-owned bounded queues.
pub struct InProcessRuntime {
    senders: HashMap<PartitionKey, mpsc::Sender<PartitionCommand>>,
    coordinator: AdmissionCoordinator,
    events: broadcast::Sender<InProcessRuntimeEvent>,
    ticket_sequence: Arc<AtomicU64>,
    workers: Vec<JoinHandle<()>>,
}

impl InProcessRuntime {
    /// Spawn an in-process runtime for a fixed set of partitions.
    ///
    /// # Errors
    /// Returns [`InProcessRuntimeError`] when config is invalid, no partitions
    /// are supplied, or a partition key is duplicated.
    pub fn spawn<H, I>(
        config: InProcessRuntimeConfig,
        partitions: I,
        handler: H,
    ) -> Result<Self, InProcessRuntimeError>
    where
        H: InProcessEffectHandler,
        I: IntoIterator<Item = PartitionKey>,
    {
        config.validate()?;

        let partitions = collect_unique_partitions(partitions)?;
        let handler = Arc::new(handler);
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_effects));
        let (events, _initial_receiver) = broadcast::channel(config.event_buffer);
        let coordinator = AdmissionCoordinator::new(&partitions);
        let mut spawner = PartitionWorkerSpawner::new(partitions.len());
        let mut senders = HashMap::new();
        let mut workers = Vec::new();

        for partition in partitions {
            let (sender, receiver) = mpsc::channel(config.partition_queue_bound);
            let worker = spawner.spawn_partition(partition_loop(
                partition.clone(),
                receiver,
                Arc::clone(&handler),
                Arc::clone(&semaphore),
                events.clone(),
                config.partition_queue_bound,
                config.partition_history_bound,
            ));
            let _previous = senders.insert(partition, sender);
            workers.push(worker);
        }

        Ok(Self {
            senders,
            coordinator,
            events,
            ticket_sequence: Arc::new(AtomicU64::new(0)),
            workers,
        })
    }

    /// Return capabilities advertised by this runtime adapter.
    #[must_use]
    pub fn capabilities(&self) -> HostDispatcherCapabilities {
        HostDispatcherCapabilities {
            api_version: CAUSLANE_HOST_API_VERSION.to_owned(),
            supports_linear_drain: false,
            supports_parallelism: true,
            supports_dependency_graph: true,
            supports_partition_coordination: true,
            requires_external_authz: true,
            requires_external_idempotency: true,
        }
    }

    /// Subscribe to runtime admission and execution events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<InProcessRuntimeEvent> {
        self.events.subscribe()
    }

    /// Submit a task, waiting for ingress capacity if the partition channel is full.
    ///
    /// # Errors
    /// Returns [`InProcessRuntimeError`] when host validation fails, the
    /// partition is unknown, the worker is closed, or partition-local admission
    /// rejects the task.
    pub async fn submit(
        &self,
        partition: &PartitionKey,
        ctx: HostDispatchContext,
        task: HostTaskSpec,
    ) -> Result<HostDispatchTicket, InProcessRuntimeError> {
        self.submit_with_backpressure(partition, ctx, task, InProcessBackpressurePolicy::wait())
            .await
    }

    /// Submit a task to its declared primary partition.
    pub async fn submit_routed(
        &self,
        ctx: HostDispatchContext,
        task: HostTaskSpec,
    ) -> Result<HostDispatchTicket, InProcessRuntimeError> {
        let partition = task.partition_route.primary.clone();
        self.submit(&partition, ctx, task).await
    }

    /// Submit a task with an explicit backpressure policy.
    ///
    /// `Wait` preserves [`Self::submit`] semantics. `FailFast` preserves
    /// [`Self::try_submit`] semantics while still awaiting the partition's
    /// admission response if the command enters the ingress channel.
    ///
    /// # Errors
    /// Returns [`InProcessRuntimeError`] when host validation fails, the
    /// partition is unknown, the worker is closed, route admission is busy, the
    /// queue is full, or partition-local admission rejects the task.
    pub async fn submit_with_backpressure(
        &self,
        partition: &PartitionKey,
        ctx: HostDispatchContext,
        task: HostTaskSpec,
        policy: InProcessBackpressurePolicy,
    ) -> Result<HostDispatchTicket, InProcessRuntimeError> {
        self.submit_with_mode(partition, ctx, task, policy.mode)
            .await
    }

    /// Submit a routed task with an explicit backpressure policy.
    ///
    /// # Errors
    /// Returns [`InProcessRuntimeError`] using the same rules as
    /// [`Self::submit_with_backpressure`].
    pub async fn submit_routed_with_backpressure(
        &self,
        ctx: HostDispatchContext,
        task: HostTaskSpec,
        policy: InProcessBackpressurePolicy,
    ) -> Result<HostDispatchTicket, InProcessRuntimeError> {
        let partition = task.partition_route.primary.clone();
        self.submit_with_backpressure(&partition, ctx, task, policy)
            .await
    }

    /// Submit a task without waiting for ingress channel capacity.
    ///
    /// This still awaits the owning partition's admission response when the
    /// command enters the channel; it returns [`InProcessRuntimeError::QueueFull`]
    /// immediately when the ingress channel itself is full.
    ///
    /// # Errors
    /// Returns [`InProcessRuntimeError`] when host validation fails, the
    /// partition is unknown, the channel is full/closed, or partition-local
    /// admission rejects the task.
    pub async fn try_submit(
        &self,
        partition: &PartitionKey,
        ctx: HostDispatchContext,
        task: HostTaskSpec,
    ) -> Result<HostDispatchTicket, InProcessRuntimeError> {
        self.submit_with_mode(partition, ctx, task, InProcessBackpressureMode::FailFast)
            .await
    }

    /// Submit a task to its declared primary partition without waiting.
    pub async fn try_submit_routed(
        &self,
        ctx: HostDispatchContext,
        task: HostTaskSpec,
    ) -> Result<HostDispatchTicket, InProcessRuntimeError> {
        let partition = task.partition_route.primary.clone();
        self.try_submit(&partition, ctx, task).await
    }

    async fn submit_with_mode(
        &self,
        partition: &PartitionKey,
        ctx: HostDispatchContext,
        task: HostTaskSpec,
        mode: InProcessBackpressureMode,
    ) -> Result<HostDispatchTicket, InProcessRuntimeError> {
        self.validate_submission_for_partition(partition, &ctx, &task)?;
        let sender = self.sender_for(partition, &task)?;
        let route = task.partition_route.clone();
        let (response_sender, response_receiver) = oneshot::channel();
        let command = self.build_command(ctx, task, response_sender);
        let task_id = command.task_id().to_owned();
        match mode {
            InProcessBackpressureMode::Wait => {
                self.enqueue_wait_command(partition, sender, &route, &task_id, command)
                    .await?;
            }
            InProcessBackpressureMode::FailFast => {
                self.enqueue_fail_fast_command(partition, sender, &route, &task_id, command)?;
            }
        }

        response_receiver
            .await
            .map_err(|_error| closed_error(partition, &task_id))?
    }

    async fn enqueue_wait_command(
        &self,
        partition: &PartitionKey,
        sender: &mpsc::Sender<PartitionCommand>,
        route: &causlane_core::PartitionRoute,
        task_id: &str,
        mut command: PartitionCommand,
    ) -> Result<(), InProcessRuntimeError> {
        self.coordinator.validate_route(route, task_id)?;
        let permit = sender
            .reserve()
            .await
            .map_err(|_error| closed_error(partition, task_id))?;
        let route_guards = self.coordinator.lock_route(route, task_id).await?;
        command.attach_admission_guards(route_guards);
        permit.send(command);
        Ok(())
    }

    fn enqueue_fail_fast_command(
        &self,
        partition: &PartitionKey,
        sender: &mpsc::Sender<PartitionCommand>,
        route: &causlane_core::PartitionRoute,
        task_id: &str,
        command: PartitionCommand,
    ) -> Result<(), InProcessRuntimeError> {
        let route_guards = self
            .coordinator
            .try_lock_route(route, task_id, &self.events)?;
        let mut command = command;
        command.attach_admission_guards(route_guards);
        match sender.try_send(command) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(command)) => {
                let task_id = command.task_id().to_owned();
                publish(
                    &self.events,
                    InProcessRuntimeEvent::QueueFull {
                        partition: partition.clone(),
                        task_id: task_id.clone(),
                    },
                );
                Err(InProcessRuntimeError::QueueFull {
                    partition: partition.clone(),
                    task_id,
                })
            }
            Err(mpsc::error::TrySendError::Closed(command)) => {
                Err(closed_error(partition, command.task_id()))
            }
        }
    }

    fn sender_for(
        &self,
        partition: &PartitionKey,
        task: &HostTaskSpec,
    ) -> Result<&mpsc::Sender<PartitionCommand>, InProcessRuntimeError> {
        self.senders
            .get(partition)
            .ok_or_else(|| InProcessRuntimeError::UnknownPartition {
                partition: partition.clone(),
                task_id: task.task_id.clone(),
            })
    }

    fn validate_submission_for_partition(
        &self,
        partition: &PartitionKey,
        ctx: &HostDispatchContext,
        task: &HostTaskSpec,
    ) -> Result<(), InProcessRuntimeError> {
        if let Err(error) = validate_host_submission(ctx, task) {
            self.publish_rejection(partition, task, error.clone());
            return Err(error.into());
        }
        if task.partition_route.primary != *partition {
            let error = HostDispatchError::InvalidPartitionRoute {
                task_id: task.task_id.clone(),
                reason: format!(
                    "explicit partition {} does not match route primary {}",
                    partition.0, task.partition_route.primary.0
                ),
            };
            self.publish_rejection(partition, task, error.clone());
            return Err(error.into());
        }
        Ok(())
    }

    fn publish_rejection(
        &self,
        partition: &PartitionKey,
        task: &HostTaskSpec,
        error: HostDispatchError,
    ) {
        publish(
            &self.events,
            InProcessRuntimeEvent::Rejected {
                partition: partition.clone(),
                task_id: non_empty_task_id(task),
                error,
            },
        );
    }

    fn build_command(
        &self,
        ctx: HostDispatchContext,
        task: HostTaskSpec,
        response: oneshot::Sender<Result<HostDispatchTicket, InProcessRuntimeError>>,
    ) -> PartitionCommand {
        let ticket = HostDispatchTicket {
            ticket_id: self.next_ticket_id(&ctx),
            task_id: task.task_id.clone(),
            api_version: CAUSLANE_HOST_API_VERSION.to_owned(),
        };

        PartitionCommand::Submit {
            ctx,
            task,
            ticket,
            admission_guards: Vec::new(),
            response,
        }
    }

    fn next_ticket_id(&self, ctx: &HostDispatchContext) -> String {
        let sequence = self.ticket_sequence.fetch_add(1, Ordering::Relaxed) + 1;
        format!("in-process-ticket:{}:{sequence}", ctx.correlation_id)
    }
}

impl Drop for InProcessRuntime {
    fn drop(&mut self) {
        for worker in &self.workers {
            worker.abort();
        }
    }
}

fn collect_unique_partitions<I>(partitions: I) -> Result<Vec<PartitionKey>, InProcessRuntimeError>
where
    I: IntoIterator<Item = PartitionKey>,
{
    let mut seen = HashSet::new();
    let mut collected = Vec::new();

    for partition in partitions {
        if !seen.insert(partition.clone()) {
            return Err(InProcessRuntimeError::DuplicatePartition { partition });
        }
        collected.push(partition);
    }

    if collected.is_empty() {
        return Err(InProcessRuntimeError::NoPartitions);
    }

    Ok(collected)
}

enum PartitionCommand {
    Submit {
        ctx: HostDispatchContext,
        task: HostTaskSpec,
        ticket: HostDispatchTicket,
        admission_guards: AdmissionGuards,
        response: oneshot::Sender<Result<HostDispatchTicket, InProcessRuntimeError>>,
    },
}

impl PartitionCommand {
    fn task_id(&self) -> &str {
        match self {
            PartitionCommand::Submit { task, .. } => &task.task_id,
        }
    }

    fn attach_admission_guards(&mut self, guards: AdmissionGuards) {
        match self {
            PartitionCommand::Submit {
                admission_guards, ..
            } => *admission_guards = guards,
        }
    }
}

fn publish(events: &broadcast::Sender<InProcessRuntimeEvent>, event: InProcessRuntimeEvent) {
    let _send_result = events.send(event);
}

fn closed_error(partition: &PartitionKey, task_id: &str) -> InProcessRuntimeError {
    InProcessRuntimeError::Closed {
        partition: partition.clone(),
        task_id: task_id.to_owned(),
    }
}

fn non_empty_task_id(task: &HostTaskSpec) -> Option<String> {
    (!task.task_id.is_empty()).then(|| task.task_id.clone())
}
