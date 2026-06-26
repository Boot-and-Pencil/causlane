//! Public replay error diagnostics: stable code tokens, invariant mappings, and
//! causal location extraction for every current `ReplayError` variant.

use causlane_replay::{CausalLocation, ReplayError, ReplayErrorCode, ReplayExplain, ReplayVerdict};

struct ErrorCase {
    name: &'static str,
    error: ReplayError,
    code: ReplayErrorCode,
    token: &'static str,
    invariant: Option<&'static str>,
    location: CausalLocation,
}

macro_rules! error_case {
    ($name:literal, $error:expr, $code:ident, $token:literal, $invariant:expr, $location:expr $(,)?) => {
        ErrorCase {
            name: $name,
            error: $error,
            code: ReplayErrorCode::$code,
            token: $token,
            invariant: $invariant,
            location: $location,
        }
    };
}

fn text(value: &str) -> String {
    value.to_owned()
}

fn empty() -> CausalLocation {
    CausalLocation::default()
}

fn action(action_id: &str) -> CausalLocation {
    CausalLocation {
        action_id: Some(text(action_id)),
        ..CausalLocation::default()
    }
}

fn event(event_id: &str) -> CausalLocation {
    CausalLocation {
        event_id: Some(text(event_id)),
        ..CausalLocation::default()
    }
}

fn action_plan(action_id: &str, plan_hash: &str) -> CausalLocation {
    CausalLocation {
        action_id: Some(text(action_id)),
        plan_hash: Some(text(plan_hash)),
        ..CausalLocation::default()
    }
}

fn event_anchor(event_id: &str, anchor_event_id: &str) -> CausalLocation {
    CausalLocation {
        event_id: Some(text(event_id)),
        anchor_event_id: Some(text(anchor_event_id)),
        ..CausalLocation::default()
    }
}

fn scope(scope: &str) -> CausalLocation {
    CausalLocation {
        scope: Some(text(scope)),
        ..CausalLocation::default()
    }
}

fn barrier_witness(barrier_event_id: &str, witness_event_id: &str) -> CausalLocation {
    CausalLocation {
        barrier_event_id: Some(text(barrier_event_id)),
        witness_event_id: Some(text(witness_event_id)),
        ..CausalLocation::default()
    }
}

fn barrier(barrier_event_id: &str) -> CausalLocation {
    CausalLocation {
        barrier_event_id: Some(text(barrier_event_id)),
        ..CausalLocation::default()
    }
}

fn requirement(requirement_id: &str) -> CausalLocation {
    CausalLocation {
        requirement_id: Some(text(requirement_id)),
        ..CausalLocation::default()
    }
}

fn stage(stage: &str) -> CausalLocation {
    CausalLocation {
        stage: Some(text(stage)),
        ..CausalLocation::default()
    }
}

fn action_event(action_id: &str, event_id: &str) -> CausalLocation {
    CausalLocation {
        action_id: Some(text(action_id)),
        event_id: Some(text(event_id)),
        ..CausalLocation::default()
    }
}

fn event_scope(event_id: &str, scope: &str) -> CausalLocation {
    CausalLocation {
        event_id: Some(text(event_id)),
        scope: Some(text(scope)),
        ..CausalLocation::default()
    }
}

fn loading_and_provenance_cases() -> Vec<ErrorCase> {
    vec![
        error_case!(
            "Decode",
            ReplayError::Decode(text("invalid json")),
            Decode,
            "Decode",
            None,
            empty(),
        ),
        error_case!(
            "BadPlanHash",
            ReplayError::BadPlanHash(text("bad plan")),
            BadPlanHash,
            "BadPlanHash",
            None,
            empty(),
        ),
        error_case!(
            "BadImpactSetHash",
            ReplayError::BadImpactSetHash {
                field: text("impact_set_hash"),
            },
            BadImpactSetHash,
            "BadImpactSetHash",
            None,
            empty(),
        ),
        error_case!(
            "MissingTracePredicate",
            ReplayError::MissingTracePredicate,
            MissingTracePredicate,
            "MissingTracePredicate",
            None,
            empty(),
        ),
        error_case!(
            "UnknownPredicate",
            ReplayError::UnknownPredicate {
                predicate: text("missing.predicate"),
            },
            UnknownPredicate,
            "UnknownPredicate",
            None,
            empty(),
        ),
        error_case!(
            "BundleHashMismatch",
            ReplayError::BundleHashMismatch {
                expected: text("sha256:expected"),
                actual: text("sha256:actual"),
            },
            BundleHashMismatch,
            "BundleHashMismatch",
            None,
            empty(),
        ),
        error_case!(
            "MissingTraceBundleHash",
            ReplayError::MissingTraceBundleHash {
                expected: text("sha256:expected"),
            },
            MissingTraceBundleHash,
            "MissingTraceBundleHash",
            None,
            empty(),
        ),
    ]
}

fn ordering_and_anchor_cases() -> Vec<ErrorCase> {
    vec![
        error_case!(
            "ExecutionWithoutBarrier",
            ReplayError::ExecutionWithoutBarrier {
                action_id: text("act"),
                plan_hash: text("sha256:1111"),
            },
            ExecutionWithoutBarrier,
            "ExecutionWithoutBarrier",
            Some("I-001"),
            action_plan("act", "sha256:1111"),
        ),
        error_case!(
            "ObservedWithoutExecution",
            ReplayError::ObservedWithoutExecution {
                action_id: text("act"),
                plan_hash: text("sha256:2222"),
            },
            ObservedWithoutExecution,
            "ObservedWithoutExecution",
            Some("I-002"),
            action_plan("act", "sha256:2222"),
        ),
        error_case!(
            "ProjectionWithoutAnchor",
            ReplayError::ProjectionWithoutAnchor {
                event_id: text("projection_evt"),
            },
            ProjectionWithoutAnchor,
            "ProjectionWithoutAnchor",
            Some("I-003"),
            event("projection_evt"),
        ),
        error_case!(
            "UnresolvedAnchorPlanHash",
            ReplayError::UnresolvedAnchorPlanHash {
                event_id: text("projection_evt"),
            },
            UnresolvedAnchorPlanHash,
            "UnresolvedAnchorPlanHash",
            Some("I-003"),
            event("projection_evt"),
        ),
        error_case!(
            "AnchorNotObservedTruth",
            ReplayError::AnchorNotObservedTruth {
                event_id: text("projection_evt"),
                anchor_event_id: text("anchor_evt"),
            },
            AnchorNotObservedTruth,
            "AnchorNotObservedTruth",
            Some("I-003"),
            event_anchor("projection_evt", "anchor_evt"),
        ),
        error_case!(
            "AnchorAttestationMismatch",
            ReplayError::AnchorAttestationMismatch {
                event_id: text("projection_evt"),
                anchor_event_id: text("anchor_evt"),
            },
            AnchorAttestationMismatch,
            "AnchorAttestationMismatch",
            Some("I-003"),
            event_anchor("projection_evt", "anchor_evt"),
        ),
    ]
}

fn barrier_capability_and_lease_cases() -> Vec<ErrorCase> {
    vec![
        error_case!(
            "MissingRequiredBarrier",
            ReplayError::MissingRequiredBarrier {
                action_id: text("act"),
            },
            MissingRequiredBarrier,
            "MissingRequiredBarrier",
            Some("I-001"),
            action("act"),
        ),
        error_case!(
            "MissingDispatchBeforeBarrier",
            ReplayError::MissingDispatchBeforeBarrier {
                action_id: text("act"),
            },
            MissingDispatchBeforeBarrier,
            "MissingDispatchBeforeBarrier",
            Some("I-001"),
            action("act"),
        ),
        error_case!(
            "MissingBarrierPayload",
            ReplayError::MissingBarrierPayload {
                event_id: text("barrier_evt"),
            },
            MissingBarrierPayload,
            "MissingBarrierPayload",
            Some("I-001"),
            event("barrier_evt"),
        ),
        error_case!(
            "MissingBarrierImpactSet",
            ReplayError::MissingBarrierImpactSet {
                event_id: text("barrier_evt"),
            },
            MissingBarrierImpactSet,
            "MissingBarrierImpactSet",
            Some("I-001"),
            event("barrier_evt"),
        ),
        error_case!(
            "CapabilityMissing",
            ReplayError::CapabilityMissing {
                event_id: text("execution_evt"),
            },
            CapabilityMissing,
            "CapabilityMissing",
            Some("I-001"),
            event("execution_evt"),
        ),
        error_case!(
            "CapabilityMismatch",
            ReplayError::CapabilityMismatch {
                event_id: text("execution_evt"),
                error: text("wrong capability"),
            },
            CapabilityMismatch,
            "CapabilityMismatch",
            Some("I-001"),
            event("execution_evt"),
        ),
        error_case!(
            "ConflictingLeases",
            ReplayError::ConflictingLeases {
                scope: text("environment:staging"),
            },
            ConflictingLeases,
            "ConflictingLeases",
            Some("I-006"),
            scope("environment:staging"),
        ),
        error_case!(
            "DrainFenceWithActiveOverlap",
            ReplayError::DrainFenceWithActiveOverlap {
                event_id: text("drain_evt"),
                scope: text("environment:staging"),
            },
            DrainFenceWithActiveOverlap,
            "DrainFenceWithActiveOverlap",
            Some("I-007"),
            event_scope("drain_evt", "environment:staging"),
        ),
        error_case!(
            "Lease",
            ReplayError::Lease(text("lease expired")),
            Lease,
            "Lease",
            None,
            empty(),
        ),
    ]
}

fn witness_and_authz_cases() -> Vec<ErrorCase> {
    vec![
        error_case!(
            "WitnessNotPrior",
            ReplayError::WitnessNotPrior {
                barrier_event_id: text("barrier_evt"),
                witness_event_id: text("witness_evt"),
            },
            WitnessNotPrior,
            "WitnessNotPrior",
            Some("I-009"),
            barrier_witness("barrier_evt", "witness_evt"),
        ),
        error_case!(
            "RequiredWitnessMissing",
            ReplayError::RequiredWitnessMissing {
                requirement_id: text("approval_required"),
            },
            RequiredWitnessMissing,
            "RequiredWitnessMissing",
            Some("I-009"),
            requirement("approval_required"),
        ),
        error_case!(
            "LegacyWitnessMismatch",
            ReplayError::LegacyWitnessMismatch {
                barrier_event_id: text("barrier_evt"),
            },
            LegacyWitnessMismatch,
            "LegacyWitnessMismatch",
            Some("I-009"),
            barrier("barrier_evt"),
        ),
        error_case!(
            "WitnessSelectorMismatch",
            ReplayError::WitnessSelectorMismatch {
                requirement_id: text("approval_required"),
            },
            WitnessSelectorMismatch,
            "WitnessSelectorMismatch",
            Some("I-009"),
            requirement("approval_required"),
        ),
        error_case!(
            "WitnessBindingMismatch",
            ReplayError::WitnessBindingMismatch {
                requirement_id: text("approval_required"),
            },
            WitnessBindingMismatch,
            "WitnessBindingMismatch",
            Some("I-009"),
            requirement("approval_required"),
        ),
        error_case!(
            "WitnessAttestationMismatch",
            ReplayError::WitnessAttestationMismatch {
                requirement_id: text("approval_required"),
            },
            WitnessAttestationMismatch,
            "WitnessAttestationMismatch",
            Some("I-009"),
            requirement("approval_required"),
        ),
        error_case!(
            "AuthzDecisionMissing",
            ReplayError::AuthzDecisionMissing {
                stage: text("execution_barrier_logged"),
            },
            AuthzDecisionMissing,
            "AuthzDecisionMissing",
            Some("I-009"),
            stage("execution_barrier_logged"),
        ),
        error_case!(
            "AuthzDecisionDenied",
            ReplayError::AuthzDecisionDenied {
                stage: text("execution_barrier_logged"),
            },
            AuthzDecisionDenied,
            "AuthzDecisionDenied",
            Some("I-009"),
            stage("execution_barrier_logged"),
        ),
    ]
}

fn lifecycle_and_authz_freshness_cases() -> Vec<ErrorCase> {
    vec![
        error_case!(
            "PlanHashMismatch",
            ReplayError::PlanHashMismatch {
                action_id: text("act"),
            },
            PlanHashMismatch,
            "PlanHashMismatch",
            None,
            action("act"),
        ),
        error_case!(
            "TemplateResolution",
            ReplayError::TemplateResolution {
                expression: text("{{ subject }}"),
                error: text("missing binding"),
            },
            TemplateResolution,
            "TemplateResolution",
            None,
            empty(),
        ),
        error_case!(
            "Lifecycle",
            ReplayError::Lifecycle(text("bad transition")),
            Lifecycle,
            "Lifecycle",
            None,
            empty(),
        ),
        error_case!(
            "EventAfterClosed",
            ReplayError::EventAfterClosed {
                action_id: text("act"),
                event_id: text("late_evt"),
            },
            EventAfterClosed,
            "EventAfterClosed",
            Some("I-008"),
            action_event("act", "late_evt"),
        ),
        error_case!(
            "AuthzDecisionExpired",
            ReplayError::AuthzDecisionExpired {
                stage: text("execution_barrier_logged"),
            },
            AuthzDecisionExpired,
            "AuthzDecisionExpired",
            Some("I-009"),
            stage("execution_barrier_logged"),
        ),
        error_case!(
            "AuthzPolicyMismatch",
            ReplayError::AuthzPolicyMismatch {
                stage: text("execution_barrier_logged"),
            },
            AuthzPolicyMismatch,
            "AuthzPolicyMismatch",
            Some("I-009"),
            stage("execution_barrier_logged"),
        ),
        error_case!(
            "AuthzIssuedAfterBarrier",
            ReplayError::AuthzIssuedAfterBarrier {
                stage: text("execution_barrier_logged"),
            },
            AuthzIssuedAfterBarrier,
            "AuthzIssuedAfterBarrier",
            Some("I-009"),
            stage("execution_barrier_logged"),
        ),
        error_case!(
            "AuthzDecisionStale",
            ReplayError::AuthzDecisionStale {
                stage: text("execution_barrier_logged"),
            },
            AuthzDecisionStale,
            "AuthzDecisionStale",
            Some("I-009"),
            stage("execution_barrier_logged"),
        ),
    ]
}

fn cases() -> Vec<ErrorCase> {
    let mut cases = Vec::new();
    cases.extend(loading_and_provenance_cases());
    cases.extend(ordering_and_anchor_cases());
    cases.extend(barrier_capability_and_lease_cases());
    cases.extend(witness_and_authz_cases());
    cases.extend(lifecycle_and_authz_freshness_cases());
    cases
}

#[test]
fn replay_error_diagnostics_are_stable_for_every_variant() {
    for case in cases() {
        assert_eq!(case.error.code(), case.code, "{} code", case.name);
        assert_eq!(case.code.as_str(), case.token, "{} code token", case.name);
        assert_eq!(
            case.error.code_token(),
            case.token,
            "{} error token",
            case.name
        );
        assert_eq!(
            case.error.invariant(),
            case.invariant,
            "{} invariant",
            case.name
        );
        assert_eq!(
            CausalLocation::from_error(&case.error),
            case.location,
            "{} causal location",
            case.name
        );
    }
}

#[test]
fn replay_diagnostics_types_remain_public() {
    let verdict = ReplayVerdict {
        accepted: true,
        stable_error_code: None,
        error_detail: None,
        checked_invariants: Vec::new(),
        bundle_hash: text("sha256:bundle"),
        trace_hash: text("sha256:trace"),
    };
    let explain = ReplayExplain {
        accepted: true,
        invariant: None,
        error_code: None,
        error_detail: None,
        causal_location: CausalLocation::default(),
        checked_invariants: verdict.checked_invariants.clone(),
        bundle_hash: verdict.bundle_hash.clone(),
        trace_hash: verdict.trace_hash.clone(),
    };

    assert!(verdict.accepted);
    assert!(explain.accepted);
    assert_eq!(ReplayErrorCode::Decode.as_str(), "Decode");
}
