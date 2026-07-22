use std::{
    collections::{BTreeSet, VecDeque},
    future::Future,
    sync::Arc,
};

use causlane_core::{
    validate_host_effect_outcome, HostDispatchContext, HostDispatchError, HostDispatchTicket,
    HostEffectOutcome, HostTaskSpec,
};
use tokio::{
    sync::{broadcast, mpsc, Semaphore},
    task::{self, JoinHandle},
};

use super::{
    publish, InProcessEffectHandler, InProcessRuntimeError, InProcessRuntimeEvent, PartitionCommand,
};
use crate::partitions::PartitionKey;

pub(super) const HANDLER_PANICKED_REASON: &str = "host effect handler panicked";
const HANDLER_CANCELLED_REASON: &str = "host effect handler task cancelled";
const CAPACITY_SEMAPHORE_CLOSED_REASON: &str = "capacity semaphore closed";

pub(super) struct PartitionWorkerSpawner {
    remaining_budget: usize,
}

impl PartitionWorkerSpawner {
    pub(super) fn new(worker_budget: usize) -> Self {
        Self {
            remaining_budget: worker_budget,
        }
    }

    pub(super) fn spawn_partition<F>(&mut self, future: F) -> JoinHandle<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        debug_assert!(self.remaining_budget > 0);
        self.remaining_budget = self.remaining_budget.saturating_sub(1);
        task::spawn(future)
    }
}

struct QueuedTask {
    ctx: HostDispatchContext,
    task: HostTaskSpec,
}

struct PartitionState {
    pending: VecDeque<QueuedTask>,
    completed: BoundedSet,
    failed: BoundedSet,
    seen_idempotency_keys: BoundedSet,
}

impl PartitionState {
    fn new(history_bound: usize) -> Self {
        Self {
            pending: VecDeque::new(),
            completed: BoundedSet::new(history_bound),
            failed: BoundedSet::new(history_bound),
            seen_idempotency_keys: BoundedSet::new(history_bound),
        }
    }

    fn next_ready_index(&self) -> Option<usize> {
        self.pending.iter().position(|queued| {
            queued
                .task
                .dependencies
                .iter()
                .all(|dependency| self.completed.contains(dependency))
        })
    }

    fn first_blocked(&self) -> Option<(&HostTaskSpec, Vec<String>)> {
        self.pending.iter().find_map(|queued| {
            let missing = queued
                .task
                .dependencies
                .iter()
                .filter(|dependency| !self.completed.contains(dependency))
                .cloned()
                .collect::<Vec<_>>();
            (!missing.is_empty()).then_some((&queued.task, missing))
        })
    }
}

struct BoundedSet {
    bound: usize,
    order: VecDeque<String>,
    members: BTreeSet<String>,
}

impl BoundedSet {
    fn new(bound: usize) -> Self {
        debug_assert!(bound > 0);
        Self {
            bound,
            order: VecDeque::new(),
            members: BTreeSet::new(),
        }
    }

    fn contains(&self, value: &str) -> bool {
        self.members.contains(value)
    }

    fn insert(&mut self, value: String) -> bool {
        if !self.members.insert(value.clone()) {
            return false;
        }

        self.order.push_back(value);
        while self.members.len() > self.bound {
            if let Some(expired) = self.order.pop_front() {
                let _was_present = self.members.remove(&expired);
            }
        }
        true
    }
}

pub(super) async fn partition_loop<H>(
    partition: PartitionKey,
    mut receiver: mpsc::Receiver<PartitionCommand>,
    handler: Arc<H>,
    semaphore: Arc<Semaphore>,
    events: broadcast::Sender<InProcessRuntimeEvent>,
    pending_limit: usize,
    history_bound: usize,
) where
    H: InProcessEffectHandler,
{
    let mut state = PartitionState::new(history_bound);

    while let Some(command) = receiver.recv().await {
        match command {
            PartitionCommand::Submit {
                ctx,
                task,
                ticket,
                admission_guards,
                response,
            } => {
                let result = admit_task(
                    &partition,
                    &mut state,
                    &events,
                    pending_limit,
                    QueuedTask { ctx, task },
                    &ticket,
                );
                let accepted = result.is_ok();
                let _response_result = response.send(result);
                drop(admission_guards);

                if accepted {
                    drain_ready(
                        &partition,
                        &mut state,
                        &events,
                        Arc::clone(&handler),
                        Arc::clone(&semaphore),
                    )
                    .await;
                }
            }
        }
    }
}

fn admit_task(
    partition: &PartitionKey,
    state: &mut PartitionState,
    events: &broadcast::Sender<InProcessRuntimeEvent>,
    pending_limit: usize,
    queued: QueuedTask,
    ticket: &HostDispatchTicket,
) -> Result<HostDispatchTicket, InProcessRuntimeError> {
    if let Some(key) = &queued.task.idempotency_key {
        if state.seen_idempotency_keys.contains(key) {
            let error = HostDispatchError::DuplicateSuppressed {
                task_id: queued.task.task_id.clone(),
            };
            publish(
                events,
                InProcessRuntimeEvent::Rejected {
                    partition: partition.clone(),
                    task_id: Some(queued.task.task_id),
                    error: error.clone(),
                },
            );
            return Err(error.into());
        }
    }

    if state.pending.len() >= pending_limit {
        publish(
            events,
            InProcessRuntimeEvent::QueueFull {
                partition: partition.clone(),
                task_id: queued.task.task_id.clone(),
            },
        );
        return Err(InProcessRuntimeError::QueueFull {
            partition: partition.clone(),
            task_id: queued.task.task_id,
        });
    }

    if let Some(key) = &queued.task.idempotency_key {
        let _is_new = state.seen_idempotency_keys.insert(key.clone());
    }

    state.pending.push_back(queued);
    publish(
        events,
        InProcessRuntimeEvent::Accepted {
            partition: partition.clone(),
            ticket: ticket.clone(),
        },
    );
    Ok(ticket.clone())
}

async fn drain_ready<H>(
    partition: &PartitionKey,
    state: &mut PartitionState,
    events: &broadcast::Sender<InProcessRuntimeEvent>,
    handler: Arc<H>,
    semaphore: Arc<Semaphore>,
) where
    H: InProcessEffectHandler,
{
    loop {
        let Some(index) = state.next_ready_index() else {
            publish_blocked(partition, state, events);
            return;
        };
        let Some(queued) = state.pending.remove(index) else {
            publish_blocked(partition, state, events);
            return;
        };

        let task_id = queued.task.task_id.clone();
        let permit = match Arc::clone(&semaphore).acquire_owned().await {
            Ok(permit) => permit,
            Err(_error) => {
                let error = HostDispatchError::HandlerRejected {
                    reason: CAPACITY_SEMAPHORE_CLOSED_REASON.to_owned(),
                };
                publish_failed(partition, state, events, task_id, error);
                publish_blocked(partition, state, events);
                return;
            }
        };

        let task = queued.task;
        let outcome =
            execute_host_effect_supervised(Arc::clone(&handler), queued.ctx, task.clone())
                .await
                .and_then(|outcome| {
                    validate_host_effect_outcome(&task, &outcome)?;
                    Ok(outcome)
                });
        drop(permit);

        match outcome {
            Ok(outcome) => {
                let _is_new = state.completed.insert(task_id.clone());
                publish(
                    events,
                    InProcessRuntimeEvent::Executed {
                        partition: partition.clone(),
                        task_id,
                        produced_refs: outcome.produced_refs,
                        action_receipt_ref: outcome.action_receipt_ref,
                        audit_ref: outcome.audit_ref,
                    },
                );
            }
            Err(error) => {
                publish_failed(partition, state, events, task_id, error);
            }
        }
    }
}

async fn execute_host_effect_supervised<H>(
    handler: Arc<H>,
    ctx: HostDispatchContext,
    task: HostTaskSpec,
) -> Result<HostEffectOutcome, HostDispatchError>
where
    H: InProcessEffectHandler,
{
    let handle = AbortOnDrop::spawn(async move { handler.execute_host_effect(ctx, task).await });

    match handle.join().await {
        Ok(outcome) => outcome,
        Err(error) if error.is_panic() => Err(handler_rejected(HANDLER_PANICKED_REASON)),
        Err(_error) => Err(handler_rejected(HANDLER_CANCELLED_REASON)),
    }
}

struct AbortOnDrop<T> {
    handle: JoinHandle<T>,
    completed: bool,
}

impl<T> AbortOnDrop<T>
where
    T: Send + 'static,
{
    fn spawn(future: impl Future<Output = T> + Send + 'static) -> Self {
        Self {
            handle: task::spawn(future),
            completed: false,
        }
    }

    async fn join(mut self) -> Result<T, task::JoinError> {
        let result = (&mut self.handle).await;
        self.completed = true;
        result
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        if !self.completed {
            self.handle.abort();
        }
    }
}

fn handler_rejected(reason: &str) -> HostDispatchError {
    HostDispatchError::HandlerRejected {
        reason: reason.to_owned(),
    }
}

fn publish_failed(
    partition: &PartitionKey,
    state: &mut PartitionState,
    events: &broadcast::Sender<InProcessRuntimeEvent>,
    task_id: String,
    error: HostDispatchError,
) {
    let _is_new = state.failed.insert(task_id.clone());
    publish(
        events,
        InProcessRuntimeEvent::Failed {
            partition: partition.clone(),
            task_id,
            error,
        },
    );
}

fn publish_blocked(
    partition: &PartitionKey,
    state: &PartitionState,
    events: &broadcast::Sender<InProcessRuntimeEvent>,
) {
    if let Some((task, missing_dependencies)) = state.first_blocked() {
        publish(
            events,
            InProcessRuntimeEvent::Blocked {
                partition: partition.clone(),
                task_id: task.task_id.clone(),
                missing_dependencies,
            },
        );
    }
}
