//! Worked examples shared by tests, fixtures and docs (seed of the scenario
//! catalog, TZ-011). Currently: the `release.promote_candidate` demo.

use crate::plan_hash::{
    content_hash, CanonicalClaim, CanonicalEffect, CanonicalImpact, CanonicalOp, PlanHashMaterial,
};

/// Canonical plan-hash material for the `release.promote_candidate` demo.
///
/// This is the single source of truth for how the demo `plan_hash` is *computed*:
/// `release_promote_plan_material(bundle_hash).compute_plan_hash()` binds the plan
/// to the compiled bundle hash, so the value shifts whenever the bundle changes.
/// The hand-authored fixtures in `contracts/examples/release_promote.trace.json`
/// and `contracts/scenarios/*` carry their own internally-consistent plan hash and
/// are NOT required to equal this value — replay checks plan-hash consistency within
/// a trace, not bundle identity (see the `DEMO_PLAN_HASH` note in `tests.rs`).
#[must_use]
pub fn release_promote_plan_material(bundle_hash: &str) -> PlanHashMaterial {
    let subject_fingerprint =
        content_hash(br#"{"release_candidate_id":"rc_123","target_environment":"staging"}"#).0;
    let circumstance_fingerprint =
        content_hash(br#"{"requested_by":"alice","reason":"ship it"}"#).0;

    PlanHashMaterial {
        hash_schema_version: 1,
        bundle_id: "causlane.demo.release_promote".to_owned(),
        bundle_version: "0.0.0".to_owned(),
        bundle_hash: bundle_hash.to_owned(),
        planner_id: "causlane.demo.planner".to_owned(),
        planner_version: "0.0.0".to_owned(),
        planner_fingerprint: "demo".to_owned(),
        action_id: "act_promote_123".to_owned(),
        predicate: "release.promote_candidate".to_owned(),
        predicate_version: 1,
        subject_fingerprint,
        circumstance_fingerprint,
        consequence_profile: "RuntimeExecution".to_owned(),
        lifecycle_class: "execution_bearing".to_owned(),
        route_id: "route.runtime_execution".to_owned(),
        ops: vec![CanonicalOp {
            index: 0,
            kind: "promote_release_candidate".to_owned(),
            effect: CanonicalEffect {
                reads: Vec::new(),
                writes: vec![
                    "environment:staging".to_owned(),
                    "release_candidate:rc_123".to_owned(),
                ],
                produces: vec!["release_candidate_promoted".to_owned()],
                requires: vec!["readiness_ok".to_owned()],
                invalidates: Vec::new(),
                conflict_domains: vec!["environment:staging".to_owned()],
                hardness: "hard".to_owned(),
            },
        }],
        planned_impacts: release_promote_impacts(),
        required_witnesses: vec!["readiness_before_promotion".to_owned()],
        required_claims: vec![
            CanonicalClaim {
                resource: "environment_write".to_owned(),
                scope: "environment:staging".to_owned(),
                mode: "exclusive".to_owned(),
                amount: 1,
            },
            CanonicalClaim {
                resource: "release_candidate_write".to_owned(),
                scope: "release_candidate:rc_123".to_owned(),
                mode: "exclusive".to_owned(),
                amount: 1,
            },
        ],
        barrier_policy: "strict_write_ahead".to_owned(),
        projection_policy: "anchored".to_owned(),
    }
}

/// The planned impacts of the `release.promote_candidate` demo.
#[must_use]
pub fn release_promote_impacts() -> Vec<CanonicalImpact> {
    vec![
        CanonicalImpact {
            scope: "environment:staging".to_owned(),
            hardness: "hard".to_owned(),
        },
        CanonicalImpact {
            scope: "release_candidate:rc_123".to_owned(),
            hardness: "hard".to_owned(),
        },
    ]
}
