//! PUB1 protocol property checks for the core semantic authorities.
//!
//! These properties exercise `KernelContracts` as the public contract surface and
//! compare it with the existing reducer/arbiter functions. They intentionally do
//! not reimplement lifecycle or constraint semantics.

use causlane_core::{
    reduce_lifecycle, resolve_constraints, AuditEventKind, ClaimMode, ConsequenceProfile,
    ConstraintDecision, ConstraintEpoch, ConstraintId, ConstraintKind, ConstraintProvider,
    ConstraintSnapshot, ConstraintSpec, KernelContracts, LifecycleGrammar, LifecycleStage,
    ResourceClaim, ResourceId, Scope, ALL_AUDIT_EVENT_KINDS,
};
use proptest::prelude::*;

const ALL_STAGES: [LifecycleStage; 9] = [
    LifecycleStage::New,
    LifecycleStage::Admitted,
    LifecycleStage::Planned,
    LifecycleStage::DispatchLogged,
    LifecycleStage::ExecutionBarrierLogged,
    LifecycleStage::Executing,
    LifecycleStage::Observed,
    LifecycleStage::Projected,
    LifecycleStage::Closed,
];

const ALL_PROFILES: [ConsequenceProfile; 6] = [
    ConsequenceProfile::RuntimeExecution,
    ConsequenceProfile::ProjectionRead,
    ConsequenceProfile::OversightMeta,
    ConsequenceProfile::TopologyMeta,
    ConsequenceProfile::EvidenceMeta,
    ConsequenceProfile::OutsideKernel,
];

fn lifecycle_stage() -> impl Strategy<Value = LifecycleStage> {
    proptest::sample::select(&ALL_STAGES)
}

fn audit_event_kind() -> impl Strategy<Value = AuditEventKind> {
    proptest::sample::select(&ALL_AUDIT_EVENT_KINDS)
}

fn consequence_profile() -> impl Strategy<Value = ConsequenceProfile> {
    proptest::sample::select(&ALL_PROFILES)
}

fn token_claim(amount: u64) -> ResourceClaim {
    ResourceClaim {
        resource: ResourceId("slots".to_owned()),
        scope: Scope("pool:a".to_owned()),
        mode: ClaimMode::Token,
        amount,
    }
}

fn token_budget_snapshot(limit: u64) -> ConstraintSnapshot {
    ConstraintSnapshot {
        snapshot_id: ConstraintId("snap".to_owned()),
        epoch: ConstraintEpoch(1),
        constraints: vec![ConstraintSpec {
            constraint_id: ConstraintId("tokens".to_owned()),
            kind: ConstraintKind::TokenBudget {
                resource: ResourceId("slots".to_owned()),
                scope: Scope("pool:a".to_owned()),
                limit,
            },
        }],
        active_leases: Vec::new(),
    }
}

proptest! {
    #[test]
    fn lifecycle_contract_surface_delegates_to_reducer(
        stage in lifecycle_stage(),
        event in audit_event_kind(),
        profile in consequence_profile(),
    ) {
        let direct = reduce_lifecycle(stage, event, profile);
        let delegated = KernelContracts.reduce(stage, event, profile);

        prop_assert_eq!(&delegated, &direct);
        prop_assert_eq!(KernelContracts.initial_stage(profile), LifecycleStage::New);
        prop_assert_eq!(KernelContracts.is_terminal(stage), stage == LifecycleStage::Closed);

        if let Ok(next) = direct {
            prop_assert!(
                !KernelContracts.is_terminal(next) || event == AuditEventKind::LifecycleClosed,
                "only LifecycleClosed may reach the terminal stage"
            );
        }
    }

    #[test]
    fn token_budget_contract_surface_preserves_documented_outcomes(
        limit in 0_u64..=64,
        first_amount in 0_u64..=80,
        second_amount in 0_u64..=80,
    ) {
        let snapshot = token_budget_snapshot(limit);
        let claims = vec![token_claim(first_amount), token_claim(second_amount)];
        let direct = resolve_constraints(&snapshot, &claims, &KernelContracts);
        let delegated = KernelContracts.resolve(&snapshot, &claims);

        prop_assert_eq!(&delegated, &direct);

        if first_amount > limit || second_amount > limit {
            prop_assert!(
                matches!(direct, ConstraintDecision::Deny { .. }),
                "oversized token claims must deny"
            );
        } else if first_amount.saturating_add(second_amount) > limit {
            prop_assert!(
                matches!(direct, ConstraintDecision::Wait { .. }),
                "same-batch token claims over budget must wait"
            );
        } else {
            match &direct {
                ConstraintDecision::Allow { required_leases } => {
                    prop_assert_eq!(required_leases, &claims);
                }
                other => {
                    prop_assert!(
                        false,
                        "in-budget token claims should allow, got {other:?}"
                    );
                }
            }
        }
    }
}
