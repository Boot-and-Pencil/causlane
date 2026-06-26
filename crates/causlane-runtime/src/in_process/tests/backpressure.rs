use super::*;

use crate::in_process::{InProcessBackpressureMode, InProcessBackpressurePolicy};

#[test]
fn backpressure_policy_defaults_to_wait() {
    assert_eq!(
        InProcessBackpressurePolicy::default(),
        InProcessBackpressurePolicy::wait()
    );
    assert_eq!(
        InProcessBackpressurePolicy::fail_fast().mode,
        InProcessBackpressureMode::FailFast
    );
}

#[tokio::test(flavor = "current_thread")]
async fn wait_policy_admits_like_submit() -> TestResult {
    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(2, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();

    let submitted = runtime
        .submit_with_backpressure(
            &partition,
            ctx(),
            task("wait-policy", Vec::new(), Some("wait-policy")),
            InProcessBackpressurePolicy::wait(),
        )
        .await;

    assert!(matches!(submitted, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Accepted {
                partition: event_partition,
                ticket
            } if *event_partition == partition && ticket.task_id == "wait-policy"
        ))
        .await?,
        InProcessRuntimeEvent::Accepted { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn routed_wait_policy_uses_declared_primary_partition() -> TestResult {
    let left = partition("left");
    let right = partition("right");
    let runtime = runtime(
        InProcessRuntimeConfig::new(4, 1),
        vec![left, right.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();

    let submitted = runtime
        .submit_routed_with_backpressure(
            ctx(),
            routed_task(&right, "right-policy", Vec::new(), Some("right-policy")),
            InProcessBackpressurePolicy::wait(),
        )
        .await;

    assert!(matches!(submitted, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Accepted { partition, ticket }
                if *partition == right && ticket.task_id == "right-policy"
        ))
        .await?,
        InProcessRuntimeEvent::Accepted { .. }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn fail_fast_policy_matches_try_submit_queue_full_surface() -> TestResult {
    let partition = partition("p1");
    let try_result = queue_full_result(BackpressureCall::TrySubmit, &partition).await?;
    let policy_result = queue_full_result(BackpressureCall::FailFastPolicy, &partition).await?;

    assert_eq!(policy_result, try_result);
    Ok(())
}

enum BackpressureCall {
    TrySubmit,
    FailFastPolicy,
}

async fn queue_full_result(
    call: BackpressureCall,
    partition: &PartitionKey,
) -> Result<Result<HostDispatchTicket, InProcessRuntimeError>, TestError> {
    let runtime = runtime(
        InProcessRuntimeConfig::new(1, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();

    let first = runtime
        .submit(
            partition,
            ctx(),
            task("blocked-1", vec!["missing".to_owned()], Some("blocked-1")),
        )
        .await;
    assert!(matches!(first, Ok(HostDispatchTicket { .. })));

    let result = match call {
        BackpressureCall::TrySubmit => {
            runtime
                .try_submit(
                    partition,
                    ctx(),
                    task("blocked-2", vec!["missing".to_owned()], Some("blocked-2")),
                )
                .await
        }
        BackpressureCall::FailFastPolicy => {
            runtime
                .submit_with_backpressure(
                    partition,
                    ctx(),
                    task("blocked-2", vec!["missing".to_owned()], Some("blocked-2")),
                    InProcessBackpressurePolicy::fail_fast(),
                )
                .await
        }
    };

    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::QueueFull { task_id, .. } if task_id == "blocked-2"
        ))
        .await?,
        InProcessRuntimeEvent::QueueFull { .. }
    ));
    Ok(result)
}
