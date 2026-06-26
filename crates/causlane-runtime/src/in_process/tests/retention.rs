use super::*;

#[tokio::test(flavor = "current_thread")]
async fn in_process_runtime_rejects_history_bound_smaller_than_queue_bound() -> TestResult {
    let mut config = InProcessRuntimeConfig::new(4, 1);
    config.partition_history_bound = 3;

    let result =
        InProcessRuntime::spawn(config, vec![partition("p1")], RecordingAsyncHandler::new());

    assert!(matches!(
        result,
        Err(InProcessRuntimeError::InvalidConfig {
            field: "partition_history_bound",
            value: 3,
        })
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn idempotency_keys_are_reusable_after_history_eviction() -> TestResult {
    let partition = partition("p1");
    let mut config = InProcessRuntimeConfig::new(2, 1);
    config.partition_history_bound = 2;
    let runtime = runtime(
        config,
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();

    for (task_id, key) in [
        ("first", "same"),
        ("history-filler-1", "history-filler-1"),
        ("history-filler-2", "history-filler-2"),
    ] {
        let submitted = runtime
            .submit(&partition, ctx(), task(task_id, Vec::new(), Some(key)))
            .await;
        assert!(matches!(submitted, Ok(HostDispatchTicket { .. })));
        expect_executed(&mut events, task_id).await?;
    }

    let reused_after_eviction = runtime
        .submit(&partition, ctx(), task("second", Vec::new(), Some("same")))
        .await;

    assert!(matches!(
        reused_after_eviction,
        Ok(HostDispatchTicket { .. })
    ));
    expect_executed(&mut events, "second").await?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn evicted_completed_task_no_longer_satisfies_new_dependencies() -> TestResult {
    let partition = partition("p1");
    let mut config = InProcessRuntimeConfig::new(2, 1);
    config.partition_history_bound = 2;
    let runtime = runtime(
        config,
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();

    for task_id in ["root-1", "root-2", "root-3"] {
        let submitted = runtime
            .submit(&partition, ctx(), task(task_id, Vec::new(), Some(task_id)))
            .await;
        assert!(matches!(submitted, Ok(HostDispatchTicket { .. })));
        expect_executed(&mut events, task_id).await?;
    }

    let child = runtime
        .submit(
            &partition,
            ctx(),
            task("child", vec!["root-1".to_owned()], Some("child")),
        )
        .await;

    assert!(matches!(child, Ok(HostDispatchTicket { .. })));
    assert!(matches!(
        recv_until(&mut events, |event| matches!(
            event,
            InProcessRuntimeEvent::Blocked {
                task_id,
                missing_dependencies,
                ..
            } if task_id == "child" && missing_dependencies == &vec!["root-1".to_owned()]
        ))
        .await?,
        InProcessRuntimeEvent::Blocked { .. }
    ));
    Ok(())
}
