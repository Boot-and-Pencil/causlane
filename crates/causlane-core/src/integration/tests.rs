use super::*;

fn ctx() -> HostDispatchContext {
    HostDispatchContext {
        actor_ref: "actor://stage8/test".to_owned(),
        trace_id: "trace-1".to_owned(),
        correlation_id: "corr-1".to_owned(),
        request_id: Some("req-1".to_owned()),
        config_snapshot_ref: "config://snapshot/1".to_owned(),
        idempotency_key: None,
        runtime_profile: HostRuntimeProfile::LinearOnly,
        created_at: Timestamp(1),
    }
}

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
fn host_api_rejects_empty_context_refs() {
    let mut bad = ctx();
    bad.actor_ref = String::new();
    assert_eq!(
        validate_host_context(&bad),
        Err(HostDispatchError::InvalidContext {
            field: "actor_ref",
            reason: "must be non-empty".to_owned(),
        })
    );

    let mut bad_optional = ctx();
    bad_optional.request_id = Some(String::new());
    assert_eq!(
        validate_host_context(&bad_optional),
        Err(HostDispatchError::InvalidContext {
            field: "request_id",
            reason: "must be non-empty when supplied".to_owned(),
        })
    );
}

#[test]
fn host_api_rejects_empty_task_refs() {
    let mut bad = task(HostEffectClass::SoftWrite);
    bad.action_id = ActionId(String::new());

    assert_eq!(
        validate_host_task(&bad),
        Err(HostDispatchError::InvalidTask {
            task_id: Some("task-1".to_owned()),
            field: "action_id",
            reason: "must be non-empty".to_owned(),
        })
    );
}

#[test]
fn host_api_rejects_dependency_shape_defects() {
    let mut self_dependency = task(HostEffectClass::SoftWrite);
    self_dependency.dependencies = vec!["task-1".to_owned()];
    assert_eq!(
        validate_host_task(&self_dependency),
        Err(HostDispatchError::InvalidTask {
            task_id: Some("task-1".to_owned()),
            field: "dependencies",
            reason: "task cannot depend on itself".to_owned(),
        })
    );

    let mut duplicate_dependency = task(HostEffectClass::SoftWrite);
    duplicate_dependency.dependencies = vec!["root".to_owned(), "root".to_owned()];
    assert_eq!(
        validate_host_task(&duplicate_dependency),
        Err(HostDispatchError::InvalidTask {
            task_id: Some("task-1".to_owned()),
            field: "dependencies",
            reason: "duplicate dependency task id root".to_owned(),
        })
    );
}

#[test]
fn host_api_requires_task_idempotency_for_hard_effects() {
    let mut bad = task(HostEffectClass::HardEffect);
    bad.idempotency_key = None;

    assert_eq!(
        validate_host_task(&bad),
        Err(HostDispatchError::InvalidTask {
            task_id: Some("task-1".to_owned()),
            field: "idempotency_key",
            reason: "hard effects require a host task idempotency key".to_owned(),
        })
    );
}

#[test]
fn host_submission_validates_context_before_task() {
    let mut bad_ctx = ctx();
    bad_ctx.trace_id = String::new();
    let mut bad_task = task(HostEffectClass::SoftWrite);
    bad_task.host_api_version = "causlane.host-dispatch.v0".to_owned();

    assert_eq!(
        validate_host_submission(&bad_ctx, &bad_task),
        Err(HostDispatchError::InvalidContext {
            field: "trace_id",
            reason: "must be non-empty".to_owned(),
        })
    );
}

#[test]
fn builders_set_current_api_version_and_validate() -> Result<(), HostDispatchError> {
    let ctx = HostDispatchContextBuilder::new(
        "actor://stage8/test",
        "trace-1",
        "corr-1",
        "config://snapshot/1",
        Timestamp(1),
    )
    .with_request_id("req-1")
    .with_runtime_profile(HostRuntimeProfile::ParallelCapableButDisabled)
    .build()?;

    let task = HostTaskSpecBuilder::new(
        "task-1",
        ActionId("foundation.task.enqueue".to_owned()),
        PredicateId("foundation.task.enqueue".to_owned()),
        "host://subject/demo",
        HostEffectClass::SoftWrite,
        PartitionRoute::for_primary(PartitionKey("partition-1".to_owned())),
    )
    .with_payload_ref("object://payload/demo")
    .with_dependencies(["root"])
    .with_idempotency_key("idem-1")
    .build()?;

    assert_eq!(
        ctx.runtime_profile,
        HostRuntimeProfile::ParallelCapableButDisabled
    );
    assert_eq!(task.host_api_version, CAUSLANE_HOST_API_VERSION);
    assert_eq!(task.dependencies, vec!["root".to_owned()]);
    Ok(())
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
