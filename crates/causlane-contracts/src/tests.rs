//! Unit tests for registry parsing, bundle compilation and plan hashing.

use causlane_core::{
    ClaimMode, ConsequenceProfile, LifecycleClass, PlanHash, PlanHashError, PredicateId,
};
use serde::Serialize;

use crate::bundle::CompiledDispatchBundle;
use crate::canonical::{canonical_json_bytes, canonical_json_hash};
use crate::examples::{release_promote_impacts, release_promote_plan_material};
use crate::plan_hash::impact_set_hash;
use crate::registry::{AuthzModeDto, BarrierPolicyDto, ProjectionPolicyDto, RegistryManifest};
use crate::{
    is_active_invariant_id, is_known_invariant_id, is_planned_invariant_id, ContractError,
};

const REGISTRY: &str = include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");

/// The pinned bundle-derived plan hash for the `release_promote` demo (see
/// `examples.rs`). The plan binds to the compiled bundle hash, so this value
/// shifts whenever the bundle changes (e.g. when `effect_templates` / merge
/// protocols are compiled in — P0-FM-009). The hand-authored trace/scenario
/// fixtures carry their own internally-consistent plan hash and do not need to
/// equal this value (replay checks plan-hash consistency, not bundle identity).
const DEMO_PLAN_HASH: &str =
    "sha256:01672ea100b6924fca6beff764f25895d900bc052af735129096b43ee3c054f9";

fn demo_bundle() -> Result<CompiledDispatchBundle, ContractError> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
    CompiledDispatchBundle::compile(&manifest)
}

#[test]
fn bundle_id_comes_from_manifest() -> Result<(), ContractError> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
    let from_manifest = CompiledDispatchBundle::compile(&manifest)?;
    assert_eq!(
        from_manifest.body.bundle_id,
        "causlane.demo.release_promote"
    );
    // Explicit override is tests/dev-only and changes the bundle hash.
    let overridden = CompiledDispatchBundle::compile_with_bundle_id(&manifest, "other.id")?;
    assert_ne!(from_manifest.bundle_hash, overridden.bundle_hash);
    Ok(())
}

#[test]
fn parses_example_registry() -> Result<(), ContractError> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
    assert_eq!(manifest.predicates.len(), 1);

    let Some(predicate) = manifest.predicates.first() else {
        return Err(ContractError::Yaml("expected one predicate".to_owned()));
    };
    assert_eq!(predicate.id, "release.promote_candidate");
    assert_eq!(
        predicate.consequence_profile.to_core(),
        ConsequenceProfile::RuntimeExecution
    );
    assert_eq!(
        predicate.lifecycle_class.to_core(),
        LifecycleClass::ExecutionBearing
    );
    assert_eq!(predicate.route_id, "route.runtime_execution");
    assert_eq!(predicate.barrier_policy, BarrierPolicyDto::StrictWriteAhead);
    assert_eq!(predicate.projection_policy, ProjectionPolicyDto::Anchored);
    assert_eq!(predicate.formal_obligations.len(), 5);
    assert_eq!(predicate.claims.len(), 2);

    let Some(claim) = predicate.claims.first() else {
        return Err(ContractError::Yaml("expected a claim".to_owned()));
    };
    assert_eq!(claim.mode.to_core(), ClaimMode::ExclusiveWrite);

    let Some(witness) = predicate.required_witnesses.first() else {
        return Err(ContractError::Yaml("expected a witness".to_owned()));
    };
    assert_eq!(witness.id, "readiness_before_promotion");
    assert_eq!(witness.selector.fact_kind, "readiness_ok");
    Ok(())
}

#[test]
fn invariant_catalog_reserves_planned_ids_without_activating_bundle_obligations(
) -> Result<(), ContractError> {
    assert!(is_active_invariant_id("I-010"));
    assert!(is_planned_invariant_id("I-011"));
    assert!(is_known_invariant_id("I-011"));

    let yaml = REGISTRY.replace("      - I-008", "      - I-011");
    let manifest = RegistryManifest::from_yaml_str(&yaml)?;
    let result = CompiledDispatchBundle::compile(&manifest);

    assert!(matches!(result, Err(ContractError::Validation(_))));
    Ok(())
}

#[test]
fn bundle_artifact_roundtrips_and_detects_stale_hash() -> Result<(), ContractError> {
    let bundle = demo_bundle()?;
    let json = bundle.to_json_pretty()?;
    let reparsed = CompiledDispatchBundle::from_json_str(&json)?;
    assert_eq!(bundle.bundle_hash, reparsed.bundle_hash);
    assert_eq!(bundle.body, reparsed.body);

    let stale_hash = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    let tampered = json.replace(&bundle.bundle_hash.0, stale_hash);
    assert!(matches!(
        CompiledDispatchBundle::from_json_str(&tampered),
        Err(ContractError::Validation(_))
    ));
    Ok(())
}

#[test]
fn canonical_serialization_v1_is_compact_and_ordered() -> Result<(), ContractError> {
    #[derive(Serialize)]
    struct GoldenMaterial {
        schema_version: u32,
        action_id: &'static str,
        plan_hash: &'static str,
    }

    let material = GoldenMaterial {
        schema_version: 1,
        action_id: "act",
        plan_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    };
    let bytes = canonical_json_bytes(&material)?;
    assert_eq!(
        bytes,
        br#"{"schema_version":1,"action_id":"act","plan_hash":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#
    );
    assert_eq!(
        canonical_json_hash(&material)?,
        "sha256:165c46a45efbfde04e47318573ab975b319c15dd6c1c8cf12ef2ac308865fd19"
    );
    Ok(())
}

#[test]
fn bundle_parser_rejects_old_schema_version() -> Result<(), ContractError> {
    let bundle = demo_bundle()?;
    let json = bundle.to_json_pretty()?;
    let old_schema = json.replace(
        "\"bundle_schema_version\": 3",
        "\"bundle_schema_version\": 2",
    );
    assert!(matches!(
        CompiledDispatchBundle::from_json_str(&old_schema),
        Err(ContractError::Validation(_))
    ));
    Ok(())
}

#[test]
fn bundle_hash_is_well_formed_and_deterministic() -> Result<(), ContractError> {
    let first = demo_bundle()?;
    let second = demo_bundle()?;
    assert_eq!(first.bundle_hash, second.bundle_hash);

    let digest = first
        .bundle_hash
        .0
        .strip_prefix("sha256:")
        .unwrap_or_default();
    assert_eq!(digest.len(), 64);

    let lookup = first.predicate(&PredicateId("release.promote_candidate".to_owned()));
    assert!(lookup.is_some());
    Ok(())
}

#[test]
fn compile_rejects_lifecycle_profile_mismatch() -> Result<(), ContractError> {
    let yaml = REGISTRY.replace(
        "lifecycle_class: execution_bearing",
        "lifecycle_class: projection_only",
    );
    let manifest = RegistryManifest::from_yaml_str(&yaml)?;
    let result = CompiledDispatchBundle::compile(&manifest);
    assert!(matches!(result, Err(ContractError::Validation(_))));
    Ok(())
}

// P0-010: an authz policy of mode=required must name the policy decisions are
// expected to be issued under, so replay/runtime can reject a decision carried
// under the wrong policy. A required policy without policy_id is rejected.
#[test]
fn compile_rejects_required_authz_without_policy_id() -> Result<(), ContractError> {
    let yaml = REGISTRY.replace(
        "mode: disabled_for_local_dev\n      allowed_in_profiles: [RuntimeExecution]\n      rationale: demo fixture without real PDP",
        "mode: required\n      stages: [execution_barrier_logged]",
    );
    let manifest = RegistryManifest::from_yaml_str(&yaml)?;
    let result = CompiledDispatchBundle::compile(&manifest);
    assert!(matches!(result, Err(ContractError::Validation(_))));
    Ok(())
}

// P0-010 defense-in-depth: a bundle whose hash is valid but whose required-authz
// predicate declares no policy must STILL be rejected on the deserialization
// path (from_json_str), since compile-time validation does not run there and an
// empty expected policy would silently disable the downstream policy check.
#[test]
fn from_json_str_rejects_required_authz_without_policy() -> Result<(), ContractError> {
    let mut bundle = demo_bundle()?;
    {
        let predicate = bundle
            .body
            .predicates
            .first_mut()
            .ok_or_else(|| ContractError::Validation("no predicates".to_owned()))?;
        predicate.authz_policy.mode = AuthzModeDto::Required;
        predicate.authz_policy.stages = vec!["execution_barrier_logged".to_owned()];
        predicate.authz_policy.policy_id = String::new();
        predicate.authz_policy.policy_version = String::new();
    }
    // Recompute the canonical hash so the artifact is hash-valid; only the authz
    // invariant is violated.
    let hash = canonical_json_hash(&bundle.body)?;
    let json = serde_json::json!({ "bundle_hash": hash, "body": bundle.body }).to_string();
    let result = CompiledDispatchBundle::from_json_str(&json);
    assert!(matches!(result, Err(ContractError::Validation(_))));
    Ok(())
}

#[test]
fn compile_rejects_runtime_execution_without_claims() -> Result<(), ContractError> {
    let yaml = REGISTRY.replace(
        "    claims:\n      - resource: environment_write\n        scope_expr: environment:${subject.target_environment}\n        mode: exclusive\n      - resource: release_candidate_write\n        scope_expr: release_candidate:${subject.release_candidate_id}\n        mode: exclusive\n",
        "    claims: []\n",
    );
    let manifest = RegistryManifest::from_yaml_str(&yaml)?;
    let result = CompiledDispatchBundle::compile(&manifest);
    assert!(matches!(result, Err(ContractError::Validation(_))));
    Ok(())
}

#[test]
fn plan_hash_is_valid_and_deterministic() -> Result<(), ContractError> {
    let bundle = demo_bundle()?;
    let material = release_promote_plan_material(bundle.bundle_hash.0.as_str());
    let first = material.compute_plan_hash()?;
    let second = material.compute_plan_hash()?;
    assert_eq!(first, second);
    assert!(first.as_str().starts_with("sha256:"));
    Ok(())
}

#[test]
fn plan_hash_pins_to_demo_value() -> Result<(), ContractError> {
    let bundle = demo_bundle()?;
    let material = release_promote_plan_material(bundle.bundle_hash.0.as_str());
    let plan_hash = material.compute_plan_hash()?;
    assert_eq!(plan_hash.as_str(), DEMO_PLAN_HASH);
    Ok(())
}

#[test]
fn plan_hash_is_sensitive_to_ops() -> Result<(), ContractError> {
    let bundle = demo_bundle()?;
    let base = release_promote_plan_material(bundle.bundle_hash.0.as_str());
    let baseline = base.compute_plan_hash()?;

    let mut mutated = base.clone();
    if let Some(op) = mutated.ops.first_mut() {
        op.kind = "tampered_op_kind".to_owned();
    }
    let changed = mutated.compute_plan_hash()?;
    assert_ne!(baseline, changed);
    Ok(())
}

#[test]
fn impact_set_hash_is_deterministic() -> Result<(), ContractError> {
    let impacts = release_promote_impacts();
    let first = impact_set_hash(&impacts)?;
    let second = impact_set_hash(&impacts)?;
    assert_eq!(first, second);
    assert!(first.0.starts_with("sha256:"));
    Ok(())
}

#[test]
fn rejects_todo_placeholder_plan_hash() {
    let parsed = PlanHash::new("sha256:TODO");
    assert_eq!(parsed, Err(PlanHashError::Placeholder));
}

#[test]
fn compile_populates_effect_templates_and_merge_protocols() -> Result<(), ContractError> {
    let bundle = demo_bundle()?;
    let predicate = bundle
        .body
        .predicates
        .first()
        .ok_or_else(|| ContractError::Validation("no predicate".to_owned()))?;
    // P0-FM-009: effect templates are compiled in, not dropped to Vec::new().
    assert!(!predicate.effect_templates.is_empty());
    assert!(predicate.effect_templates.iter().any(|effect| {
        effect.op_kind == "promote_release"
            && !effect.conflict_domains.is_empty()
            && effect.hardness == crate::registry::EffectHardnessDto::Hard
            && effect.idempotency_domain.is_some()
    }));
    // Merge protocols are compiled into the bundle; only Verified permits concurrency.
    assert!(!bundle.body.merge_protocols.is_empty());
    assert!(bundle
        .body
        .merge_protocols
        .iter()
        .any(|protocol| protocol.status.permits_concurrency()));
    assert!(!crate::MergeProtocolStatus::DeclaredButUnverified.permits_concurrency());
    assert!(!crate::MergeProtocolStatus::Absent.permits_concurrency());
    assert!(!crate::MergeProtocolStatus::Disabled.permits_concurrency());
    Ok(())
}

#[test]
fn merge_decision_is_fail_closed_without_verified_applicability() -> Result<(), ContractError> {
    let bundle = demo_bundle()?;
    let predicate = bundle
        .body
        .predicates
        .first()
        .ok_or_else(|| ContractError::Validation("no predicate".to_owned()))?;
    // The demo declares a verified protocol but links no applicability to
    // promote_release, so overlapping mutable writes remain non-mergeable.
    assert_eq!(
        crate::merge_decision(
            &bundle.body.merge_protocols,
            &predicate.merge_protocol_applicability,
            "promote_release",
        ),
        crate::MergeDecision::NotMergeable
    );
    // With an applicability linking the verified protocol, it becomes mergeable.
    let applicability = vec![crate::MergeProtocolApplicabilityManifest {
        protocol_id: "append_only_release_log_v1".to_owned(),
        applies_to: "promote_release".to_owned(),
    }];
    assert!(matches!(
        crate::merge_decision(
            &bundle.body.merge_protocols,
            &applicability,
            "promote_release"
        ),
        crate::MergeDecision::Mergeable { .. }
    ));
    Ok(())
}
