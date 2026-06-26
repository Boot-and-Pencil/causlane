//! Compiled dispatch bundle: the artifact runtime, replay, tests and (later)
//! generated formal models all consume (ADR-0014).

use serde::{Deserialize, Serialize};

use causlane_core::{BundleHash, PredicateId};

use crate::invariants::{is_active_invariant_id, ACTIVE_INVARIANT_RANGE};
use crate::registry::{
    AuthzPolicyManifest, BarrierPolicyDto, ClaimManifest, ConsequenceProfileDto,
    ConstraintPolicyManifest, LifecycleClassDto, MergeProtocolApplicabilityManifest,
    ProjectionPolicyDto, RegistryManifest, RequiredWitnessManifest, RouteDerivationDto,
    SchemaHashesManifest, TruthCommitPolicyDto,
};
use crate::ContractError;
use crate::{canonical_json_hash, CANONICAL_SERIALIZATION_VERSION};

/// A compiled, content-addressed dispatch bundle.
///
/// [`Self::bundle_hash`] is the SHA-256 over the canonical serialization of
/// [`Self::body`] and feeds the plan hash material (ADR-0009).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledDispatchBundle {
    /// Content hash of [`Self::body`] (`sha256:...`).
    pub bundle_hash: BundleHash,
    /// The hashed bundle content.
    pub body: BundleBody,
}

/// Serializable on-disk compiled bundle artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleArtifact {
    /// Content hash of `body` (`sha256:...`).
    pub bundle_hash: String,
    /// The hashed bundle content.
    pub body: BundleBody,
}

/// The canonical, hashed content of a bundle (everything except the hash).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleBody {
    /// Bundle schema version.
    pub bundle_schema_version: u32,
    /// Stable bundle identifier.
    pub bundle_id: String,
    /// Bundle content version (inherited from the registry).
    pub bundle_version: String,
    /// Compiled predicates.
    pub predicates: Vec<CompiledPredicate>,
    /// Declared merge protocols (empty means: no concurrent overlapping writes).
    pub merge_protocols: Vec<MergeProtocolSpec>,
}

/// A predicate as compiled into a bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledPredicate {
    /// Canonical predicate id.
    pub predicate: String,
    /// Predicate schema version.
    pub version: u32,
    /// Consequence classification.
    pub consequence_profile: ConsequenceProfileDto,
    /// Lifecycle class.
    pub lifecycle_class: LifecycleClassDto,
    /// Stable route chosen for this predicate.
    pub route_id: String,
    /// How the route was derived.
    pub route_derivation: RouteDerivationDto,
    /// Barrier policy.
    pub barrier_policy: BarrierPolicyDto,
    /// Projection anchoring policy.
    pub projection_policy: ProjectionPolicyDto,
    /// Authorization policy.
    pub authz_policy: AuthzPolicyManifest,
    /// Truth commit policy.
    pub truth_commit_policy: TruthCommitPolicyDto,
    /// Formal invariant ids this predicate contributes evidence for.
    pub formal_obligations: Vec<String>,
    /// Schema hashes consumed by generated formal facts.
    pub schema_hashes: SchemaHashesManifest,
    /// Constraint/lease policy.
    pub constraint_policy: ConstraintPolicyManifest,
    /// Merge-protocol applicability declarations.
    pub merge_protocol_applicability: Vec<MergeProtocolApplicabilityManifest>,
    /// Subject schema reference.
    pub subject_schema_ref: String,
    /// Circumstance schema reference.
    pub circumstance_schema_ref: String,
    /// Required witnesses.
    pub required_witnesses: Vec<RequiredWitnessManifest>,
    /// Resource claims.
    pub claims: Vec<ClaimManifest>,
    /// Stages that require an authorization decision.
    pub authz_required_stages: Vec<String>,
    /// Selector/policy for authz evidence binding.
    pub authz_decision_selector: AuthzDecisionSelectorSpec,
    /// Witness selector schema understood by replay/codegen.
    pub witness_selector_schema: String,
    /// Resource claim templates before subject/circumstance resolution.
    pub claim_templates: Vec<ClaimManifest>,
    /// Lease policy derived from the constraint policy.
    pub lease_policy: LeasePolicySpec,
    /// Impact-set binding policy.
    pub impact_set_policy: String,
    /// Effect templates consumed by generated formal facts.
    pub effect_templates: Vec<EffectTemplateSpec>,
    /// Lifecycle transition grammar token.
    pub lifecycle_transition_policy: String,
    /// Projection anchor policy token.
    pub projection_anchor_policy: String,
    /// Scenario files included in the formal-ready matrix.
    pub scenario_refs: Vec<String>,
}

/// Authorization evidence selector compiled into the bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthzDecisionSelectorSpec {
    /// Whether authz evidence is required.
    pub required: bool,
    /// Required lifecycle stages.
    pub stages: Vec<String>,
    /// Decision selector schema version.
    pub selector_schema: String,
}

/// Lease/claim policy compiled into the bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeasePolicySpec {
    /// Whether every resolved claim must have an active lease.
    pub claim_coverage: crate::registry::ConstraintPolicyModeDto,
    /// Whether overlapping lease conflicts are checked.
    pub lease_conflicts: crate::registry::ConstraintPolicyModeDto,
    /// Drain/epoch policy token.
    pub drain_policy: String,
}

/// Effect template compiled into the bundle for generated formal facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectTemplateSpec {
    /// Operation kind or selector this template applies to.
    pub op_kind: String,
    /// Resolved/templated read scopes.
    pub reads: Vec<String>,
    /// Resolved/templated write scopes.
    pub writes: Vec<String>,
    /// Produced fact kinds.
    pub produces: Vec<String>,
    /// Required fact kinds.
    pub requires: Vec<String>,
    /// Conflict-domain scope expressions used by I-006/I-007 reasoning.
    pub conflict_domains: Vec<String>,
    /// Hardness classification of the effect.
    pub hardness: crate::registry::EffectHardnessDto,
    /// Idempotency-domain expression, if the op is idempotent.
    pub idempotency_domain: Option<String>,
}

/// A bundle-level merge protocol (ADR-0012). Absence of a matching `Verified`
/// protocol means overlapping mutable writes conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeProtocolSpec {
    /// Stable protocol id referenced by ops.
    pub id: String,
    /// Protocol version.
    pub version: u32,
    /// Verification status.
    pub status: MergeProtocolStatus,
    /// The join algebra used to merge concurrent results.
    pub algebra: MergeAlgebra,
}

/// Verification status of a merge protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeProtocolStatus {
    /// No protocol is declared for the effect (the default; fail-closed).
    Absent,
    /// Declared but not yet verified; not usable to permit concurrency.
    DeclaredButUnverified,
    /// Verified and usable to permit overlapping mutable writes.
    Verified,
    /// Explicitly disabled.
    Disabled,
}

impl MergeProtocolStatus {
    /// Only a `Verified` protocol may permit overlapping mutable writes.
    /// `Absent`, `DeclaredButUnverified` and `Disabled` all fail closed.
    #[must_use]
    pub fn permits_concurrency(self) -> bool {
        matches!(self, MergeProtocolStatus::Verified)
    }
}

/// Whether two overlapping mutable effects may run concurrently (§7.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeDecision {
    /// No verified protocol applies — overlapping mutable writes conflict.
    NotMergeable,
    /// A verified merge protocol permits concurrency.
    Mergeable {
        /// The verified protocol's id.
        protocol_id: String,
    },
}

impl MergeDecision {
    /// Whether this decision permits overlapping mutable writes. Feeds the core
    /// `ConflictOracle`'s `verified_merge` flag, so the conflict oracle and the
    /// merge resolver agree on one fail-closed answer.
    #[must_use]
    pub fn permits_concurrency(&self) -> bool {
        matches!(self, MergeDecision::Mergeable { .. })
    }
}

/// §7.7 — the merge-protocol resolver contract. Resolving a [`MergeDecision`] is
/// the only sanctioned way to decide whether overlapping mutable effects may run
/// concurrently; the default and every non-`Verified` status fail closed.
pub trait MergeSemantics {
    /// Resolve the merge decision for an op kind from the bundle's protocols and
    /// a predicate's applicability declarations.
    fn merge_decision(
        &self,
        protocols: &[MergeProtocolSpec],
        applicability: &[crate::registry::MergeProtocolApplicabilityManifest],
        op_kind: &str,
    ) -> MergeDecision;
}

/// The canonical merge-semantics authority, delegating to [`merge_decision`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KernelMergeSemantics;

impl MergeSemantics for KernelMergeSemantics {
    fn merge_decision(
        &self,
        protocols: &[MergeProtocolSpec],
        applicability: &[crate::registry::MergeProtocolApplicabilityManifest],
        op_kind: &str,
    ) -> MergeDecision {
        merge_decision(protocols, applicability, op_kind)
    }
}

/// Resolve the merge decision for an op kind from the bundle's merge protocols
/// and a predicate's applicability declarations. Fail-closed: only a `Verified`
/// protocol whose applicability matches the op yields `Mergeable`.
#[must_use]
pub fn merge_decision(
    protocols: &[MergeProtocolSpec],
    applicability: &[crate::registry::MergeProtocolApplicabilityManifest],
    op_kind: &str,
) -> MergeDecision {
    for applies in applicability {
        if applies.applies_to != op_kind {
            continue;
        }
        if let Some(protocol) = protocols
            .iter()
            .find(|protocol| protocol.id == applies.protocol_id)
        {
            if protocol.status.permits_concurrency() {
                return MergeDecision::Mergeable {
                    protocol_id: protocol.id.clone(),
                };
            }
        }
    }
    MergeDecision::NotMergeable
}

/// Resolve the conflict-domain scopes a **verified, applicable** merge protocol
/// permits overlapping mutable writes on (I-006), for one predicate under a set of
/// template bindings, as sorted scope tokens. Fail-closed: with no applicable
/// verified protocol the set is empty and every overlapping exclusive lease
/// conflicts. This is the single resolver the replay oracle and the Alloy
/// generator both consult, so they cannot diverge on which scopes are mergeable
/// (P0-005).
///
/// # Errors
/// Returns [`ContractError`] if a conflict-domain expression cannot be resolved
/// against the bindings.
pub fn resolve_mergeable_scopes(
    bundle: &CompiledDispatchBundle,
    predicate: &crate::CompiledPredicate,
    bindings: &crate::template::TemplateBindings,
) -> Result<Vec<String>, ContractError> {
    use crate::contract::{BoundaryContracts, TemplateResolver};
    let mut scopes = std::collections::BTreeSet::new();
    for template in &predicate.effect_templates {
        let decision = merge_decision(
            &bundle.body.merge_protocols,
            &predicate.merge_protocol_applicability,
            &template.op_kind,
        );
        if decision.permits_concurrency() {
            for expression in &template.conflict_domains {
                let scope = BoundaryContracts
                    .resolve_scope(expression, bindings)
                    .map_err(|err| {
                        ContractError::Validation(format!(
                            "conflict-domain scope '{expression}': {err:?}"
                        ))
                    })?;
                scopes.insert(scope.0);
            }
        }
    }
    Ok(scopes.into_iter().collect())
}

/// The deterministic join algebra a merge protocol relies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeAlgebra {
    /// Commutative, idempotent join (e.g. set union).
    CommutativeIdempotentJoin,
    /// Append-only writes over disjoint keys.
    AppendOnlyDisjointKeys,
    /// Additive counter.
    AdditiveCounter,
    /// Last-writer-wins is explicitly forbidden as a merge algebra.
    LastWriterWinsForbidden,
}

impl CompiledDispatchBundle {
    /// Compile a registry manifest into a content-addressed bundle.
    ///
    /// # Errors
    /// Returns [`ContractError::Json`] if the canonical serialization used for
    /// hashing fails.
    #[must_use = "the compiled bundle must be used"]
    pub fn compile(manifest: &RegistryManifest) -> Result<Self, ContractError> {
        Self::compile_with_bundle_id(manifest, &manifest.bundle_id)
    }

    /// Compile with an explicit `bundle_id` override — for tests / dev only.
    /// Normal compilation uses [`Self::compile`] so a registry always yields the
    /// same `bundle_hash` regardless of the caller (B-005).
    ///
    /// # Errors
    /// Returns [`ContractError::Json`] if the canonical serialization used for
    /// hashing fails.
    #[must_use = "the compiled bundle must be used"]
    pub fn compile_with_bundle_id(
        manifest: &RegistryManifest,
        bundle_id: &str,
    ) -> Result<Self, ContractError> {
        validate_manifest(manifest, bundle_id)?;
        let predicates = manifest
            .predicates
            .iter()
            .map(|p| CompiledPredicate {
                predicate: p.id.clone(),
                version: p.version,
                consequence_profile: p.consequence_profile,
                lifecycle_class: p.lifecycle_class,
                route_id: p.route_id.clone(),
                route_derivation: p.route_derivation,
                barrier_policy: p.barrier_policy,
                projection_policy: p.projection_policy,
                authz_policy: p.authz_policy.clone(),
                truth_commit_policy: p.truth_commit_policy,
                formal_obligations: p.formal_obligations.clone(),
                schema_hashes: p.schema_hashes.clone(),
                constraint_policy: p.constraint_policy.clone(),
                merge_protocol_applicability: p.merge_protocol_applicability.clone(),
                subject_schema_ref: p.subject_schema_ref.clone(),
                circumstance_schema_ref: p.circumstance_schema_ref.clone(),
                required_witnesses: p.required_witnesses.clone(),
                claims: p.claims.clone(),
                authz_required_stages: p.authz_policy.stages.clone(),
                authz_decision_selector: AuthzDecisionSelectorSpec {
                    required: p.authz_policy.mode == crate::registry::AuthzModeDto::Required,
                    stages: p.authz_policy.stages.clone(),
                    selector_schema: format!("canonical_json_v{CANONICAL_SERIALIZATION_VERSION}"),
                },
                witness_selector_schema: "subject_circumstance_template_v0_2".to_owned(),
                claim_templates: p.claims.clone(),
                lease_policy: LeasePolicySpec {
                    claim_coverage: p.constraint_policy.claim_coverage,
                    lease_conflicts: p.constraint_policy.lease_conflicts,
                    drain_policy: "drain_fence_v0_2".to_owned(),
                },
                impact_set_policy: "required_for_runtime_execution".to_owned(),
                effect_templates: p
                    .effect_templates
                    .iter()
                    .map(|effect| EffectTemplateSpec {
                        op_kind: effect.op_kind.clone(),
                        reads: effect.reads.clone(),
                        writes: effect.writes.clone(),
                        produces: effect.produces.clone(),
                        requires: effect.requires.clone(),
                        conflict_domains: effect.conflict_domains.clone(),
                        hardness: effect.hardness,
                        idempotency_domain: effect.idempotency_domain.clone(),
                    })
                    .collect(),
                lifecycle_transition_policy: lifecycle_transition_policy(p.lifecycle_class),
                projection_anchor_policy: projection_anchor_policy(p.projection_policy),
                scenario_refs: p.scenario_refs.clone(),
            })
            .collect();

        let body = BundleBody {
            bundle_schema_version: 3,
            bundle_id: bundle_id.to_owned(),
            bundle_version: manifest.registry_version.clone(),
            predicates,
            merge_protocols: manifest.merge_protocols.clone(),
        };

        let bundle_hash = BundleHash(canonical_json_hash(&body)?);
        Ok(Self { bundle_hash, body })
    }

    /// Convert to the serializable on-disk artifact shape.
    #[must_use]
    pub fn to_artifact(&self) -> BundleArtifact {
        BundleArtifact {
            bundle_hash: self.bundle_hash.0.clone(),
            body: self.body.clone(),
        }
    }

    /// Serialize this bundle artifact as pretty JSON.
    ///
    /// # Errors
    /// Returns [`ContractError::Json`] if serialization fails.
    #[must_use = "the JSON output must be used"]
    pub fn to_json_pretty(&self) -> Result<String, ContractError> {
        Ok(serde_json::to_string_pretty(&self.to_artifact())?)
    }

    /// Parse a compiled bundle artifact and verify its `bundle_hash`.
    ///
    /// # Errors
    /// Returns [`ContractError::Json`] for invalid JSON and
    /// [`ContractError::Validation`] if the embedded hash is stale.
    #[must_use = "the parsed bundle must be used"]
    pub fn from_json_str(json: &str) -> Result<Self, ContractError> {
        let artifact: BundleArtifact = serde_json::from_str(json)?;
        if artifact.body.bundle_schema_version != 3 {
            return Err(ContractError::Validation(format!(
                "bundle_schema_version must be 3, got {}",
                artifact.body.bundle_schema_version
            )));
        }
        let expected = canonical_json_hash(&artifact.body)?;
        if artifact.bundle_hash != expected {
            return Err(ContractError::Validation(format!(
                "bundle_hash mismatch: expected {expected}, got {}",
                artifact.bundle_hash
            )));
        }
        // P0-010: re-enforce the load-bearing authz policy-binding invariant on
        // the deserialization path. Compile-time `validate_authz` does not run
        // here, and an empty expected policy silently disables the downstream
        // policy check (replay/runtime), so a hand-built or forged-hash bundle
        // could otherwise carry `mode=required` with no policy. Fail closed.
        for predicate in &artifact.body.predicates {
            if predicate.authz_policy.mode == crate::registry::AuthzModeDto::Required
                && (predicate.authz_policy.policy_id.is_empty()
                    || predicate.authz_policy.policy_version.is_empty())
            {
                return Err(ContractError::Validation(format!(
                    "predicate {} authz_policy=required must declare policy_id and policy_version",
                    predicate.predicate
                )));
            }
        }
        Ok(Self {
            bundle_hash: BundleHash(artifact.bundle_hash),
            body: artifact.body,
        })
    }

    /// Look up a compiled predicate by id.
    #[must_use]
    pub fn predicate(&self, id: &PredicateId) -> Option<&CompiledPredicate> {
        self.body.predicates.iter().find(|p| p.predicate == id.0)
    }
}

fn lifecycle_transition_policy(lifecycle: LifecycleClassDto) -> String {
    match lifecycle {
        LifecycleClassDto::ExecutionBearing => "execution_bearing_v0_2".to_owned(),
        LifecycleClassDto::ProjectionOnly => "projection_only_v0_2".to_owned(),
        LifecycleClassDto::Meta => "meta_v0_2".to_owned(),
    }
}

fn projection_anchor_policy(policy: ProjectionPolicyDto) -> String {
    match policy {
        ProjectionPolicyDto::Anchored => "anchored_prior_truth_same_plan".to_owned(),
        ProjectionPolicyDto::None => "not_required".to_owned(),
    }
}

fn validate_manifest(manifest: &RegistryManifest, bundle_id: &str) -> Result<(), ContractError> {
    require_nonempty("bundle_id", bundle_id)?;
    require_nonempty("registry_version", &manifest.registry_version)?;
    if manifest.predicates.is_empty() {
        return Err(ContractError::Validation(
            "registry must declare at least one predicate".to_owned(),
        ));
    }
    for predicate in &manifest.predicates {
        validate_predicate(predicate)?;
    }
    Ok(())
}

fn validate_predicate(predicate: &crate::registry::PredicateManifest) -> Result<(), ContractError> {
    require_nonempty("predicate.id", &predicate.id)?;
    require_nonempty("predicate.route_id", &predicate.route_id)?;
    require_nonempty(
        "predicate.subject_schema_ref",
        &predicate.subject_schema_ref,
    )?;
    require_nonempty(
        "predicate.circumstance_schema_ref",
        &predicate.circumstance_schema_ref,
    )?;
    require_sha(
        "predicate.schema_hashes.subject",
        &predicate.schema_hashes.subject,
    )?;
    require_sha(
        "predicate.schema_hashes.circumstance",
        &predicate.schema_hashes.circumstance,
    )?;

    // I-005: the route's lifecycle class is derived from the consequence profile,
    // not chosen freely. Delegate to the single core rule the formal lanes verify
    // (`route_consistent_with_profile`) rather than duplicating the mapping here.
    let profile = predicate.consequence_profile.to_core();
    let expected_lifecycle = causlane_core::lifecycle_class_for_profile(profile);
    if !causlane_core::route_consistent_with_profile(predicate.lifecycle_class.to_core(), profile) {
        return Err(ContractError::Validation(format!(
            "predicate {} has lifecycle {:?}, expected {:?} for profile {:?}",
            predicate.id,
            predicate.lifecycle_class,
            expected_lifecycle,
            predicate.consequence_profile
        )));
    }

    validate_authz(predicate)?;
    validate_formal_obligations(predicate)?;
    validate_required_witnesses(predicate)?;
    validate_claims(predicate)?;
    validate_effect_templates(predicate)?;
    validate_merge_applicability(predicate)?;
    validate_scenario_refs(predicate)?;

    match predicate.consequence_profile {
        ConsequenceProfileDto::RuntimeExecution => {
            if predicate.barrier_policy != BarrierPolicyDto::StrictWriteAhead {
                return Err(ContractError::Validation(format!(
                    "RuntimeExecution predicate {} requires barrier_policy=strict_write_ahead",
                    predicate.id
                )));
            }
            if predicate.truth_commit_policy != TruthCommitPolicyDto::Allowed {
                return Err(ContractError::Validation(format!(
                    "RuntimeExecution predicate {} requires truth_commit_policy=allowed",
                    predicate.id
                )));
            }
            if predicate.claims.is_empty() {
                return Err(ContractError::Validation(format!(
                    "RuntimeExecution predicate {} must declare resource claims",
                    predicate.id
                )));
            }
            if predicate.constraint_policy.claim_coverage
                != crate::registry::ConstraintPolicyModeDto::Required
            {
                return Err(ContractError::Validation(format!(
                    "RuntimeExecution predicate {} requires claim coverage",
                    predicate.id
                )));
            }
        }
        ConsequenceProfileDto::ProjectionRead => {
            if predicate.projection_policy != ProjectionPolicyDto::Anchored {
                return Err(ContractError::Validation(format!(
                    "ProjectionRead predicate {} requires projection_policy=anchored",
                    predicate.id
                )));
            }
            if predicate.truth_commit_policy != TruthCommitPolicyDto::Disallowed {
                return Err(ContractError::Validation(format!(
                    "ProjectionRead predicate {} requires truth_commit_policy=disallowed",
                    predicate.id
                )));
            }
        }
        ConsequenceProfileDto::OversightMeta
        | ConsequenceProfileDto::TopologyMeta
        | ConsequenceProfileDto::EvidenceMeta
        | ConsequenceProfileDto::OutsideKernel => {}
    }

    Ok(())
}

fn validate_claims(predicate: &crate::registry::PredicateManifest) -> Result<(), ContractError> {
    for claim in &predicate.claims {
        require_nonempty("claim.resource", &claim.resource)?;
        require_nonempty("claim.scope_expr", &claim.scope_expr)?;
        validate_template_expression("claim.scope_expr", &claim.scope_expr)?;
    }
    Ok(())
}

fn validate_effect_templates(
    predicate: &crate::registry::PredicateManifest,
) -> Result<(), ContractError> {
    for effect in &predicate.effect_templates {
        require_nonempty("effect_template.op_kind", &effect.op_kind)?;
        for scope in effect
            .reads
            .iter()
            .chain(&effect.writes)
            .chain(&effect.conflict_domains)
        {
            validate_template_expression("effect_template.scope", scope)?;
        }
        if let Some(domain) = &effect.idempotency_domain {
            validate_template_expression("effect_template.idempotency_domain", domain)?;
        }
    }
    Ok(())
}

fn validate_formal_obligations(
    predicate: &crate::registry::PredicateManifest,
) -> Result<(), ContractError> {
    for invariant in &predicate.formal_obligations {
        if !is_active_invariant_id(invariant) {
            return Err(ContractError::Validation(format!(
                "predicate {} references unknown active invariant id {}; expected {}",
                predicate.id, invariant, ACTIVE_INVARIANT_RANGE
            )));
        }
    }
    Ok(())
}

fn validate_authz(predicate: &crate::registry::PredicateManifest) -> Result<(), ContractError> {
    match predicate.authz_policy.mode {
        crate::registry::AuthzModeDto::Required => {
            if predicate.authz_policy.stages.is_empty() {
                return Err(ContractError::Validation(format!(
                    "predicate {} authz_policy=required must list stages",
                    predicate.id
                )));
            }
            // P0-010: a required authz policy must name the policy every decision
            // is expected to be issued under, so replay/runtime can reject an
            // otherwise-valid decision carried under the wrong policy.
            require_nonempty(
                "predicate.authz_policy.policy_id",
                &predicate.authz_policy.policy_id,
            )?;
            require_nonempty(
                "predicate.authz_policy.policy_version",
                &predicate.authz_policy.policy_version,
            )?;
        }
        crate::registry::AuthzModeDto::DisabledForLocalDev => {
            if !predicate
                .authz_policy
                .allowed_in_profiles
                .contains(&predicate.consequence_profile)
            {
                return Err(ContractError::Validation(format!(
                    "predicate {} local-dev authz exemption must list its profile",
                    predicate.id
                )));
            }
            require_nonempty(
                "predicate.authz_policy.rationale",
                &predicate.authz_policy.rationale,
            )?;
        }
    }
    Ok(())
}

fn validate_required_witnesses(
    predicate: &crate::registry::PredicateManifest,
) -> Result<(), ContractError> {
    for witness in &predicate.required_witnesses {
        require_nonempty("witness.id", &witness.id)?;
        if !known_stage(&witness.target_stage) {
            return Err(ContractError::Validation(format!(
                "predicate {} witness {} has unknown target_stage {}",
                predicate.id, witness.id, witness.target_stage
            )));
        }
        require_nonempty("witness.selector.event_kind", &witness.selector.event_kind)?;
        require_nonempty("witness.selector.predicate", &witness.selector.predicate)?;
        require_nonempty("witness.selector.fact_kind", &witness.selector.fact_kind)?;
        require_nonempty("witness.selector.scope_expr", &witness.selector.scope_expr)?;
        validate_template_expression("witness.selector.scope_expr", &witness.selector.scope_expr)?;
    }
    Ok(())
}

fn validate_scenario_refs(
    predicate: &crate::registry::PredicateManifest,
) -> Result<(), ContractError> {
    for scenario_ref in &predicate.scenario_refs {
        require_nonempty("predicate.scenario_refs", scenario_ref)?;
    }
    Ok(())
}

fn validate_merge_applicability(
    predicate: &crate::registry::PredicateManifest,
) -> Result<(), ContractError> {
    for applicability in &predicate.merge_protocol_applicability {
        require_nonempty(
            "merge_protocol_applicability.protocol_id",
            &applicability.protocol_id,
        )?;
        require_nonempty(
            "merge_protocol_applicability.applies_to",
            &applicability.applies_to,
        )?;
    }
    Ok(())
}

fn require_nonempty(field: &str, value: &str) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        return Err(ContractError::Validation(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

fn require_sha(field: &str, value: &str) -> Result<(), ContractError> {
    if crate::is_canonical_sha256_token(value) {
        Ok(())
    } else {
        Err(ContractError::Validation(format!(
            "{field} must be a sha256:<64 lowercase hex> token"
        )))
    }
}

fn known_stage(stage: &str) -> bool {
    matches!(
        stage,
        "action_admitted"
            | "action_planned"
            | "dispatch_logged"
            | "execution_barrier_logged"
            | "execution_started"
            | "execution_completed"
            | "observed_truth_committed"
            | "projection_emitted"
            | "lifecycle_closed"
            | "gate_approved"
            | "gate_denied"
    )
}

fn validate_template_expression(field: &str, expression: &str) -> Result<(), ContractError> {
    crate::template::validate_template_expression(expression)
        .map_err(|err| ContractError::Validation(format!("{field}: {err}")))
}
