#![forbid(unsafe_code)]
#![deny(warnings)]

use std::fmt;

use causlane_contracts::{
    content_hash, impact_set_hash, is_canonical_sha256_token, BoundaryContracts, BundleCompiler,
    BundleValidator, CanonicalClaim, CanonicalEffect, CanonicalImpact, CanonicalOp,
    CompiledDispatchBundle, ContractError, PlanHashMaterial, PlanTemplateCache,
    PlanTemplateCacheKey, PlanTemplateSnapshotRef, RegistryManifest, TemplateBindings,
    TemplateError, TemplateResolver,
};

const REGISTRY_YAML: &str = r#"
bundle_id: causlane.example.release_governance
registry_version: 0.0.0
predicates:
  - id: release.approve_window
    version: 1
    consequence_profile: OversightMeta
    subject_schema_ref: schemas/release_window_subject.schema.json
    circumstance_schema_ref: schemas/release_window_circumstance.schema.json
    lifecycle_class: meta
    route_id: route.oversight
    route_derivation: consequence_profile
    barrier_policy: none
    projection_policy: none
    authz_policy:
      mode: required
      stages: [gate_approved]
      policy_id: release-governance-pdp
      policy_version: "2026-06-29"
      freshness_max_age: 300
    truth_commit_policy: disallowed
    schema_hashes:
      subject: sha256:1111111111111111111111111111111111111111111111111111111111111111
      circumstance: sha256:2222222222222222222222222222222222222222222222222222222222222222
    constraint_policy:
      claim_coverage: disabled
      lease_conflicts: disabled
    formal_obligations: [I-001, I-002]
    effect_templates:
      - op_kind: approve_release_window
        reads:
          - release_candidate:${subject.release_candidate_id}
        writes: []
        produces:
          - release.window_approved
        requires: []
        conflict_domains: []
        hardness: meta
        idempotency_domain: release_window:${subject.release_candidate_id}:${subject.target_environment}
    scenario_refs:
      - contracts/scenarios/release_window_approved.scenario.yaml
  - id: release.promote_candidate
    version: 1
    consequence_profile: RuntimeExecution
    subject_schema_ref: schemas/release_promote_subject.schema.json
    circumstance_schema_ref: schemas/release_promote_circumstance.schema.json
    lifecycle_class: execution_bearing
    route_id: route.runtime_execution
    route_derivation: consequence_profile
    barrier_policy: strict_write_ahead
    projection_policy: anchored
    authz_policy:
      mode: required
      stages: [execution_barrier_logged, may_project]
      policy_id: release-governance-pdp
      policy_version: "2026-06-29"
      freshness_max_age: 120
    truth_commit_policy: allowed
    schema_hashes:
      subject: sha256:3333333333333333333333333333333333333333333333333333333333333333
      circumstance: sha256:4444444444444444444444444444444444444444444444444444444444444444
    constraint_policy:
      claim_coverage: required
      lease_conflicts: required
    formal_obligations: [I-001, I-002, I-003, I-006, I-008]
    required_witnesses:
      - id: window_approved_before_promotion
        target_stage: execution_barrier_logged
        selector:
          event_kind: gate.approved
          predicate: release.approve_window
          fact_kind: release.window_approved
          scope_expr: release_candidate:${subject.release_candidate_id}
    claims:
      - resource: environment_write
        scope_expr: environment:${subject.target_environment}
        mode: exclusive
      - resource: release_candidate_write
        scope_expr: release_candidate:${subject.release_candidate_id}
        mode: exclusive
    effect_templates:
      - op_kind: promote_release_candidate
        reads:
          - release_candidate:${subject.release_candidate_id}
          - environment:${subject.target_environment}
        writes:
          - environment:${subject.target_environment}
        produces:
          - release.promoted
        requires:
          - release.window_approved
        conflict_domains:
          - environment:${subject.target_environment}
          - release_candidate:${subject.release_candidate_id}
        hardness: hard
        idempotency_domain: promote:${subject.release_candidate_id}:${subject.target_environment}
    scenario_refs:
      - contracts/scenarios/release_promote_success.scenario.yaml
      - contracts/scenarios/window_missing_invalid.scenario.yaml
merge_protocols:
  - id: append_only_release_log_v1
    version: 1
    status: verified
    algebra: append_only_disjoint_keys
"#;

const SNAPSHOT_HASH: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_SNAPSHOT_HASH: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

/// Summary returned by the contracts registry/bundle workflow example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractsRegistryBundleWorkflowSummary {
    /// Registry manifests parsed from YAML.
    pub parsed_manifests: usize,
    /// Predicates compiled into the bundle.
    pub compiled_predicates: usize,
    /// Bundle artifact reloads verified through public APIs.
    pub verified_artifacts: usize,
    /// Canonical hash tokens inspected.
    pub canonical_hashes: usize,
    /// Template expressions resolved through public APIs.
    pub resolved_templates: usize,
    /// Plan-template cache lookups exercised.
    pub plan_cache_lookups: usize,
    /// Deterministic negative controls that failed closed as expected.
    pub negative_controls: usize,
}

/// Error type for the contracts registry/bundle workflow example.
#[derive(Debug)]
pub enum ContractsRegistryBundleWorkflowError {
    /// Registry, bundle or hash contract failed.
    Contract(ContractError),
    /// Template validation or resolution failed.
    Template(TemplateError),
    /// Expected predicate was absent from the compiled bundle.
    MissingPredicate(&'static str),
    /// A hash-critical value was not a canonical `sha256:` token.
    NonCanonicalHash(&'static str),
    /// A bundle artifact accepted after tampering.
    TamperedBundleAccepted,
    /// A template expression accepted without a required binding.
    MissingTemplateAccepted,
    /// A registry missing required authz policy data compiled successfully.
    InvalidAuthzPolicyAccepted,
    /// Mutating plan material did not change the plan hash.
    PlanHashMutationNotDetected,
    /// Plan-template cache behavior diverged from expected miss/hit behavior.
    PlanCacheMismatch,
    /// A deterministic check observed a different outcome from the one expected.
    UnexpectedOutcome {
        /// Check being evaluated.
        check: &'static str,
        /// Debug rendering of the unexpected value.
        actual: String,
    },
}

impl fmt::Display for ContractsRegistryBundleWorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contract(error) => write!(f, "contract boundary failed: {error}"),
            Self::Template(error) => write!(f, "template boundary failed: {error}"),
            Self::MissingPredicate(predicate) => write!(f, "missing predicate {predicate}"),
            Self::NonCanonicalHash(label) => write!(f, "non-canonical hash token: {label}"),
            Self::TamperedBundleAccepted => f.write_str("tampered bundle artifact was accepted"),
            Self::MissingTemplateAccepted => f.write_str("missing template binding was accepted"),
            Self::InvalidAuthzPolicyAccepted => {
                f.write_str("registry missing required authz policy data was accepted")
            }
            Self::PlanHashMutationNotDetected => {
                f.write_str("mutated plan material did not change the plan hash")
            }
            Self::PlanCacheMismatch => f.write_str("plan-template cache result was unexpected"),
            Self::UnexpectedOutcome { check, actual } => {
                write!(f, "unexpected outcome for {check}: {actual}")
            }
        }
    }
}

impl std::error::Error for ContractsRegistryBundleWorkflowError {}

impl From<ContractError> for ContractsRegistryBundleWorkflowError {
    fn from(error: ContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<TemplateError> for ContractsRegistryBundleWorkflowError {
    fn from(error: TemplateError) -> Self {
        Self::Template(error)
    }
}

/// Run the public contracts registry/bundle workflow.
///
/// # Errors
/// Returns an error if public registry, bundle, template, hash or cache APIs
/// diverge from the expected deterministic, fail-closed behavior.
#[must_use = "the runnable example result must be checked"]
pub fn run_contracts_registry_bundle_workflow(
) -> Result<ContractsRegistryBundleWorkflowSummary, ContractsRegistryBundleWorkflowError> {
    let boundary = BoundaryContracts;
    let manifest = RegistryManifest::from_yaml_str(REGISTRY_YAML)?;
    boundary.validate_manifest(&manifest)?;
    let bundle = boundary.compile_registry(&manifest)?;
    verify_bundle_shape(&bundle)?;

    let verified_artifacts = verify_bundle_artifact_roundtrip(&bundle)?;
    let (canonical_hashes, plan_material) = verify_plan_and_impact_hashes(&bundle)?;
    let resolved_templates = verify_template_resolution(&boundary)?;
    let plan_cache_lookups = verify_plan_template_cache(&plan_material)?;

    let negative_controls = verify_tampered_bundle_is_rejected(&bundle)?
        + verify_missing_template_fails(&boundary)?
        + verify_required_authz_policy_fails_closed()?
        + verify_plan_material_mutation_changes_hash(&plan_material)?;

    Ok(ContractsRegistryBundleWorkflowSummary {
        parsed_manifests: 1,
        compiled_predicates: bundle.body.predicates.len(),
        verified_artifacts,
        canonical_hashes,
        resolved_templates,
        plan_cache_lookups,
        negative_controls,
    })
}

fn verify_bundle_shape(
    bundle: &CompiledDispatchBundle,
) -> Result<(), ContractsRegistryBundleWorkflowError> {
    if bundle.body.bundle_id != "causlane.example.release_governance" {
        return Err(unexpected("bundle id", &bundle.body.bundle_id));
    }
    require_hash("bundle_hash", bundle.bundle_hash.0.as_str())?;

    let approval = find_predicate(bundle, "release.approve_window")?;
    if approval.authz_policy.policy_id != "release-governance-pdp" {
        return Err(unexpected(
            "approval authz policy id",
            &approval.authz_policy.policy_id,
        ));
    }

    let promote = find_predicate(bundle, "release.promote_candidate")?;
    if promote.required_witnesses.len() != 1
        || promote.claims.len() != 2
        || promote.effect_templates.len() != 1
    {
        return Err(unexpected(
            "promote compiled shape",
            &(
                promote.required_witnesses.len(),
                promote.claims.len(),
                promote.effect_templates.len(),
            ),
        ));
    }
    Ok(())
}

/// Verify bundle JSON artifact reload through public APIs.
///
/// # Errors
/// Returns an error if serialization, parsing or hash verification fails.
#[must_use = "bundle artifacts must be verified after serialization"]
pub fn verify_bundle_artifact_roundtrip(
    bundle: &CompiledDispatchBundle,
) -> Result<usize, ContractsRegistryBundleWorkflowError> {
    let json = bundle.to_json_pretty()?;
    let reparsed = CompiledDispatchBundle::from_json_str(&json)?;
    if reparsed.bundle_hash != bundle.bundle_hash || reparsed.body != bundle.body {
        return Err(unexpected(
            "bundle artifact roundtrip",
            &(reparsed.bundle_hash.0, bundle.bundle_hash.0.as_str()),
        ));
    }
    Ok(1)
}

/// Verify plan and impact hashing through workflow material.
///
/// # Errors
/// Returns an error if hash computation fails or emits a non-canonical token.
#[must_use = "plan and impact hashes must be inspected"]
pub fn verify_plan_and_impact_hashes(
    bundle: &CompiledDispatchBundle,
) -> Result<(usize, PlanHashMaterial), ContractsRegistryBundleWorkflowError> {
    let material = release_governance_plan_material(bundle.bundle_hash.0.as_str());
    let plan_hash = material.compute_plan_hash()?;
    let impact_hash = impact_set_hash(&material.planned_impacts)?;
    require_hash("plan_hash", plan_hash.as_str())?;
    require_hash("impact_set_hash", impact_hash.0.as_str())?;
    require_hash("subject_fingerprint", material.subject_fingerprint.as_str())?;
    require_hash(
        "circumstance_fingerprint",
        material.circumstance_fingerprint.as_str(),
    )?;
    Ok((4, material))
}

/// Verify exact template resolution through [`BoundaryContracts`].
///
/// # Errors
/// Returns an error if template validation or resolution fails.
#[must_use = "resolved templates must be inspected"]
pub fn verify_template_resolution(
    boundary: &BoundaryContracts,
) -> Result<usize, ContractsRegistryBundleWorkflowError> {
    let bindings = template_bindings();
    boundary.validate_expression("environment:${subject.target_environment}")?;
    let env = boundary.resolve_string("environment:${subject.target_environment}", &bindings)?;
    if env != "environment:production" {
        return Err(unexpected("target environment template", &env));
    }
    let candidate = boundary.resolve_scope(
        "release_candidate:${subject.release_candidate_id}",
        &bindings,
    )?;
    if candidate.0 != "release_candidate:rc_456" {
        return Err(unexpected("release candidate template", &candidate.0));
    }
    let requested_by =
        boundary.resolve_string("requested_by:${circumstance.requested_by}", &bindings)?;
    if requested_by != "requested_by:bob" {
        return Err(unexpected("circumstance template", &requested_by));
    }
    Ok(3)
}

/// Verify plan-template cache miss/hit behavior through public APIs.
///
/// # Errors
/// Returns an error if cache key construction, lookup or hash computation fails.
#[must_use = "cache lookups must be inspected"]
pub fn verify_plan_template_cache(
    material: &PlanHashMaterial,
) -> Result<usize, ContractsRegistryBundleWorkflowError> {
    let snapshots = [
        PlanTemplateSnapshotRef::new("registry@0", SNAPSHOT_HASH)?,
        PlanTemplateSnapshotRef::new("schemas@0", OTHER_SNAPSHOT_HASH)?,
    ];
    let key = PlanTemplateCacheKey::new(material.clone(), snapshots)?;
    let key_hash = key.key_hash()?;
    require_hash("plan_template_cache_key", key_hash.as_str())?;

    let mut cache = PlanTemplateCache::new();
    let first = cache.lookup_or_insert(&key)?;
    let second = cache.lookup_or_insert(&key)?;
    if first.hit || !second.hit || first.entry != second.entry || cache.len() != 1 {
        return Err(ContractsRegistryBundleWorkflowError::PlanCacheMismatch);
    }
    if first.entry.plan_hash != material.compute_plan_hash()? {
        return Err(ContractsRegistryBundleWorkflowError::PlanCacheMismatch);
    }
    Ok(2)
}

/// Verify stale bundle artifacts fail closed.
///
/// # Errors
/// Returns an error if a tampered artifact is accepted.
#[must_use = "negative controls must be counted"]
pub fn verify_tampered_bundle_is_rejected(
    bundle: &CompiledDispatchBundle,
) -> Result<usize, ContractsRegistryBundleWorkflowError> {
    let json = bundle.to_json_pretty()?;
    let tampered = json.replacen(
        "\"bundle_version\": \"0.0.0\"",
        "\"bundle_version\": \"0.0.1\"",
        1,
    );
    match CompiledDispatchBundle::from_json_str(&tampered) {
        Ok(_) => Err(ContractsRegistryBundleWorkflowError::TamperedBundleAccepted),
        Err(ContractError::Validation(_)) => Ok(1),
        Err(error) => Err(ContractsRegistryBundleWorkflowError::Contract(error)),
    }
}

/// Verify missing template bindings fail closed.
///
/// # Errors
/// Returns an error if a missing template path resolves.
#[must_use = "negative controls must be counted"]
pub fn verify_missing_template_fails(
    boundary: &BoundaryContracts,
) -> Result<usize, ContractsRegistryBundleWorkflowError> {
    match boundary.resolve_string("${subject.missing}", &template_bindings()) {
        Ok(_) => Err(ContractsRegistryBundleWorkflowError::MissingTemplateAccepted),
        Err(TemplateError::MissingPath { .. }) => Ok(1),
        Err(error) => Err(ContractsRegistryBundleWorkflowError::Template(error)),
    }
}

/// Verify required authz policy fields are fail-closed at registry compile time.
///
/// # Errors
/// Returns an error if an incomplete required policy compiles.
#[must_use = "negative controls must be counted"]
pub fn verify_required_authz_policy_fails_closed(
) -> Result<usize, ContractsRegistryBundleWorkflowError> {
    let yaml = REGISTRY_YAML.replacen(
        "      policy_id: release-governance-pdp\n      policy_version: \"2026-06-29\"\n",
        "      policy_id: \"\"\n      policy_version: \"2026-06-29\"\n",
        1,
    );
    let manifest = RegistryManifest::from_yaml_str(&yaml)?;
    match CompiledDispatchBundle::compile(&manifest) {
        Ok(_) => Err(ContractsRegistryBundleWorkflowError::InvalidAuthzPolicyAccepted),
        Err(ContractError::Validation(_)) => Ok(1),
        Err(error) => Err(ContractsRegistryBundleWorkflowError::Contract(error)),
    }
}

/// Verify plan hashes are sensitive to plan material changes.
///
/// # Errors
/// Returns an error if a material mutation keeps the same plan hash.
#[must_use = "negative controls must be counted"]
pub fn verify_plan_material_mutation_changes_hash(
    material: &PlanHashMaterial,
) -> Result<usize, ContractsRegistryBundleWorkflowError> {
    let baseline = material.compute_plan_hash()?;
    let mut mutated = material.clone();
    if let Some(op) = mutated.ops.first_mut() {
        op.kind = "tampered_op_kind".to_owned();
    }
    let changed = mutated.compute_plan_hash()?;
    if baseline == changed {
        return Err(ContractsRegistryBundleWorkflowError::PlanHashMutationNotDetected);
    }
    Ok(1)
}

fn find_predicate<'a>(
    bundle: &'a CompiledDispatchBundle,
    predicate: &'static str,
) -> Result<&'a causlane_contracts::CompiledPredicate, ContractsRegistryBundleWorkflowError> {
    bundle
        .body
        .predicates
        .iter()
        .find(|compiled| compiled.predicate == predicate)
        .ok_or(ContractsRegistryBundleWorkflowError::MissingPredicate(
            predicate,
        ))
}

fn release_governance_plan_material(bundle_hash: &str) -> PlanHashMaterial {
    let subject = br#"{"release_candidate_id":"rc_456","target_environment":"production"}"#;
    let circumstance = br#"{"requested_by":"bob","change_ticket":"CHG-456"}"#;
    PlanHashMaterial {
        hash_schema_version: 1,
        bundle_id: "causlane.example.release_governance".to_owned(),
        bundle_version: "0.0.0".to_owned(),
        bundle_hash: bundle_hash.to_owned(),
        planner_id: "release-governance-planner".to_owned(),
        planner_version: "0.0.0".to_owned(),
        planner_fingerprint: "example".to_owned(),
        action_id: "act_release_governance_456".to_owned(),
        predicate: "release.promote_candidate".to_owned(),
        predicate_version: 1,
        subject_fingerprint: content_hash(subject).0,
        circumstance_fingerprint: content_hash(circumstance).0,
        consequence_profile: "RuntimeExecution".to_owned(),
        lifecycle_class: "execution_bearing".to_owned(),
        route_id: "route.runtime_execution".to_owned(),
        ops: vec![CanonicalOp {
            index: 0,
            kind: "promote_release_candidate".to_owned(),
            effect: CanonicalEffect {
                reads: vec![
                    "release_candidate:rc_456".to_owned(),
                    "environment:production".to_owned(),
                ],
                writes: vec!["environment:production".to_owned()],
                produces: vec!["release.promoted".to_owned()],
                requires: vec!["release.window_approved".to_owned()],
                invalidates: Vec::new(),
                conflict_domains: vec![
                    "environment:production".to_owned(),
                    "release_candidate:rc_456".to_owned(),
                ],
                hardness: "hard".to_owned(),
            },
        }],
        planned_impacts: vec![
            CanonicalImpact {
                scope: "environment:production".to_owned(),
                hardness: "hard".to_owned(),
            },
            CanonicalImpact {
                scope: "release_candidate:rc_456".to_owned(),
                hardness: "hard".to_owned(),
            },
        ],
        required_witnesses: vec!["window_approved_before_promotion".to_owned()],
        required_claims: vec![
            CanonicalClaim {
                resource: "environment_write".to_owned(),
                scope: "environment:production".to_owned(),
                mode: "exclusive".to_owned(),
                amount: 1,
            },
            CanonicalClaim {
                resource: "release_candidate_write".to_owned(),
                scope: "release_candidate:rc_456".to_owned(),
                mode: "exclusive".to_owned(),
                amount: 1,
            },
        ],
        barrier_policy: "strict_write_ahead".to_owned(),
        projection_policy: "anchored".to_owned(),
    }
}

fn template_bindings() -> TemplateBindings {
    TemplateBindings::from_pairs(
        [
            ("release_candidate_id".to_owned(), "rc_456".to_owned()),
            ("target_environment".to_owned(), "production".to_owned()),
        ],
        [
            ("requested_by".to_owned(), "bob".to_owned()),
            ("change_ticket".to_owned(), "CHG-456".to_owned()),
        ],
    )
}

fn require_hash(
    label: &'static str,
    value: &str,
) -> Result<(), ContractsRegistryBundleWorkflowError> {
    if is_canonical_sha256_token(value) {
        Ok(())
    } else {
        Err(ContractsRegistryBundleWorkflowError::NonCanonicalHash(
            label,
        ))
    }
}

fn unexpected<T: fmt::Debug>(
    check: &'static str,
    actual: &T,
) -> ContractsRegistryBundleWorkflowError {
    ContractsRegistryBundleWorkflowError::UnexpectedOutcome {
        check,
        actual: format!("{actual:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        run_contracts_registry_bundle_workflow, verify_missing_template_fails,
        verify_plan_material_mutation_changes_hash, verify_plan_template_cache,
        verify_required_authz_policy_fails_closed, verify_tampered_bundle_is_rejected,
        ContractsRegistryBundleWorkflowError,
    };
    use causlane_contracts::{BoundaryContracts, BundleCompiler, RegistryManifest};

    #[test]
    fn contracts_registry_bundle_workflow_summary_counts(
    ) -> Result<(), ContractsRegistryBundleWorkflowError> {
        let summary = run_contracts_registry_bundle_workflow()?;

        assert_eq!(summary.parsed_manifests, 1);
        assert_eq!(summary.compiled_predicates, 2);
        assert_eq!(summary.verified_artifacts, 1);
        assert_eq!(summary.canonical_hashes, 4);
        assert_eq!(summary.resolved_templates, 3);
        assert_eq!(summary.plan_cache_lookups, 2);
        assert_eq!(summary.negative_controls, 4);
        Ok(())
    }

    #[test]
    fn negative_controls_are_independently_observable(
    ) -> Result<(), ContractsRegistryBundleWorkflowError> {
        let boundary = BoundaryContracts;
        let manifest = RegistryManifest::from_yaml_str(super::REGISTRY_YAML)?;
        let bundle = boundary.compile_registry(&manifest)?;
        let (_, material) = super::verify_plan_and_impact_hashes(&bundle)?;

        assert_eq!(verify_tampered_bundle_is_rejected(&bundle)?, 1);
        assert_eq!(verify_missing_template_fails(&boundary)?, 1);
        assert_eq!(verify_required_authz_policy_fails_closed()?, 1);
        assert_eq!(verify_plan_material_mutation_changes_hash(&material)?, 1);
        Ok(())
    }

    #[test]
    fn plan_template_cache_reports_miss_then_hit(
    ) -> Result<(), ContractsRegistryBundleWorkflowError> {
        let boundary = BoundaryContracts;
        let manifest = RegistryManifest::from_yaml_str(super::REGISTRY_YAML)?;
        let bundle = boundary.compile_registry(&manifest)?;
        let (_, material) = super::verify_plan_and_impact_hashes(&bundle)?;

        assert_eq!(verify_plan_template_cache(&material)?, 2);
        Ok(())
    }
}
