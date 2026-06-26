use std::{
    collections::BTreeSet,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use causlane_core::{
    ActionId, HostDispatchContext, HostDispatchError, HostDispatchTicket, HostEffectClass,
    HostEffectOutcome, HostRuntimeProfile, HostTaskSpec, PartitionRoute, PredicateId, Timestamp,
    CAUSLANE_HOST_API_VERSION,
};
use tokio::{
    sync::{broadcast, mpsc},
    time::timeout,
};

use super::worker::HANDLER_PANICKED_REASON;
use super::{
    InProcessEffectFuture, InProcessEffectHandler, InProcessRuntime, InProcessRuntimeConfig,
    InProcessRuntimeError, InProcessRuntimeEvent,
};
use crate::partitions::PartitionKey;
use crate::shadow::{
    compare_shadow_events, ShadowExpectation, ShadowExpectationKind, ShadowStatus,
};

mod backpressure;
mod recovery;
mod retention;

type TestResult = Result<(), TestError>;

#[derive(Debug)]
enum TestError {
    Runtime,
    EventTimeout,
    EventClosed,
    EventLagged,
    HandlerEntryTimeout,
    HandlerEntryClosed,
    MissingEvent,
}

impl From<InProcessRuntimeError> for TestError {
    fn from(_error: InProcessRuntimeError) -> Self {
        Self::Runtime
    }
}

#[derive(Clone)]
struct RecordingAsyncHandler {
    active: Arc<AtomicUsize>,
    peak: Arc<AtomicUsize>,
    entered: Option<mpsc::Sender<String>>,
    sleep_for: Duration,
    fail_ids: BTreeSet<String>,
}

impl RecordingAsyncHandler {
    fn new() -> Self {
        Self {
            active: Arc::new(AtomicUsize::new(0)),
            peak: Arc::new(AtomicUsize::new(0)),
            entered: None,
            sleep_for: Duration::from_millis(0),
            fail_ids: BTreeSet::new(),
        }
    }

    fn with_entry_sender(sender: mpsc::Sender<String>, sleep_for: Duration) -> Self {
        Self {
            entered: Some(sender),
            sleep_for,
            ..Self::new()
        }
    }

    fn failing(task_id: &str) -> Self {
        Self {
            fail_ids: BTreeSet::from([task_id.to_owned()]),
            ..Self::new()
        }
    }
}

impl InProcessEffectHandler for RecordingAsyncHandler {
    fn execute_host_effect(
        &self,
        _ctx: HostDispatchContext,
        task: HostTaskSpec,
    ) -> InProcessEffectFuture {
        let active = Arc::clone(&self.active);
        let peak = Arc::clone(&self.peak);
        let entered = self.entered.clone();
        let sleep_for = self.sleep_for;
        let fail = self.fail_ids.contains(&task.task_id);

        Box::pin(async move {
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            update_peak(&peak, current);
            if let Some(sender) = entered {
                let _send_result = sender.try_send(task.task_id.clone());
            }
            if !sleep_for.is_zero() {
                tokio::time::sleep(sleep_for).await;
            }
            let _previous = active.fetch_sub(1, Ordering::SeqCst);

            if fail {
                return Err(HostDispatchError::HandlerRejected {
                    reason: format!("failed {}", task.task_id),
                });
            }

            Ok(HostEffectOutcome {
                produced_refs: vec![format!("fact://{}", task.task_id)],
            })
        })
    }
}

fn update_peak(peak: &AtomicUsize, current: usize) {
    let mut observed = peak.load(Ordering::SeqCst);
    while current > observed {
        match peak.compare_exchange(observed, current, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_previous) => return,
            Err(next) => observed = next,
        }
    }
}

fn ctx() -> HostDispatchContext {
    HostDispatchContext {
        actor_ref: "actor://stage8/test".to_owned(),
        trace_id: "trace-1".to_owned(),
        correlation_id: "corr-1".to_owned(),
        request_id: Some("req-1".to_owned()),
        config_snapshot_ref: "config://snapshot/1".to_owned(),
        idempotency_key: None,
        runtime_profile: HostRuntimeProfile::ParallelCapableButDisabled,
        created_at: Timestamp(1),
    }
}

fn partition(id: &str) -> PartitionKey {
    PartitionKey(id.to_owned())
}

fn task(id: &str, dependencies: Vec<String>, idempotency_key: Option<&str>) -> HostTaskSpec {
    HostTaskSpec {
        task_id: id.to_owned(),
        action_id: ActionId("foundation.task.enqueue".to_owned()),
        predicate_id: PredicateId("foundation.task.enqueue".to_owned()),
        subject_ref: format!("subject://{id}"),
        plan_hash: None,
        effect_class: HostEffectClass::SoftWrite,
        payload_ref: Some(format!("object://payload/{id}")),
        dependencies,
        idempotency_key: idempotency_key.map(str::to_owned),
        partition_route: PartitionRoute::for_primary(partition("p1")),
        host_api_version: CAUSLANE_HOST_API_VERSION.to_owned(),
    }
}

fn routed_task(
    partition: &PartitionKey,
    id: &str,
    dependencies: Vec<String>,
    idempotency_key: Option<&str>,
) -> HostTaskSpec {
    let mut routed = task(id, dependencies, idempotency_key);
    routed.partition_route = PartitionRoute::for_primary(partition.clone());
    routed
}

fn multi_route_task(
    primary: &PartitionKey,
    participants: Vec<PartitionKey>,
    id: &str,
    idempotency_key: Option<&str>,
) -> HostTaskSpec {
    let mut routed = routed_task(primary, id, Vec::new(), idempotency_key);
    routed.partition_route = PartitionRoute::new(primary.clone(), participants);
    routed
}

fn runtime(
    config: InProcessRuntimeConfig,
    partitions: Vec<PartitionKey>,
    handler: impl InProcessEffectHandler,
) -> Result<InProcessRuntime, TestError> {
    InProcessRuntime::spawn(config, partitions, handler).map_err(Into::into)
}

async fn recv_event(
    receiver: &mut broadcast::Receiver<InProcessRuntimeEvent>,
) -> Result<InProcessRuntimeEvent, TestError> {
    match timeout(Duration::from_secs(1), receiver.recv()).await {
        Ok(Ok(event)) => Ok(event),
        Ok(Err(broadcast::error::RecvError::Closed)) => Err(TestError::EventClosed),
        Ok(Err(broadcast::error::RecvError::Lagged(_count))) => Err(TestError::EventLagged),
        Err(_elapsed) => Err(TestError::EventTimeout),
    }
}

async fn recv_until(
    receiver: &mut broadcast::Receiver<InProcessRuntimeEvent>,
    matches: impl Fn(&InProcessRuntimeEvent) -> bool,
) -> Result<InProcessRuntimeEvent, TestError> {
    for _attempt in 0..16 {
        let event = recv_event(receiver).await?;
        if matches(&event) {
            return Ok(event);
        }
    }
    Err(TestError::MissingEvent)
}

async fn recv_entry(receiver: &mut mpsc::Receiver<String>) -> Result<String, TestError> {
    match timeout(Duration::from_secs(1), receiver.recv()).await {
        Ok(Some(task_id)) => Ok(task_id),
        Ok(None) => Err(TestError::HandlerEntryClosed),
        Err(_elapsed) => Err(TestError::HandlerEntryTimeout),
    }
}

async fn expect_executed(
    events: &mut broadcast::Receiver<InProcessRuntimeEvent>,
    expected_task_id: &str,
) -> TestResult {
    assert!(matches!(
        recv_until(events, |event| matches!(
            event,
            InProcessRuntimeEvent::Executed { task_id, .. } if task_id == expected_task_id
        ))
        .await?,
        InProcessRuntimeEvent::Executed { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn in_process_runtime_rejects_invalid_host_task() -> TestResult {
    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(2, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();
    let mut invalid = task("bad", Vec::new(), Some("bad"));
    invalid.host_api_version = "causlane.host-dispatch.v0".to_owned();

    let result = runtime.submit(&partition, ctx(), invalid).await;

    assert!(matches!(
        result,
        Err(InProcessRuntimeError::HostRejected {
            error: HostDispatchError::UnsupportedApiVersion { .. }
        })
    ));
    assert!(matches!(
        recv_event(&mut events).await?,
        InProcessRuntimeEvent::Rejected {
            error: HostDispatchError::UnsupportedApiVersion { .. },
            ..
        }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn in_process_runtime_suppresses_partition_local_duplicates() -> TestResult {
    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;

    let first = runtime
        .submit(&partition, ctx(), task("first", Vec::new(), Some("same")))
        .await;
    let duplicate = runtime
        .submit(&partition, ctx(), task("second", Vec::new(), Some("same")))
        .await;

    assert!(matches!(first, Ok(HostDispatchTicket { .. })));
    assert_eq!(
        duplicate,
        Err(InProcessRuntimeError::HostRejected {
            error: HostDispatchError::DuplicateSuppressed {
                task_id: "second".to_owned(),
            },
        })
    );
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn in_process_runtime_reports_partition_coordination_capability() -> TestResult {
    let runtime = runtime(
        InProcessRuntimeConfig::new(2, 1),
        vec![partition("p1")],
        RecordingAsyncHandler::new(),
    )?;

    let capabilities = runtime.capabilities();

    assert!(capabilities.supports_parallelism);
    assert!(capabilities.supports_partition_coordination);
    assert!(!capabilities.supports_linear_drain);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn try_submit_reports_bounded_queue_full() -> TestResult {
    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(1, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();

    let first = runtime
        .submit(
            &partition,
            ctx(),
            task("blocked-1", vec!["missing".to_owned()], Some("b1")),
        )
        .await;
    let second = runtime
        .try_submit(
            &partition,
            ctx(),
            task("blocked-2", vec!["missing".to_owned()], Some("b2")),
        )
        .await;

    assert!(matches!(first, Ok(HostDispatchTicket { .. })));
    assert_eq!(
        second,
        Err(InProcessRuntimeError::QueueFull {
            partition: partition.clone(),
            task_id: "blocked-2".to_owned(),
        })
    );
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::QueueFull { task_id, .. } if task_id == "blocked-2"
        ))
        .await?,
        InProcessRuntimeEvent::QueueFull { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn submit_routed_uses_declared_primary_partition() -> TestResult {
    let left = partition("left");
    let right = partition("right");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![left, right.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();

    let submitted = runtime
        .submit_routed(
            ctx(),
            routed_task(&right, "right-task", Vec::new(), Some("right")),
        )
        .await;

    assert!(matches!(submitted, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Accepted {
                partition,
                ticket
            } if *partition == right && ticket.task_id == "right-task"
        ))
        .await?,
        InProcessRuntimeEvent::Accepted { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn explicit_submit_rejects_route_primary_mismatch() -> TestResult {
    let left = partition("left");
    let right = partition("right");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![left.clone(), right.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();

    let result = runtime
        .submit(
            &left,
            ctx(),
            routed_task(&right, "wrong-route", Vec::new(), Some("wrong-route")),
        )
        .await;

    assert!(matches!(
        result,
        Err(InProcessRuntimeError::HostRejected {
            error: HostDispatchError::InvalidPartitionRoute { .. }
        })
    ));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Rejected {
                partition,
                task_id: Some(task_id),
                error: HostDispatchError::InvalidPartitionRoute { .. }
            } if *partition == left && task_id == "wrong-route"
        ))
        .await?,
        InProcessRuntimeEvent::Rejected { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn unknown_route_participant_rejects_before_admission() -> TestResult {
    let primary = partition("primary");
    let missing = partition("missing");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![primary.clone()],
        RecordingAsyncHandler::new(),
    )?;

    let result = runtime
        .submit_routed(
            ctx(),
            multi_route_task(
                &primary,
                vec![missing.clone()],
                "needs-missing",
                Some("needs-missing"),
            ),
        )
        .await;

    assert_eq!(
        result,
        Err(InProcessRuntimeError::UnknownPartition {
            partition: missing,
            task_id: "needs-missing".to_owned(),
        })
    );
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn reversed_multi_partition_routes_do_not_deadlock() -> TestResult {
    let left = partition("left");
    let right = partition("right");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 2),
        vec![left.clone(), right.clone()],
        RecordingAsyncHandler::new(),
    )?;

    let (left_result, right_result) = tokio::join!(
        runtime.submit_routed(
            ctx(),
            multi_route_task(&left, vec![right.clone()], "left-task", Some("left"))
        ),
        runtime.submit_routed(
            ctx(),
            multi_route_task(&right, vec![left.clone()], "right-task", Some("right"))
        )
    );

    assert!(matches!(left_result, Ok(HostDispatchTicket { .. })));
    assert!(matches!(right_result, Ok(HostDispatchTicket { .. })));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn independent_partitions_execute_concurrently() -> TestResult {
    let (entry_sender, mut entry_receiver) = mpsc::channel(4);
    let handler = RecordingAsyncHandler::with_entry_sender(entry_sender, Duration::from_millis(50));
    let peak = Arc::clone(&handler.peak);
    let left = partition("left");
    let right = partition("right");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 2),
        vec![left.clone(), right.clone()],
        handler,
    )?;

    let (left_result, right_result) = tokio::join!(
        runtime.submit(
            &left,
            ctx(),
            routed_task(&left, "left-task", Vec::new(), Some("left"))
        ),
        runtime.submit(
            &right,
            ctx(),
            routed_task(&right, "right-task", Vec::new(), Some("right"))
        )
    );

    assert!(matches!(left_result, Ok(HostDispatchTicket { .. })));
    assert!(matches!(right_result, Ok(HostDispatchTicket { .. })));
    let _first_entry = recv_entry(&mut entry_receiver).await?;
    let _second_entry = recv_entry(&mut entry_receiver).await?;
    assert_eq!(peak.load(Ordering::SeqCst), 2);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn semaphore_capacity_prevents_overlapping_handler_calls() -> TestResult {
    let (entry_sender, mut entry_receiver) = mpsc::channel(4);
    let handler = RecordingAsyncHandler::with_entry_sender(entry_sender, Duration::from_millis(50));
    let peak = Arc::clone(&handler.peak);
    let left = partition("left");
    let right = partition("right");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![left.clone(), right.clone()],
        handler,
    )?;

    let (left_result, right_result) = tokio::join!(
        runtime.submit(
            &left,
            ctx(),
            routed_task(&left, "left-task", Vec::new(), Some("left"))
        ),
        runtime.submit(
            &right,
            ctx(),
            routed_task(&right, "right-task", Vec::new(), Some("right"))
        )
    );

    assert!(matches!(left_result, Ok(HostDispatchTicket { .. })));
    assert!(matches!(right_result, Ok(HostDispatchTicket { .. })));
    let _first_entry = recv_entry(&mut entry_receiver).await?;
    let second_early = timeout(Duration::from_millis(10), entry_receiver.recv()).await;
    assert!(second_early.is_err());
    let _second_entry = recv_entry(&mut entry_receiver).await?;
    assert_eq!(peak.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn dependencies_execute_in_partition_local_order() -> TestResult {
    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();

    let child = runtime
        .submit(
            &partition,
            ctx(),
            task("child", vec!["root".to_owned()], Some("child")),
        )
        .await;
    let root = runtime
        .submit(&partition, ctx(), task("root", Vec::new(), Some("root")))
        .await;

    assert!(matches!(child, Ok(HostDispatchTicket { .. })));
    assert!(matches!(root, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Executed { task_id, .. } if task_id == "root"
        ))
        .await?,
        InProcessRuntimeEvent::Executed { .. }
    ));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Executed { task_id, .. } if task_id == "child"
        ))
        .await?,
        InProcessRuntimeEvent::Executed { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn handler_failure_does_not_complete_task_and_blocks_dependents() -> TestResult {
    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::failing("root"),
    )?;
    let mut events = runtime.subscribe();

    let root = runtime
        .submit(&partition, ctx(), task("root", Vec::new(), Some("root")))
        .await;
    let child = runtime
        .submit(
            &partition,
            ctx(),
            task("child", vec!["root".to_owned()], Some("child")),
        )
        .await;

    assert!(matches!(root, Ok(HostDispatchTicket { .. })));
    assert!(matches!(child, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Failed { task_id, .. } if task_id == "root"
        ))
        .await?,
        InProcessRuntimeEvent::Failed { .. }
    ));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Blocked {
                task_id,
                missing_dependencies,
                ..
            } if task_id == "child" && missing_dependencies == &vec!["root".to_owned()]
        ))
        .await?,
        InProcessRuntimeEvent::Blocked { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn shadow_comparison_matches_in_process_runtime_events() -> TestResult {
    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();

    let submitted = runtime
        .submit(&partition, ctx(), task("root", Vec::new(), Some("root")))
        .await;

    assert!(matches!(submitted, Ok(HostDispatchTicket { .. })));
    let accepted = recv_until(&mut events, |event| {
        matches!(
            event,
            InProcessRuntimeEvent::Accepted { ticket, .. } if ticket.task_id == "root"
        )
    })
    .await?;
    let executed = recv_until(&mut events, |event| {
        matches!(
            event,
            InProcessRuntimeEvent::Executed {
                task_id,
                produced_refs,
                ..
            } if task_id == "root" && produced_refs == &vec!["fact://root".to_owned()]
        )
    })
    .await?;
    let observed = [accepted, executed];
    let expectations = vec![
        ShadowExpectation::in_partition(partition.clone(), "root", ShadowExpectationKind::Accepted),
        ShadowExpectation::in_partition(
            partition,
            "root",
            ShadowExpectationKind::Executed {
                produced_refs: vec!["fact://root".to_owned()],
            },
        ),
    ];

    let comparison = compare_shadow_events(&expectations, observed.iter());

    assert_eq!(comparison.status, ShadowStatus::Match);
    assert_eq!(comparison.matched, expectations);
    assert!(comparison.mismatches.is_empty());
    assert!(comparison.unexpected.is_empty());
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn shadow_mismatch_does_not_enforce_runtime_execution() -> TestResult {
    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();

    let submitted = runtime
        .submit(&partition, ctx(), task("root", Vec::new(), Some("root")))
        .await;

    assert!(matches!(submitted, Ok(HostDispatchTicket { .. })));
    let accepted = recv_until(&mut events, |event| {
        matches!(
            event,
            InProcessRuntimeEvent::Accepted { ticket, .. } if ticket.task_id == "root"
        )
    })
    .await?;
    let executed = recv_until(&mut events, |event| {
        matches!(
            event,
            InProcessRuntimeEvent::Executed { task_id, .. } if task_id == "root"
        )
    })
    .await?;
    let observed = [accepted, executed];
    let accepted_expectation =
        ShadowExpectation::in_partition(partition.clone(), "root", ShadowExpectationKind::Accepted);
    let executed_expectation = ShadowExpectation::in_partition(
        partition,
        "root",
        ShadowExpectationKind::Executed {
            produced_refs: vec!["fact://wrong".to_owned()],
        },
    );
    let expectations = vec![accepted_expectation.clone(), executed_expectation];

    let comparison = compare_shadow_events(&expectations, observed.iter());

    assert_eq!(comparison.status, ShadowStatus::Mismatch);
    assert_eq!(comparison.matched, vec![accepted_expectation]);
    assert_eq!(comparison.mismatches.len(), 1);
    assert!(comparison
        .mismatches
        .first()
        .and_then(|mismatch| mismatch.actual.as_ref())
        .is_some());
    assert!(comparison.unexpected.is_empty());
    Ok(())
}
