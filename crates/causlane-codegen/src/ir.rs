//! Target-neutral formal IR projected from compiled bundles and scenarios.

use std::collections::BTreeSet;

use causlane_contracts::{
    canonical_json_hash, is_active_invariant_id, BarrierPolicyDto, ClaimModeDto,
    CompiledDispatchBundle, CompiledPredicate, ConsequenceProfileDto, ConstraintPolicyModeDto,
    EffectHardnessDto, LifecycleClassDto, MergeProtocolStatus, ProjectionPolicyDto,
    TruthCommitPolicyDto, ACTIVE_INVARIANT_RANGE, CANONICAL_SERIALIZATION_VERSION,
};
use serde::{Deserialize, Serialize};

use crate::{AlloyEventKind, AlloyLeaseMode, AlloyScenarioFacts, CodegenError, GENERATOR_VERSION};

const FORMAL_IR_SCHEMA_VERSION: u32 = 2;

/// Target-neutral formal IR v2.
///
/// v2 is a strict superset of v1: it additionally projects the producer
/// attestation (`fact_kind`/`scope`) carried on scenario events and the
/// bundle's merge-protocol fail-closed `status`. Both are *faithful projections*
/// of facts the replay oracle already enforces (P0-004 / I-006), surfaced into
/// the target-neutral IR so downstream generators can ground their checks in the
/// same payload rather than self-asserting it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalIr {
    /// Formal IR schema version.
    pub schema_version: u32,
    /// Hash of the IR digest material (`sha256:...`), excluding this field.
    pub formal_ir_hash: String,
    /// Generator version that emitted the IR.
    pub generator_version: String,
    /// Canonical serialization policy version used for the digest.
    pub canonical_serialization_version: u32,
    /// Source compiled bundle hash.
    pub source_bundle_hash: String,
    /// Source bundle schema version.
    pub bundle_schema_version: u32,
    /// Stable source bundle id.
    pub bundle_id: String,
    /// Source bundle version.
    pub bundle_version: String,
    /// Optional scenario hash when the IR is scenario-bound.
    pub scenario_hash: Option<String>,
    /// Expected scenario result (`pass` or `fail`) when scenario-bound.
    pub expected_result: Option<String>,
    /// Expected stable replay error code for negative scenarios.
    pub expected_error_code: Option<String>,
    /// Predicate contracts projected from the bundle.
    pub predicates: Vec<FormalPredicate>,
    /// Bundle merge-protocol facts (v2). Only a `verified` protocol may permit
    /// overlapping mutable writes; every other status fails closed (I-006).
    pub merge_protocols: Vec<FormalMergeProtocolFact>,
    /// Scenario events projected into target-neutral form.
    pub scenario_events: Vec<FormalEvent>,
    /// Invariant ids covered by this IR.
    pub invariants: Vec<String>,
}

/// Bundle merge-protocol fail-closed status, projected into Formal IR (v2). The
/// replay oracle and `KernelMergeSemantics` already enforce that only `verified`
/// permits concurrency; this surfaces the status so a generator can model the
/// same fail-closed default for I-006 instead of assuming mergeability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalMergeProtocolFact {
    /// Stable protocol id referenced by ops.
    pub protocol_id: String,
    /// Protocol version.
    pub version: u32,
    /// Verification status token (`absent`/`declared_but_unverified`/`verified`/
    /// `disabled`); only `verified` permits concurrency.
    pub status: String,
    /// Whether this status permits overlapping mutable writes (only `verified`).
    pub permits_concurrency: bool,
}

/// Predicate contract facts consumed by formal targets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalPredicate {
    /// Predicate id.
    pub predicate: String,
    /// Predicate schema version.
    pub version: u32,
    /// Consequence profile token.
    pub consequence_profile: String,
    /// Lifecycle class token.
    pub lifecycle_class: String,
    /// Route id.
    pub route_id: String,
    /// Barrier policy token.
    pub barrier_policy: String,
    /// Projection policy token.
    pub projection_policy: String,
    /// Truth commit policy token.
    pub truth_commit_policy: String,
    /// Whether authz evidence is required.
    pub authz_required: bool,
    /// Authz-required lifecycle stages.
    pub authz_required_stages: Vec<String>,
    /// Witness selector schema token.
    pub witness_selector_schema: String,
    /// Impact-set binding policy token.
    pub impact_set_policy: String,
    /// Lifecycle transition grammar token.
    pub lifecycle_transition_policy: String,
    /// Projection anchor policy token.
    pub projection_anchor_policy: String,
    /// Lease claim coverage policy token.
    pub lease_claim_coverage: String,
    /// Lease conflict policy token.
    pub lease_conflicts: String,
    /// Drain policy token.
    pub drain_policy: String,
    /// Required witnesses.
    pub required_witnesses: Vec<FormalWitnessRequirement>,
    /// Required claims.
    pub claims: Vec<FormalClaim>,
    /// Effect templates.
    pub effect_templates: Vec<FormalEffectTemplate>,
    /// Formal invariant ids declared by this predicate.
    pub formal_obligations: Vec<String>,
    /// Scenario refs declared by this predicate.
    pub scenario_refs: Vec<String>,
}

/// Required witness selector projected into Formal IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalWitnessRequirement {
    /// Requirement id.
    pub requirement_id: String,
    /// Target lifecycle stage.
    pub target_stage: String,
    /// Required audit event kind.
    pub event_kind: String,
    /// Required producer predicate.
    pub producer_predicate: String,
    /// Required fact kind.
    pub fact_kind: String,
    /// Scope expression after bundle compilation.
    pub scope_expr: String,
}

/// Resource claim projected into Formal IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalClaim {
    /// Claimed resource.
    pub resource: String,
    /// Scope expression.
    pub scope_expr: String,
    /// Claim mode token.
    pub mode: String,
}

/// Effect template projected into Formal IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalEffectTemplate {
    /// Operation kind.
    pub op_kind: String,
    /// Read scopes.
    pub reads: Vec<String>,
    /// Write scopes.
    pub writes: Vec<String>,
    /// Produced fact kinds.
    pub produces: Vec<String>,
    /// Required fact kinds.
    pub requires: Vec<String>,
    /// Conflict-domain scopes (I-006/I-007 input).
    pub conflict_domains: Vec<String>,
    /// Hardness token (`hard`/`soft`/`meta`).
    pub hardness: String,
    /// Idempotency-domain expression, if any.
    pub idempotency_domain: Option<String>,
}

/// Scenario event projected into Formal IR (payload-bound — P0-FM-003).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalEvent {
    /// Event id.
    pub event_id: String,
    /// Event kind token.
    pub kind: String,
    /// Action this event belongs to.
    pub action_id: Option<String>,
    /// Optional plan hash.
    pub plan_hash: Option<String>,
    /// Op index, when the event binds one (e.g. execution.started).
    pub op_index: Option<u32>,
    /// Fact kind this event attests about itself (v2). Carried on producer
    /// events (e.g. `gate.approved`, `observed_truth.committed`); the replay
    /// oracle grounds witness/anchor claims against this (P0-004).
    pub fact_kind: Option<String>,
    /// Scope the attested fact applies to (v2). Paired with `fact_kind`.
    pub scope: Option<String>,
    /// Projection truth anchors (v2): structured, carrying each anchor's claimed
    /// `fact_kind`/`scope` so a generator can ground the projection against the
    /// observed-truth event's attestation (P0-004), not just the event id.
    pub anchors: Vec<FormalAnchorPayload>,
    /// Lease facts attached to this event.
    pub leases: Vec<FormalLeaseFact>,
    /// Barrier payload, when this is `execution.barrier_logged`.
    pub barrier: Option<FormalBarrierPayload>,
    /// Capability payload, when this is `execution.started`.
    pub capability: Option<FormalCapabilityPayload>,
    /// Authz decision payload, when this is `authz.decision_recorded`.
    pub authz_decision: Option<FormalAuthzDecisionPayload>,
}

/// A projection truth anchor projected into Formal IR (v2): the observed-truth
/// event the projection derives from and the fact/scope it claims about that
/// truth, so a generator can ground the claim against the observed event's own
/// attestation (P0-004).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalAnchorPayload {
    /// The observed-truth event id this projection is anchored to.
    pub event_id: String,
    /// Fact kind the projection claims for the anchored truth, if any.
    pub fact_kind: Option<String>,
    /// Scope the projection claims for the anchored truth, if any.
    pub scope: Option<String>,
}

/// Barrier facts that bind execution to an action/plan/op/impact set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalBarrierPayload {
    /// Barrier event id.
    pub barrier_event_id: String,
    /// Action id the barrier authorizes.
    pub action_id: String,
    /// Plan hash the barrier authorizes.
    pub plan_hash: String,
    /// Op indexes covered by the barrier.
    pub op_indexes: Vec<u32>,
    /// Impact set hash bound to the barrier.
    pub impact_set_hash: String,
    /// Typed witnesses (with bindings) backing the barrier.
    pub witnesses: Vec<FormalWitnessPayload>,
    /// Lease ids covering the barrier's ops.
    pub lease_ids: Vec<String>,
    /// Authz decision event ids backing the barrier.
    pub authz_decision_event_ids: Vec<String>,
}

/// Capability facts that bind an execution to a barrier-derived permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalCapabilityPayload {
    /// Capability id.
    pub capability_id: String,
    /// Barrier event id this capability derives from.
    pub barrier_event_id: String,
    /// Op index authorized.
    pub op_index: u32,
    /// Lease ids the capability spends.
    pub lease_ids: Vec<String>,
}

/// Witness facts (kind/fact/scope + action/plan/impact binding).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalWitnessPayload {
    /// Witness (evidence) event id.
    pub witness_event_id: String,
    /// Bundle required-witness id.
    pub requirement_id: String,
    /// Witness kind token.
    pub kind: String,
    /// Fact kind asserted.
    pub fact_kind: Option<String>,
    /// Scope asserted.
    pub scope: Option<String>,
    /// Bound action id (I-009).
    pub binds_to_action_id: Option<String>,
    /// Bound plan hash (I-009).
    pub binds_to_plan_hash: Option<String>,
    /// Bound impact set hash (I-009).
    pub binds_to_impact_set_hash: Option<String>,
}

/// Authz decision facts (stage / allow-deny / action / plan).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalAuthzDecisionPayload {
    /// Decision event id.
    pub decision_event_id: String,
    /// Action id authorized.
    pub action_id: String,
    /// Plan hash authorized.
    pub plan_hash: String,
    /// Lifecycle stage authorized.
    pub stage: String,
    /// Allow/deny token.
    pub decision: String,
}

/// Lease fact projected into Formal IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalLeaseFact {
    /// Lease id.
    pub lease_id: String,
    /// Resource id.
    pub resource: String,
    /// Scope token.
    pub scope: String,
    /// Lease mode token.
    pub mode: String,
    /// Constraint epoch the lease was granted in.
    #[serde(default)]
    pub epoch: u64,
}

#[derive(Serialize)]
struct FormalIrDigestMaterial<'a> {
    schema_version: u32,
    generator_version: &'a str,
    canonical_serialization_version: u32,
    source_bundle_hash: &'a str,
    bundle_schema_version: u32,
    bundle_id: &'a str,
    bundle_version: &'a str,
    scenario_hash: &'a Option<String>,
    expected_result: &'a Option<String>,
    expected_error_code: &'a Option<String>,
    predicates: &'a [FormalPredicate],
    merge_protocols: &'a [FormalMergeProtocolFact],
    scenario_events: &'a [FormalEvent],
    invariants: &'a [String],
}

impl FormalIr {
    /// Serialize the IR as pretty JSON.
    ///
    /// # Errors
    /// Returns [`CodegenError::Scenario`] if serialization fails.
    #[must_use = "the IR JSON must be written or returned"]
    pub fn to_json_pretty(&self) -> Result<String, CodegenError> {
        serde_json::to_string_pretty(self).map_err(|err| CodegenError::Scenario(err.to_string()))
    }

    /// Parse Formal IR JSON.
    ///
    /// # Errors
    /// Returns [`CodegenError::Scenario`] if decoding fails.
    #[must_use = "the parsed IR must be used"]
    pub fn from_json_str(json: &str) -> Result<Self, CodegenError> {
        serde_json::from_str(json).map_err(|err| CodegenError::Scenario(err.to_string()))
    }

    fn digest_material(&self) -> FormalIrDigestMaterial<'_> {
        FormalIrDigestMaterial {
            schema_version: self.schema_version,
            generator_version: &self.generator_version,
            canonical_serialization_version: self.canonical_serialization_version,
            source_bundle_hash: &self.source_bundle_hash,
            bundle_schema_version: self.bundle_schema_version,
            bundle_id: &self.bundle_id,
            bundle_version: &self.bundle_version,
            scenario_hash: &self.scenario_hash,
            expected_result: &self.expected_result,
            expected_error_code: &self.expected_error_code,
            predicates: &self.predicates,
            merge_protocols: &self.merge_protocols,
            scenario_events: &self.scenario_events,
            invariants: &self.invariants,
        }
    }

    fn finalize_hash(&mut self) -> Result<(), CodegenError> {
        self.formal_ir_hash = canonical_json_hash(&self.digest_material())
            .map_err(|err| CodegenError::Scenario(err.to_string()))?;
        Ok(())
    }
}

/// Build Formal IR v2 from a compiled bundle and optional scenario projection.
///
/// # Errors
/// Returns [`CodegenError::Scenario`] if invariant ids are unknown or IR
/// hashing/serialization fails.
#[must_use = "the Formal IR must be generated into target artifacts"]
pub fn build_formal_ir(
    bundle: &CompiledDispatchBundle,
    scenario: Option<&AlloyScenarioFacts>,
) -> Result<FormalIr, CodegenError> {
    let predicates = bundle
        .body
        .predicates
        .iter()
        .map(formal_predicate)
        .collect::<Result<Vec<_>, _>>()?;
    let scenario_events = match scenario {
        Some(facts) => facts.events.iter().map(formal_event).collect::<Vec<_>>(),
        None => Vec::new(),
    };
    let merge_protocols = bundle
        .body
        .merge_protocols
        .iter()
        .map(|protocol| FormalMergeProtocolFact {
            protocol_id: protocol.id.clone(),
            version: protocol.version,
            status: merge_status_token(protocol.status).to_owned(),
            permits_concurrency: protocol.status.permits_concurrency(),
        })
        .collect();
    let invariants = collect_invariants(bundle, scenario)?;
    let mut ir = FormalIr {
        schema_version: FORMAL_IR_SCHEMA_VERSION,
        formal_ir_hash: String::new(),
        generator_version: GENERATOR_VERSION.to_owned(),
        canonical_serialization_version: CANONICAL_SERIALIZATION_VERSION,
        source_bundle_hash: bundle.bundle_hash.0.clone(),
        bundle_schema_version: bundle.body.bundle_schema_version,
        bundle_id: bundle.body.bundle_id.clone(),
        bundle_version: bundle.body.bundle_version.clone(),
        scenario_hash: scenario.map(|facts| facts.scenario_hash.clone()),
        expected_result: scenario.map(|facts| facts.expected_result.clone()),
        expected_error_code: scenario.and_then(|facts| facts.expected_error_code.clone()),
        predicates,
        merge_protocols,
        scenario_events,
        invariants,
    };
    ir.finalize_hash()?;
    crate::identifier_check::check_identifier_injectivity(&ir)?;
    Ok(ir)
}

fn formal_predicate(predicate: &CompiledPredicate) -> Result<FormalPredicate, CodegenError> {
    validate_invariant_ids(&predicate.formal_obligations)?;
    Ok(FormalPredicate {
        predicate: predicate.predicate.clone(),
        version: predicate.version,
        consequence_profile: consequence_profile_token(predicate.consequence_profile).to_owned(),
        lifecycle_class: lifecycle_class_token(predicate.lifecycle_class).to_owned(),
        route_id: predicate.route_id.clone(),
        barrier_policy: barrier_policy_token(predicate.barrier_policy).to_owned(),
        projection_policy: projection_policy_token(predicate.projection_policy).to_owned(),
        truth_commit_policy: truth_commit_policy_token(predicate.truth_commit_policy).to_owned(),
        authz_required: predicate.authz_decision_selector.required,
        authz_required_stages: predicate.authz_required_stages.clone(),
        witness_selector_schema: predicate.witness_selector_schema.clone(),
        impact_set_policy: predicate.impact_set_policy.clone(),
        lifecycle_transition_policy: predicate.lifecycle_transition_policy.clone(),
        projection_anchor_policy: predicate.projection_anchor_policy.clone(),
        lease_claim_coverage: constraint_mode_token(predicate.lease_policy.claim_coverage)
            .to_owned(),
        lease_conflicts: constraint_mode_token(predicate.lease_policy.lease_conflicts).to_owned(),
        drain_policy: predicate.lease_policy.drain_policy.clone(),
        required_witnesses: predicate
            .required_witnesses
            .iter()
            .map(|witness| FormalWitnessRequirement {
                requirement_id: witness.id.clone(),
                target_stage: witness.target_stage.clone(),
                event_kind: witness.selector.event_kind.clone(),
                producer_predicate: witness.selector.predicate.clone(),
                fact_kind: witness.selector.fact_kind.clone(),
                scope_expr: witness.selector.scope_expr.clone(),
            })
            .collect(),
        claims: predicate.claims.iter().map(formal_claim).collect(),
        effect_templates: predicate
            .effect_templates
            .iter()
            .map(|effect| FormalEffectTemplate {
                op_kind: effect.op_kind.clone(),
                reads: effect.reads.clone(),
                writes: effect.writes.clone(),
                produces: effect.produces.clone(),
                requires: effect.requires.clone(),
                conflict_domains: effect.conflict_domains.clone(),
                hardness: effect_hardness_token(effect.hardness).to_owned(),
                idempotency_domain: effect.idempotency_domain.clone(),
            })
            .collect(),
        formal_obligations: predicate.formal_obligations.clone(),
        scenario_refs: predicate.scenario_refs.clone(),
    })
}

fn formal_claim(claim: &causlane_contracts::ClaimManifest) -> FormalClaim {
    FormalClaim {
        resource: claim.resource.clone(),
        scope_expr: claim.scope_expr.clone(),
        mode: claim_mode_token(claim.mode).to_owned(),
    }
}

fn formal_event(event: &crate::AlloyScenarioEvent) -> FormalEvent {
    FormalEvent {
        event_id: event.event_id.clone(),
        kind: event_kind_token(event.kind).to_owned(),
        action_id: event.action_id.clone(),
        plan_hash: event.plan_hash.clone(),
        op_index: event.op_index,
        fact_kind: event.fact_kind.clone(),
        scope: event.scope.clone(),
        anchors: event
            .anchor_facts
            .iter()
            .map(|anchor| FormalAnchorPayload {
                event_id: anchor.event_id.clone(),
                fact_kind: anchor.fact_kind.clone(),
                scope: anchor.scope.clone(),
            })
            .collect(),
        leases: event
            .leases
            .iter()
            .map(|lease| FormalLeaseFact {
                lease_id: lease.lease_id.clone(),
                resource: lease.resource.clone(),
                scope: lease.scope.clone(),
                mode: lease_mode_token(lease.mode).to_owned(),
                epoch: lease.epoch,
            })
            .collect(),
        barrier: event.barrier.clone(),
        capability: event.capability.clone(),
        authz_decision: event.authz_decision.clone(),
    }
}

fn collect_invariants(
    bundle: &CompiledDispatchBundle,
    scenario: Option<&AlloyScenarioFacts>,
) -> Result<Vec<String>, CodegenError> {
    let mut ids = BTreeSet::new();
    for predicate in &bundle.body.predicates {
        validate_invariant_ids(&predicate.formal_obligations)?;
        for invariant in &predicate.formal_obligations {
            ids.insert(invariant.clone());
        }
    }
    if let Some(facts) = scenario {
        validate_invariant_ids(&facts.formal_obligations)?;
        for invariant in &facts.formal_obligations {
            ids.insert(invariant.clone());
        }
    }
    Ok(ids.into_iter().collect())
}

fn validate_invariant_ids(ids: &[String]) -> Result<(), CodegenError> {
    for id in ids {
        if !is_active_invariant_id(id) {
            return Err(CodegenError::Scenario(format!(
                "unknown active invariant id {id}; expected {ACTIVE_INVARIANT_RANGE}"
            )));
        }
    }
    Ok(())
}

fn consequence_profile_token(value: ConsequenceProfileDto) -> &'static str {
    match value {
        ConsequenceProfileDto::RuntimeExecution => "RuntimeExecution",
        ConsequenceProfileDto::ProjectionRead => "ProjectionRead",
        ConsequenceProfileDto::OversightMeta => "OversightMeta",
        ConsequenceProfileDto::TopologyMeta => "TopologyMeta",
        ConsequenceProfileDto::EvidenceMeta => "EvidenceMeta",
        ConsequenceProfileDto::OutsideKernel => "OutsideKernel",
    }
}

fn lifecycle_class_token(value: LifecycleClassDto) -> &'static str {
    match value {
        LifecycleClassDto::ExecutionBearing => "execution_bearing",
        LifecycleClassDto::ProjectionOnly => "projection_only",
        LifecycleClassDto::Meta => "meta",
    }
}

fn barrier_policy_token(value: BarrierPolicyDto) -> &'static str {
    match value {
        BarrierPolicyDto::StrictWriteAhead => "strict_write_ahead",
        BarrierPolicyDto::None => "none",
    }
}

fn projection_policy_token(value: ProjectionPolicyDto) -> &'static str {
    match value {
        ProjectionPolicyDto::Anchored => "anchored",
        ProjectionPolicyDto::None => "none",
    }
}

fn truth_commit_policy_token(value: TruthCommitPolicyDto) -> &'static str {
    match value {
        TruthCommitPolicyDto::Allowed => "allowed",
        TruthCommitPolicyDto::Disallowed => "disallowed",
    }
}

fn constraint_mode_token(value: ConstraintPolicyModeDto) -> &'static str {
    match value {
        ConstraintPolicyModeDto::Required => "required",
        ConstraintPolicyModeDto::Disabled => "disabled",
    }
}

fn claim_mode_token(value: ClaimModeDto) -> &'static str {
    match value {
        ClaimModeDto::Exclusive => "exclusive",
        ClaimModeDto::Shared => "shared",
        ClaimModeDto::Token => "token",
    }
}

fn effect_hardness_token(value: EffectHardnessDto) -> &'static str {
    match value {
        EffectHardnessDto::Hard => "hard",
        EffectHardnessDto::Soft => "soft",
        EffectHardnessDto::Meta => "meta",
    }
}

fn event_kind_token(value: AlloyEventKind) -> &'static str {
    match value {
        AlloyEventKind::ActionAdmitted => "action.admitted",
        AlloyEventKind::ActionPlanned => "action.planned",
        AlloyEventKind::DispatchLogged => "dispatch.logged",
        AlloyEventKind::ExecutionBarrierLogged => "execution.barrier_logged",
        AlloyEventKind::ExecutionStarted => "execution.started",
        AlloyEventKind::ExecutionCompleted => "execution.completed",
        AlloyEventKind::ObservedTruthCommitted => "observed_truth.committed",
        AlloyEventKind::ProjectionEmitted => "projection.emitted",
        AlloyEventKind::LifecycleClosed => "lifecycle.closed",
        AlloyEventKind::GateApproved => "gate.approved",
        AlloyEventKind::GateDenied => "gate.denied",
        AlloyEventKind::ConstraintLeaseGranted => "constraint.lease_granted",
        AlloyEventKind::ConstraintLeaseReleased => "constraint.lease_released",
        AlloyEventKind::ViolationDetected => "violation.detected",
        AlloyEventKind::AuthzDecisionRecorded => "authz.decision_recorded",
        AlloyEventKind::DrainFenceRequested => "drain.fence_requested",
        AlloyEventKind::DrainFenceAcquired => "drain.fence_acquired",
    }
}

fn lease_mode_token(value: AlloyLeaseMode) -> &'static str {
    match value {
        AlloyLeaseMode::Exclusive => "exclusive",
        AlloyLeaseMode::Shared => "shared",
        AlloyLeaseMode::Token => "token",
    }
}

fn merge_status_token(value: MergeProtocolStatus) -> &'static str {
    match value {
        MergeProtocolStatus::Absent => "absent",
        MergeProtocolStatus::DeclaredButUnverified => "declared_but_unverified",
        MergeProtocolStatus::Verified => "verified",
        MergeProtocolStatus::Disabled => "disabled",
    }
}

#[cfg(test)]
mod tests {
    use causlane_contracts::{CompiledDispatchBundle, ContractError, RegistryManifest};

    use super::build_formal_ir;

    const REGISTRY: &str =
        include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");

    fn demo_bundle() -> Result<CompiledDispatchBundle, ContractError> {
        let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
        CompiledDispatchBundle::compile(&manifest)
    }

    #[test]
    fn formal_ir_is_deterministic() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let left = build_formal_ir(&bundle, None)?;
        let right = build_formal_ir(&bundle, None)?;
        assert_eq!(left.formal_ir_hash, right.formal_ir_hash);
        assert_eq!(left.to_json_pretty()?, right.to_json_pretty()?);
        Ok(())
    }

    #[test]
    fn formal_ir_hash_changes_when_bundle_changes() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let mut changed = bundle.clone();
        changed.body.bundle_version = "changed-for-test".to_owned();
        let left = build_formal_ir(&bundle, None)?;
        let right = build_formal_ir(&changed, None)?;
        assert_ne!(left.formal_ir_hash, right.formal_ir_hash);
        Ok(())
    }

    #[test]
    fn formal_ir_is_v2_and_projects_merge_status() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let ir = build_formal_ir(&bundle, None)?;
        assert_eq!(ir.schema_version, 2);
        // Merge protocols are projected faithfully from the bundle, carrying the
        // fail-closed status (only `verified` permits concurrency).
        assert_eq!(ir.merge_protocols.len(), bundle.body.merge_protocols.len());
        for (projected, source) in ir.merge_protocols.iter().zip(&bundle.body.merge_protocols) {
            assert_eq!(projected.protocol_id, source.id);
            assert_eq!(
                projected.permits_concurrency,
                source.status.permits_concurrency()
            );
        }
        Ok(())
    }

    #[test]
    fn formal_ir_rejects_unknown_invariant() -> Result<(), Box<dyn std::error::Error>> {
        let mut bundle = demo_bundle()?;
        assert!(!bundle.body.predicates.is_empty());
        if let Some(predicate) = bundle.body.predicates.first_mut() {
            predicate.formal_obligations.push("I-999".to_owned());
        }
        let result = build_formal_ir(&bundle, None);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn formal_ir_rejects_planned_invariant_until_it_is_active(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut bundle = demo_bundle()?;
        assert!(!bundle.body.predicates.is_empty());
        if let Some(predicate) = bundle.body.predicates.first_mut() {
            predicate.formal_obligations.push("I-011".to_owned());
        }

        let result = build_formal_ir(&bundle, None);

        assert!(result.is_err());
        Ok(())
    }
}
