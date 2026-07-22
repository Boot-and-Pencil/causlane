//! Deterministic reference implementation of the host-facing dispatcher API.
//!
//! This module deliberately does not expose the full semantic kernel.  It is a
//! small compatibility adapter for host projects that need to bind to a stable
//! dispatch seam before the production dispatcher internals are wired in.

use std::collections::{BTreeSet, VecDeque};

use causlane_core::{
    validate_host_context, validate_host_effect_outcome, validate_host_submission,
    HostDispatchContext, HostDispatchError, HostDispatchPort, HostDispatchTicket,
    HostDispatcherCapabilities, HostDrainOutcome, HostEffectHandler, HostTaskSpec,
};

/// Deterministic linear implementation of [`HostDispatchPort`].
///
/// Tasks are admitted after host API validation, then drained in queue order once
/// their declared dependencies have completed.  This reference implementation is
/// intentionally synchronous and single-process; it is suitable as a stable
/// integration seam and test double, not as a durable production queue.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LinearHostDispatcher {
    queue: VecDeque<HostTaskSpec>,
    completed: BTreeSet<String>,
    seen_idempotency_keys: BTreeSet<String>,
    ticket_sequence: u64,
}

impl LinearHostDispatcher {
    /// Create an empty linear dispatcher.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of queued tasks that have not yet executed.
    #[must_use]
    pub fn queued_len(&self) -> usize {
        self.queue.len()
    }

    /// Return true if `task_id` has completed in this process.
    #[must_use]
    pub fn has_completed(&self, task_id: &str) -> bool {
        self.completed.contains(task_id)
    }

    fn next_ready_index(&self) -> Option<usize> {
        self.queue.iter().position(|task| {
            task.dependencies
                .iter()
                .all(|dependency| self.completed.contains(dependency))
        })
    }

    fn next_ticket_id(&mut self, ctx: &HostDispatchContext) -> String {
        self.ticket_sequence = self.ticket_sequence.saturating_add(1);
        format!(
            "host-ticket:{}:{}",
            ctx.correlation_id, self.ticket_sequence
        )
    }
}

impl HostDispatchPort for LinearHostDispatcher {
    fn capabilities(&self) -> HostDispatcherCapabilities {
        HostDispatcherCapabilities::linear_reference()
    }

    fn submit(
        &mut self,
        ctx: &HostDispatchContext,
        task: HostTaskSpec,
    ) -> Result<HostDispatchTicket, HostDispatchError> {
        validate_host_submission(ctx, &task)?;

        if let Some(key) = &task.idempotency_key {
            if self.seen_idempotency_keys.contains(key) {
                return Err(HostDispatchError::DuplicateSuppressed {
                    task_id: task.task_id,
                });
            }
            let _is_new = self.seen_idempotency_keys.insert(key.clone());
        }

        let ticket = HostDispatchTicket {
            ticket_id: self.next_ticket_id(ctx),
            task_id: task.task_id.clone(),
            api_version: self.capabilities().api_version,
        };
        self.queue.push_back(task);
        Ok(ticket)
    }

    fn drain_once<H: HostEffectHandler>(
        &mut self,
        ctx: &HostDispatchContext,
        handler: &mut H,
    ) -> Result<HostDrainOutcome, HostDispatchError> {
        validate_host_context(ctx)?;
        if self.queue.is_empty() {
            return Ok(HostDrainOutcome::Idle);
        }

        let Some(index) = self.next_ready_index() else {
            return Ok(HostDrainOutcome::Blocked);
        };

        // `next_ready_index` returned a valid position; a missing slot is
        // unreachable, but we fail safe to `Idle` rather than panic (anti-panic
        // discipline — no `expect`/`unwrap` in production paths).
        let Some(task) = self.queue.remove(index) else {
            return Ok(HostDrainOutcome::Idle);
        };
        let outcome = handler.execute_host_effect(ctx, &task)?;
        validate_host_effect_outcome(&task, &outcome)?;
        let _is_new = self.completed.insert(task.task_id.clone());

        Ok(HostDrainOutcome::Executed {
            task_id: task.task_id,
            produced_refs: outcome.produced_refs,
            action_receipt_ref: outcome.action_receipt_ref,
            audit_ref: outcome.audit_ref,
        })
    }
}

#[cfg(test)]
mod tests {
    use causlane_core::{
        ActionId, HostEffectClass, HostEffectOutcome, HostRuntimeProfile, PartitionKey,
        PartitionRoute, PredicateId, Timestamp,
    };

    use super::*;

    #[derive(Default)]
    struct RecordingHandler {
        seen: Vec<String>,
    }

    impl HostEffectHandler for RecordingHandler {
        fn execute_host_effect(
            &mut self,
            _ctx: &HostDispatchContext,
            task: &HostTaskSpec,
        ) -> Result<HostEffectOutcome, HostDispatchError> {
            self.seen.push(task.task_id.clone());
            Ok(HostEffectOutcome {
                produced_refs: vec![format!("fact://{}", task.task_id)],
                action_receipt_ref: Some(format!("receipt://action/{}", task.task_id)),
                audit_ref: format!("audit://host/outcome/{}", task.task_id),
            })
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
            runtime_profile: HostRuntimeProfile::LinearOnly,
            created_at: Timestamp(1),
        }
    }

    fn task(id: &str, dependencies: Vec<String>, idempotency_key: Option<&str>) -> HostTaskSpec {
        HostTaskSpec {
            task_id: id.to_owned(),
            action_id: ActionId("foundation.task.enqueue".to_owned()),
            predicate_id: PredicateId("foundation.task.enqueue".to_owned()),
            subject_ref: format!("subject://{id}"),
            plan_hash: None,
            effect_class: HostEffectClass::SoftWrite,
            confirmation_or_quorum_refs: Vec::new(),
            audit_ref: format!("audit://host/admission/{id}"),
            payload_ref: Some(format!("object://payload/{id}")),
            dependencies,
            idempotency_key: idempotency_key.map(str::to_owned),
            partition_route: PartitionRoute::for_primary(PartitionKey("linear".to_owned())),
            host_api_version: causlane_core::CAUSLANE_HOST_API_VERSION.to_owned(),
        }
    }

    #[test]
    fn linear_dispatcher_rejects_invalid_host_context() {
        let mut ctx = ctx();
        ctx.correlation_id = String::new();
        let mut dispatcher = LinearHostDispatcher::new();

        assert!(matches!(
            dispatcher.submit(&ctx, task("bad-context", Vec::new(), Some("bad-context"))),
            Err(HostDispatchError::InvalidContext {
                field: "correlation_id",
                ..
            })
        ));
    }

    #[test]
    fn linear_dispatcher_rejects_controlled_effect_without_idempotency() {
        let ctx = ctx();
        let mut dispatcher = LinearHostDispatcher::new();
        let mut task = task("hard-effect", Vec::new(), None);
        task.effect_class = HostEffectClass::ControlledEffect;
        task.confirmation_or_quorum_refs = vec!["approval://operator/quorum".to_owned()];

        assert!(matches!(
            dispatcher.submit(&ctx, task),
            Err(HostDispatchError::InvalidTask {
                field: "idempotency_key",
                ..
            })
        ));
    }

    #[test]
    fn linear_dispatcher_validates_drain_context() -> Result<(), HostDispatchError> {
        let good_ctx = ctx();
        let mut bad_ctx = ctx();
        bad_ctx.actor_ref = String::new();
        let mut dispatcher = LinearHostDispatcher::new();
        let mut handler = RecordingHandler::default();

        let _task = dispatcher.submit(&good_ctx, task("root", Vec::new(), Some("root")))?;

        assert!(matches!(
            dispatcher.drain_once(&bad_ctx, &mut handler),
            Err(HostDispatchError::InvalidContext {
                field: "actor_ref",
                ..
            })
        ));
        Ok(())
    }

    #[test]
    fn linear_dispatcher_executes_ready_tasks_in_dependency_order() -> Result<(), HostDispatchError>
    {
        let ctx = ctx();
        let mut dispatcher = LinearHostDispatcher::new();
        let mut handler = RecordingHandler::default();

        let _child =
            dispatcher.submit(&ctx, task("child", vec!["root".to_owned()], Some("child")))?;
        let _root = dispatcher.submit(&ctx, task("root", Vec::new(), Some("root")))?;

        assert_eq!(
            dispatcher.drain_once(&ctx, &mut handler)?,
            HostDrainOutcome::Executed {
                task_id: "root".to_owned(),
                produced_refs: vec!["fact://root".to_owned()],
                action_receipt_ref: Some("receipt://action/root".to_owned()),
                audit_ref: "audit://host/outcome/root".to_owned(),
            }
        );
        assert_eq!(
            dispatcher.drain_once(&ctx, &mut handler)?,
            HostDrainOutcome::Executed {
                task_id: "child".to_owned(),
                produced_refs: vec!["fact://child".to_owned()],
                action_receipt_ref: Some("receipt://action/child".to_owned()),
                audit_ref: "audit://host/outcome/child".to_owned(),
            }
        );
        assert_eq!(handler.seen, vec!["root".to_owned(), "child".to_owned()]);
        assert!(dispatcher.has_completed("root"));
        assert!(dispatcher.has_completed("child"));
        Ok(())
    }

    #[test]
    fn linear_dispatcher_reports_blocked_when_no_dependency_is_ready(
    ) -> Result<(), HostDispatchError> {
        let ctx = ctx();
        let mut dispatcher = LinearHostDispatcher::new();
        let mut handler = RecordingHandler::default();

        let _child = dispatcher.submit(
            &ctx,
            task("child", vec!["missing".to_owned()], Some("child")),
        )?;

        assert_eq!(
            dispatcher.drain_once(&ctx, &mut handler)?,
            HostDrainOutcome::Blocked
        );
        Ok(())
    }

    #[test]
    fn linear_dispatcher_suppresses_duplicate_idempotency_keys() -> Result<(), HostDispatchError> {
        let ctx = ctx();
        let mut dispatcher = LinearHostDispatcher::new();

        let _first = dispatcher.submit(&ctx, task("first", Vec::new(), Some("same-key")))?;

        assert_eq!(
            dispatcher.submit(&ctx, task("second", Vec::new(), Some("same-key"))),
            Err(HostDispatchError::DuplicateSuppressed {
                task_id: "second".to_owned(),
            })
        );
        Ok(())
    }
}
