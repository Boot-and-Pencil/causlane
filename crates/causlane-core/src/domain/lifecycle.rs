//! Lifecycle reducer and transition guards.

use super::{AuditEventKind, ConsequenceProfile};

/// The stage an action's lifecycle has reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleStage {
    /// Not yet admitted.
    New,
    /// Admitted into dispatch.
    Admitted,
    /// Compiled into a plan.
    Planned,
    /// Dispatch logged.
    DispatchLogged,
    /// Execution barrier logged (leases held).
    ExecutionBarrierLogged,
    /// Execution in progress.
    Executing,
    /// Observed truth committed.
    Observed,
    /// Projection emitted.
    Projected,
    /// Lifecycle closed.
    Closed,
}

/// A rejected lifecycle transition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleViolation {
    /// The (stage, event, profile) triple is not a permitted transition.
    ForbiddenTransition {
        /// The stage the action was in.
        from: LifecycleStage,
        /// The event that was attempted.
        event: AuditEventKind,
        /// The consequence profile in effect.
        profile: ConsequenceProfile,
    },
}

/// Apply an event to a lifecycle stage, returning the next stage or a violation.
// explicit transition table reads clearer than merged arms
#[allow(clippy::match_same_arms)]
pub fn reduce_lifecycle(
    from: LifecycleStage,
    event: AuditEventKind,
    profile: ConsequenceProfile,
) -> Result<LifecycleStage, LifecycleViolation> {
    use AuditEventKind as E;
    use ConsequenceProfile as P;
    use LifecycleStage as S;

    if from == S::Closed {
        return Err(LifecycleViolation::ForbiddenTransition {
            from,
            event,
            profile,
        });
    }

    let next = match (profile, from, event) {
        (_, S::New, E::ActionAdmitted) => S::Admitted,
        (_, S::Admitted, E::ActionPlanned) => S::Planned,
        (_, S::Planned, E::DispatchLogged) => S::DispatchLogged,

        (P::RuntimeExecution, S::DispatchLogged, E::GateApproved) => S::DispatchLogged,
        (P::RuntimeExecution, S::DispatchLogged, E::AuthzDecisionRecorded) => S::DispatchLogged,
        (P::RuntimeExecution, S::DispatchLogged, E::ConstraintLeaseGranted) => S::DispatchLogged,
        (P::RuntimeExecution, S::DispatchLogged, E::DrainFenceRequested) => S::DispatchLogged,
        (P::RuntimeExecution, S::DispatchLogged, E::DrainFenceAcquired) => S::DispatchLogged,
        (P::RuntimeExecution, S::DispatchLogged, E::ExecutionBarrierLogged) => {
            S::ExecutionBarrierLogged
        }
        (P::RuntimeExecution, S::ExecutionBarrierLogged, E::ExecutionStarted) => S::Executing,
        (P::RuntimeExecution, S::Executing, E::ExecutionCompleted) => S::Executing,
        (P::RuntimeExecution, S::Executing, E::ObservedTruthCommitted) => S::Observed,
        (P::RuntimeExecution, S::Observed, E::ProjectionEmitted) => S::Projected,
        (P::RuntimeExecution, S::Projected, E::ConstraintLeaseReleased) => S::Projected,
        (P::RuntimeExecution, S::Projected, E::LifecycleClosed) => S::Closed,

        // Observational records (a denied gate or a detected violation) do not
        // advance or terminate the lifecycle — they are evidence the audit
        // vocabulary is designed to carry, and replay already treats them as
        // no-ops in the lease/witness/capability passes. Accept them as
        // stage-preserving rather than rejecting valid histories that record
        // the deny/violation path.
        (P::RuntimeExecution, _, E::GateDenied | E::ViolationDetected) => from,

        (P::ProjectionRead, S::DispatchLogged, E::ProjectionEmitted) => S::Projected,
        (P::ProjectionRead, S::Projected, E::LifecycleClosed) => S::Closed,

        (
            P::OversightMeta | P::TopologyMeta | P::EvidenceMeta | P::OutsideKernel,
            S::DispatchLogged,
            E::LifecycleClosed,
        ) => S::Closed,

        (_profile, _from, _event) => {
            return Err(LifecycleViolation::ForbiddenTransition {
                from,
                event,
                profile,
            });
        }
    };

    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::{reduce_lifecycle, LifecycleStage as S};
    use crate::contract::{KernelContracts, LifecycleGrammar};
    use crate::domain::{AuditEventKind as E, ConsequenceProfile as P};

    /// Every `LifecycleStage` variant — kept exhaustive so the property test below
    /// is a complete proof over the finite input space.
    const ALL_STAGES: [S; 9] = [
        S::New,
        S::Admitted,
        S::Planned,
        S::DispatchLogged,
        S::ExecutionBarrierLogged,
        S::Executing,
        S::Observed,
        S::Projected,
        S::Closed,
    ];

    /// Every `AuditEventKind` variant.
    const ALL_EVENTS: [E; 17] = [
        E::ActionAdmitted,
        E::ActionPlanned,
        E::DispatchLogged,
        E::ExecutionBarrierLogged,
        E::ExecutionStarted,
        E::ExecutionCompleted,
        E::ObservedTruthCommitted,
        E::ProjectionEmitted,
        E::LifecycleClosed,
        E::GateApproved,
        E::GateDenied,
        E::ConstraintLeaseGranted,
        E::ConstraintLeaseReleased,
        E::ViolationDetected,
        E::AuthzDecisionRecorded,
        E::DrainFenceRequested,
        E::DrainFenceAcquired,
    ];

    /// Every `ConsequenceProfile` variant.
    const ALL_PROFILES: [P; 6] = [
        P::RuntimeExecution,
        P::ProjectionRead,
        P::OversightMeta,
        P::TopologyMeta,
        P::EvidenceMeta,
        P::OutsideKernel,
    ];

    // A recorded gate denial or detected violation is observational: it must
    // reduce without error and leave the stage unchanged, at any reachable
    // RuntimeExecution stage.
    #[test]
    fn observational_events_are_stage_preserving() {
        for stage in [
            S::DispatchLogged,
            S::ExecutionBarrierLogged,
            S::Executing,
            S::Observed,
            S::Projected,
        ] {
            for event in [E::GateDenied, E::ViolationDetected] {
                assert_eq!(
                    reduce_lifecycle(stage, event, P::RuntimeExecution),
                    Ok(stage),
                    "{event:?} at {stage:?} should be a stage-preserving no-op",
                );
            }
        }
    }

    // OutsideKernel routes as a Meta lifecycle class (routing.rs) and so must be
    // able to close from DispatchLogged like the other Meta profiles.
    #[test]
    fn outside_kernel_can_close() {
        assert_eq!(
            reduce_lifecycle(S::DispatchLogged, E::LifecycleClosed, P::OutsideKernel),
            Ok(S::Closed),
        );
    }

    #[test]
    fn closed_rejects_observational_noops() {
        for event in [E::GateDenied, E::ViolationDetected] {
            assert!(
                reduce_lifecycle(S::Closed, event, P::RuntimeExecution).is_err(),
                "{event:?} must not be accepted after Closed",
            );
        }
    }

    // Sanity: a genuinely forbidden transition still fails closed.
    #[test]
    fn forbidden_transition_still_rejected() {
        assert!(reduce_lifecycle(S::New, E::ExecutionStarted, P::RuntimeExecution).is_err());
    }

    // Property test (complete, not sampled): enumerate the entire finite
    // (stage, event, profile) input space and assert the grammar's structural
    // invariants hold for every triple.
    #[test]
    fn lifecycle_grammar_properties_hold_exhaustively() {
        // The terminal predicate is exactly "Closed", for every stage.
        for &stage in &ALL_STAGES {
            assert_eq!(KernelContracts.is_terminal(stage), stage == S::Closed);
        }
        for &profile in &ALL_PROFILES {
            // Every lifecycle starts un-admitted, regardless of profile.
            assert_eq!(KernelContracts.initial_stage(profile), S::New);
            for &stage in &ALL_STAGES {
                for &event in &ALL_EVENTS {
                    let result = reduce_lifecycle(stage, event, profile);
                    // Determinism: the reducer is a pure function of its inputs.
                    assert_eq!(result, reduce_lifecycle(stage, event, profile));
                    // I-008: Closed is absorbing — no event advances from it.
                    if stage == S::Closed {
                        assert!(
                            result.is_err(),
                            "Closed must reject {event:?} under {profile:?}",
                        );
                    }
                    // The terminal stage is reachable only via LifecycleClosed.
                    if result == Ok(S::Closed) {
                        assert_eq!(
                            event,
                            E::LifecycleClosed,
                            "only LifecycleClosed may reach Closed (from {stage:?}/{profile:?})",
                        );
                    }
                }
            }
        }
    }
}
