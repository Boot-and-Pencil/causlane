use super::*;

#[tokio::test(flavor = "current_thread")]
async fn rejects_invalid_host_task() -> TestResult {
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
async fn rejects_invalid_host_context() -> TestResult {
    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(2, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();
    let mut invalid_ctx = ctx();
    invalid_ctx.actor_ref = String::new();

    let result = runtime
        .submit(
            &partition,
            invalid_ctx,
            task("bad-context", Vec::new(), Some("bad-context")),
        )
        .await;

    assert!(matches!(
        result,
        Err(InProcessRuntimeError::HostRejected {
            error: HostDispatchError::InvalidContext {
                field: "actor_ref",
                ..
            }
        })
    ));
    assert!(matches!(
        recv_event(&mut events).await?,
        InProcessRuntimeEvent::Rejected {
            error: HostDispatchError::InvalidContext {
                field: "actor_ref",
                ..
            },
            ..
        }
    ));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn rejects_controlled_effect_without_idempotency() -> TestResult {
    let partition = partition("p1");
    let runtime = runtime(
        InProcessRuntimeConfig::new(2, 1),
        vec![partition.clone()],
        RecordingAsyncHandler::new(),
    )?;
    let mut events = runtime.subscribe();
    let mut invalid = task("hard-effect", Vec::new(), None);
    invalid.effect_class = HostEffectClass::ControlledEffect;
    invalid.confirmation_or_quorum_refs = vec!["approval://operator/quorum".to_owned()];

    let result = runtime.submit(&partition, ctx(), invalid).await;

    assert!(matches!(
        result,
        Err(InProcessRuntimeError::HostRejected {
            error: HostDispatchError::InvalidTask {
                field: "idempotency_key",
                ..
            }
        })
    ));
    assert!(matches!(
        recv_event(&mut events).await?,
        InProcessRuntimeEvent::Rejected {
            error: HostDispatchError::InvalidTask {
                field: "idempotency_key",
                ..
            },
            ..
        }
    ));
    Ok(())
}
