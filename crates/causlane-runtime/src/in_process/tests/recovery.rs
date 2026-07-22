use std::sync::Arc;

use tokio::sync::{mpsc, Notify};

use super::*;

#[derive(Clone)]
struct PausingHandler {
    pause_task_id: String,
    entered: mpsc::Sender<String>,
    release: Arc<Notify>,
}

impl PausingHandler {
    fn new(pause_task_id: &str, entered: mpsc::Sender<String>, release: Arc<Notify>) -> Self {
        Self {
            pause_task_id: pause_task_id.to_owned(),
            entered,
            release,
        }
    }
}

impl InProcessEffectHandler for PausingHandler {
    fn execute_host_effect(
        &self,
        _ctx: HostDispatchContext,
        task: HostTaskSpec,
    ) -> InProcessEffectFuture {
        let pause_task_id = self.pause_task_id.clone();
        let entered = self.entered.clone();
        let release = Arc::clone(&self.release);

        Box::pin(async move {
            if task.task_id == pause_task_id {
                let release_signal = release.notified();
                let _send_result = entered.try_send(task.task_id.clone());
                release_signal.await;
            } else {
                let _send_result = entered.try_send(task.task_id.clone());
            }

            Ok(HostEffectOutcome {
                produced_refs: vec![format!("fact://{}", task.task_id)],
                action_receipt_ref: Some(format!("receipt://action/{}", task.task_id)),
                audit_ref: format!("audit://host/outcome/{}", task.task_id),
            })
        })
    }
}

#[derive(Clone, Copy)]
enum PanicPhase {
    BeforeFuture,
    DuringPoll,
}

#[derive(Clone)]
struct PanickingHandler {
    panic_task_id: String,
    phase: PanicPhase,
}

impl PanickingHandler {
    fn new(panic_task_id: &str, phase: PanicPhase) -> Self {
        Self {
            panic_task_id: panic_task_id.to_owned(),
            phase,
        }
    }
}

impl InProcessEffectHandler for PanickingHandler {
    fn execute_host_effect(
        &self,
        _ctx: HostDispatchContext,
        task: HostTaskSpec,
    ) -> InProcessEffectFuture {
        if task.task_id == self.panic_task_id {
            match self.phase {
                PanicPhase::BeforeFuture => {
                    panic!("intentional handler panic before future creation")
                }
                PanicPhase::DuringPoll => {
                    return Box::pin(async {
                        panic!("intentional handler panic during future polling")
                    });
                }
            }
        }

        Box::pin(async move {
            Ok(HostEffectOutcome {
                produced_refs: vec![format!("fact://{}", task.task_id)],
                action_receipt_ref: Some(format!("receipt://action/{}", task.task_id)),
                audit_ref: format!("audit://host/outcome/{}", task.task_id),
            })
        })
    }
}

#[tokio::test(flavor = "current_thread")]
async fn slow_handler_keeps_fail_fast_overload_visible() -> TestResult {
    let partition = partition("p1");
    let (entry_sender, mut entry_receiver) = mpsc::channel(4);
    let release = Arc::new(Notify::new());
    let runtime = runtime(
        InProcessRuntimeConfig::new(1, 1),
        vec![partition.clone()],
        PausingHandler::new("slow-root", entry_sender, Arc::clone(&release)),
    )?;
    let mut events = runtime.subscribe();

    let slow = runtime
        .submit(
            &partition,
            ctx(),
            task("slow-root", Vec::new(), Some("slow-root")),
        )
        .await;
    assert!(matches!(slow, Ok(HostDispatchTicket { .. })));
    assert_eq!(recv_entry(&mut entry_receiver).await?, "slow-root");

    let queued = timeout(
        Duration::from_millis(25),
        runtime.try_submit(
            &partition,
            ctx(),
            task("queued", Vec::new(), Some("queued")),
        ),
    )
    .await;
    assert!(queued.is_err());

    let overflow = runtime
        .try_submit(
            &partition,
            ctx(),
            task("overflow", Vec::new(), Some("overflow")),
        )
        .await;
    assert_eq!(
        overflow,
        Err(InProcessRuntimeError::RouteBusy {
            partition: partition.clone(),
            task_id: "overflow".to_owned(),
        })
    );
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::RouteBusy { task_id, .. } if task_id == "overflow"
        ))
        .await?,
        InProcessRuntimeEvent::RouteBusy { .. }
    ));

    release.notify_waiters();
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Executed { task_id, .. } if task_id == "slow-root"
        ))
        .await?,
        InProcessRuntimeEvent::Executed { .. }
    ));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Executed { task_id, .. } if task_id == "queued"
        ))
        .await?,
        InProcessRuntimeEvent::Executed { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn provider_unavailable_fails_closed_without_execution() -> TestResult {
    let partition = partition("p1");
    let handler = |_ctx: HostDispatchContext, task: HostTaskSpec| -> InProcessEffectFuture {
        Box::pin(async move {
            Err(HostDispatchError::HandlerRejected {
                reason: format!("provider unavailable for {}", task.task_id),
            })
        })
    };
    let runtime = runtime(
        InProcessRuntimeConfig::new(2, 1),
        vec![partition.clone()],
        handler,
    )?;
    let mut events = runtime.subscribe();

    let submitted = runtime
        .submit(
            &partition,
            ctx(),
            task("provider-down", Vec::new(), Some("provider-down")),
        )
        .await;

    assert!(matches!(submitted, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Failed {
                task_id,
                error: HostDispatchError::HandlerRejected { reason },
                ..
            } if task_id == "provider-down" && reason == "provider unavailable for provider-down"
        ))
        .await?,
        InProcessRuntimeEvent::Failed { .. }
    ));
    assert!(timeout(
        Duration::from_millis(25),
        recv_until(&mut events, |event| {
            matches!(
                event,
                InProcessRuntimeEvent::Executed { task_id, .. } if task_id == "provider-down"
            )
        })
    )
    .await
    .is_err());
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn handler_panic_before_future_fails_task_and_worker_continues() -> TestResult {
    handler_panic_fails_task_and_worker_continues(PanicPhase::BeforeFuture).await
}

#[tokio::test(flavor = "current_thread")]
async fn handler_panic_during_poll_fails_task_and_worker_continues() -> TestResult {
    handler_panic_fails_task_and_worker_continues(PanicPhase::DuringPoll).await
}

async fn handler_panic_fails_task_and_worker_continues(phase: PanicPhase) -> TestResult {
    const PANIC_TASK_ID: &str = "panic-root";
    const AFTER_PANIC_TASK_ID: &str = "after-panic";

    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![partition.clone()],
        PanickingHandler::new(PANIC_TASK_ID, phase),
    )?;
    let mut events = runtime.subscribe();

    let panicking = runtime
        .submit(
            &partition,
            ctx(),
            task(PANIC_TASK_ID, Vec::new(), Some(PANIC_TASK_ID)),
        )
        .await;
    assert!(matches!(panicking, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Failed {
                task_id,
                error: HostDispatchError::HandlerRejected { reason },
                ..
            } if task_id == PANIC_TASK_ID && reason == HANDLER_PANICKED_REASON
        ))
        .await?,
        InProcessRuntimeEvent::Failed { .. }
    ));

    let after_panic = runtime
        .submit(
            &partition,
            ctx(),
            task(AFTER_PANIC_TASK_ID, Vec::new(), Some(AFTER_PANIC_TASK_ID)),
        )
        .await;
    assert!(matches!(after_panic, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Executed { task_id, .. } if task_id == AFTER_PANIC_TASK_ID
        ))
        .await?,
        InProcessRuntimeEvent::Executed { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn retry_after_failure_is_host_owned_and_idempotency_safe() -> TestResult {
    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::failing("root"),
    )?;
    let mut events = runtime.subscribe();

    let root = runtime
        .submit(
            &partition,
            ctx(),
            task("root", Vec::new(), Some("retry-key")),
        )
        .await;
    assert!(matches!(root, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Failed { task_id, .. } if task_id == "root"
        ))
        .await?,
        InProcessRuntimeEvent::Failed { .. }
    ));

    let duplicate_retry = runtime
        .submit(
            &partition,
            ctx(),
            task("root-retry-same-key", Vec::new(), Some("retry-key")),
        )
        .await;
    assert_eq!(
        duplicate_retry,
        Err(InProcessRuntimeError::HostRejected {
            error: HostDispatchError::DuplicateSuppressed {
                task_id: "root-retry-same-key".to_owned(),
            },
        })
    );
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Rejected {
                task_id: Some(task_id),
                error: HostDispatchError::DuplicateSuppressed { .. },
                ..
            } if task_id == "root-retry-same-key"
        ))
        .await?,
        InProcessRuntimeEvent::Rejected { .. }
    ));

    let host_retry = runtime
        .submit(
            &partition,
            ctx(),
            task("root-retry-new-key", Vec::new(), Some("retry-key-2")),
        )
        .await;
    assert!(matches!(host_retry, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Executed { task_id, .. } if task_id == "root-retry-new-key"
        ))
        .await?,
        InProcessRuntimeEvent::Executed { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn routed_drain_under_load_reports_busy_without_deadlock() -> TestResult {
    let left = partition("left");
    let right = partition("right");
    let (entry_sender, mut entry_receiver) = mpsc::channel(4);
    let release = Arc::new(Notify::new());
    let runtime = runtime(
        InProcessRuntimeConfig::new(2, 1),
        vec![left.clone(), right.clone()],
        PausingHandler::new("active", entry_sender, Arc::clone(&release)),
    )?;
    let mut events = runtime.subscribe();

    assert!(!runtime.capabilities().supports_linear_drain);
    let active = runtime
        .submit_routed(
            ctx(),
            multi_route_task(&left, vec![right.clone()], "active", Some("active")),
        )
        .await;
    assert!(matches!(active, Ok(HostDispatchTicket { .. })));
    assert_eq!(recv_entry(&mut entry_receiver).await?, "active");

    let held_route = timeout(
        Duration::from_millis(25),
        runtime.submit_routed(
            ctx(),
            multi_route_task(&left, vec![right.clone()], "held", Some("held")),
        ),
    )
    .await;
    assert!(held_route.is_err());

    let blocked = runtime
        .try_submit_routed(
            ctx(),
            multi_route_task(
                &right,
                vec![left.clone()],
                "blocked-route",
                Some("blocked-route"),
            ),
        )
        .await;
    assert_eq!(
        blocked,
        Err(InProcessRuntimeError::RouteBusy {
            partition: left.clone(),
            task_id: "blocked-route".to_owned(),
        })
    );
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::RouteBusy { task_id, .. } if task_id == "blocked-route"
        ))
        .await?,
        InProcessRuntimeEvent::RouteBusy { .. }
    ));

    release.notify_waiters();
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Executed { task_id, .. } if task_id == "active"
        ))
        .await?,
        InProcessRuntimeEvent::Executed { .. }
    ));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Executed { task_id, .. } if task_id == "held"
        ))
        .await?,
        InProcessRuntimeEvent::Executed { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn routed_wait_does_not_hold_participant_while_primary_ingress_is_full() -> TestResult {
    const ACTIVE_TASK_ID: &str = "active";
    const QUEUED_PRIMARY_TASK_ID: &str = "queued-primary";
    const BLOCKED_ROUTE_TASK_ID: &str = "blocked-route";
    const PARTICIPANT_TASK_ID: &str = "participant-independent";

    let participant = partition("a-participant");
    let primary = partition("z-primary");
    let (entry_sender, mut entry_receiver) = mpsc::channel(4);
    let release = Arc::new(Notify::new());
    let runtime = runtime(
        InProcessRuntimeConfig::new(1, 1),
        vec![participant.clone(), primary.clone()],
        PausingHandler::new(ACTIVE_TASK_ID, entry_sender, Arc::clone(&release)),
    )?;
    let mut events = runtime.subscribe();

    let active = runtime
        .submit(
            &primary,
            ctx(),
            routed_task(&primary, ACTIVE_TASK_ID, Vec::new(), Some(ACTIVE_TASK_ID)),
        )
        .await;
    assert!(matches!(active, Ok(HostDispatchTicket { .. })));
    assert_eq!(recv_entry(&mut entry_receiver).await?, ACTIVE_TASK_ID);

    let queued_primary = runtime.submit(
        &primary,
        ctx(),
        routed_task(
            &primary,
            QUEUED_PRIMARY_TASK_ID,
            Vec::new(),
            Some(QUEUED_PRIMARY_TASK_ID),
        ),
    );
    tokio::pin!(queued_primary);
    assert!(timeout(Duration::from_millis(25), queued_primary.as_mut())
        .await
        .is_err());

    let blocked_route = runtime.submit_routed(
        ctx(),
        multi_route_task(
            &primary,
            vec![participant.clone()],
            BLOCKED_ROUTE_TASK_ID,
            Some(BLOCKED_ROUTE_TASK_ID),
        ),
    );
    tokio::pin!(blocked_route);
    assert!(timeout(Duration::from_millis(25), blocked_route.as_mut())
        .await
        .is_err());

    let participant_result = runtime
        .try_submit(
            &participant,
            ctx(),
            routed_task(
                &participant,
                PARTICIPANT_TASK_ID,
                Vec::new(),
                Some(PARTICIPANT_TASK_ID),
            ),
        )
        .await;
    assert!(matches!(participant_result, Ok(HostDispatchTicket { .. })));

    release.notify_waiters();
    assert!(matches!(
        queued_primary.await,
        Ok(HostDispatchTicket { .. })
    ));
    assert!(matches!(blocked_route.await, Ok(HostDispatchTicket { .. })));

    expect_executed(&mut events, ACTIVE_TASK_ID).await?;
    expect_executed(&mut events, PARTICIPANT_TASK_ID).await?;
    expect_executed(&mut events, QUEUED_PRIMARY_TASK_ID).await?;
    expect_executed(&mut events, BLOCKED_ROUTE_TASK_ID).await?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn partition_restart_is_ephemeral_rejoin_not_durable_recovery() -> TestResult {
    let partition = partition("p1");
    {
        let runtime = runtime(
            InProcessRuntimeConfig::new(2, 1),
            vec![partition.clone()],
            RecordingAsyncHandler::new(),
        )?;
        let mut events = runtime.subscribe();
        let submitted = runtime
            .submit(
                &partition,
                ctx(),
                task("before-restart", Vec::new(), Some("before-restart")),
            )
            .await;

        assert!(matches!(submitted, Ok(HostDispatchTicket { .. })));
        assert!(matches!(
            recv_until(&mut events, |event| matches!(
                event,
                InProcessRuntimeEvent::Executed { task_id, .. } if task_id == "before-restart"
            ))
            .await?,
            InProcessRuntimeEvent::Executed { .. }
        ));
    }

    let runtime = runtime(
        InProcessRuntimeConfig::new(2, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();
    let submitted = runtime
        .submit(
            &partition,
            ctx(),
            task("after-restart", Vec::new(), Some("after-restart")),
        )
        .await;

    assert!(matches!(submitted, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Executed { task_id, .. } if task_id == "after-restart"
        ))
        .await?,
        InProcessRuntimeEvent::Executed { .. }
    ));
    Ok(())
}
