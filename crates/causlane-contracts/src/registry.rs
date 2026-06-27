//! Typed registry manifest — the on-disk `*.registry.yaml` shape.
//!
//! These `*Dto` types own the textual-token mapping (via `serde`) so the pure
//! kernel never branches on raw strings. Each maps to a `causlane-core` value
//! through a `to_core` method that matches on typed variants only.

use serde::{Deserialize, Serialize};

use causlane_core::{ClaimMode, ConsequenceProfile, LifecycleClass};
use noyalib::compat::serde_yaml;

use crate::bundle::MergeProtocolSpec;
use crate::ContractError;

/// A whole predicate registry document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryManifest {
    /// Stable bundle identity (B-005). Part of the bundle hash, so it must be
    /// declared by the registry — not chosen ad hoc by the compiling command.
    pub bundle_id: String,
    /// Registry schema/content version (free-form string, e.g. `0.0.0`).
    pub registry_version: String,
    /// The predicates declared by this registry.
    pub predicates: Vec<PredicateManifest>,
    /// Bundle-level merge protocols (ADR-0012). Compiled into the bundle so the
    /// formal layer can tell which overlapping mutable writes are permitted by a
    /// `Verified` protocol (P0-FM-009). Default: none.
    #[serde(default)]
    pub merge_protocols: Vec<MergeProtocolSpec>,
}

impl RegistryManifest {
    /// Parse a registry manifest from a YAML string.
    ///
    /// # Errors
    /// Returns [`ContractError::Yaml`] if the document is not valid YAML or does
    /// not match the manifest schema.
    #[must_use = "the parsed manifest must be used"]
    pub fn from_yaml_str(yaml: &str) -> Result<Self, ContractError> {
        let manifest = serde_yaml::from_str(yaml)?;
        Ok(manifest)
    }
}

/// One predicate entry in a registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateManifest {
    /// Canonical predicate id, e.g. `release.promote_candidate`.
    pub id: String,
    /// Predicate schema version.
    pub version: u32,
    /// Consequence classification driving route/lifecycle/barrier obligations.
    pub consequence_profile: ConsequenceProfileDto,
    /// Reference to the subject JSON schema.
    pub subject_schema_ref: String,
    /// Reference to the circumstance JSON schema.
    pub circumstance_schema_ref: String,
    /// Lifecycle class derived from the consequence profile.
    pub lifecycle_class: LifecycleClassDto,
    /// Stable route chosen for this predicate.
    pub route_id: String,
    /// How the route is derived and audited.
    pub route_derivation: RouteDerivationDto,
    /// Barrier policy required before execution-bearing work starts.
    pub barrier_policy: BarrierPolicyDto,
    /// Projection anchoring policy.
    pub projection_policy: ProjectionPolicyDto,
    /// Authorization policy. Absence is not allowed; deny-by-default must be
    /// represented either by `required` or an explicit local-dev exemption.
    pub authz_policy: AuthzPolicyManifest,
    /// Whether this predicate may commit observed truth.
    pub truth_commit_policy: TruthCommitPolicyDto,
    /// Hashes of schemas consumed by generated formal facts.
    pub schema_hashes: SchemaHashesManifest,
    /// Constraint/lease policy for this predicate.
    pub constraint_policy: ConstraintPolicyManifest,
    /// Formal invariant ids this predicate contributes evidence for.
    #[serde(default)]
    pub formal_obligations: Vec<String>,
    /// Merge protocols applicable to effects of this predicate.
    #[serde(default)]
    pub merge_protocol_applicability: Vec<MergeProtocolApplicabilityManifest>,
    /// Witnesses required before a stage transition (default: none).
    #[serde(default)]
    pub required_witnesses: Vec<RequiredWitnessManifest>,
    /// Resource claims this predicate needs (default: none).
    #[serde(default)]
    pub claims: Vec<ClaimManifest>,
    /// Effect templates describing the reads/writes/produces/requires and
    /// conflict domains of this predicate's operations (P0-FM-009). Compiled
    /// into the bundle and projected into the Formal IR. Default: none.
    #[serde(default)]
    pub effect_templates: Vec<EffectTemplateManifest>,
    /// Scenario files that exercise this predicate in the formal-ready matrix.
    #[serde(default)]
    pub scenario_refs: Vec<String>,
}

/// Hardness classification of an effect (drives conflict/merge semantics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectHardnessDto {
    /// Externally-visible, non-trivially-reversible effect.
    Hard,
    /// Reversible/compensatable effect.
    Soft,
    /// Kernel-internal/meta effect with no external consequence.
    Meta,
}

/// An effect template declaration on a predicate (P0-FM-009).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectTemplateManifest {
    /// Operation kind or selector this template applies to.
    pub op_kind: String,
    /// Read scope expressions (may contain `${subject.*}`/`${circumstance.*}`).
    #[serde(default)]
    pub reads: Vec<String>,
    /// Write scope expressions (may contain templates).
    #[serde(default)]
    pub writes: Vec<String>,
    /// Produced fact kinds.
    #[serde(default)]
    pub produces: Vec<String>,
    /// Required fact kinds.
    #[serde(default)]
    pub requires: Vec<String>,
    /// Conflict-domain scope expressions used by I-006/I-007 reasoning.
    #[serde(default)]
    pub conflict_domains: Vec<String>,
    /// Hardness classification.
    pub hardness: EffectHardnessDto,
    /// Idempotency-domain expression, if the op is idempotent.
    #[serde(default)]
    pub idempotency_domain: Option<String>,
}

/// How a predicate route was derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteDerivationDto {
    /// Route is directly derived from the consequence profile.
    ConsequenceProfile,
    /// Route is explicitly declared after review.
    ExplicitReviewed,
}

/// Barrier policy tokens carried by the bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BarrierPolicyDto {
    /// Strict write-ahead execution barrier is required.
    StrictWriteAhead,
    /// No execution barrier is required.
    None,
}

/// Projection anchoring policy tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionPolicyDto {
    /// Every emitted projection must anchor to observed truth.
    Anchored,
    /// Projection anchoring is not required.
    None,
}

/// Authorization policy for a predicate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthzPolicyManifest {
    /// Authorization mode.
    pub mode: AuthzModeDto,
    /// Profiles for which an explicit local-dev exemption is allowed.
    #[serde(default)]
    pub allowed_in_profiles: Vec<ConsequenceProfileDto>,
    /// Required stages, when `mode = required`.
    #[serde(default)]
    pub stages: Vec<String>,
    /// Expected policy id every authz decision must be issued under, when
    /// `mode = required` (P0-010). A decision that authorizes the right
    /// action/plan/stage but under a different policy is rejected.
    #[serde(default)]
    pub policy_id: String,
    /// Expected policy version every authz decision must carry, when
    /// `mode = required` (P0-010).
    #[serde(default)]
    pub policy_version: String,
    /// Maximum age, in the same time unit as decision timestamps, a decision may
    /// have at the barrier's evaluation time (ADR-0011 "fresh"). A decision older
    /// than this is stale and rejected even if it has not yet expired. `None`
    /// imposes no freshness bound beyond the decision's own `expires_at`.
    #[serde(
        default,
        deserialize_with = "crate::serde_numeric::deserialize_option_u64_lossless"
    )]
    pub freshness_max_age: Option<u64>,
    /// Human-readable rationale for local-dev exemptions.
    #[serde(default)]
    pub rationale: String,
}

/// Authorization policy mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthzModeDto {
    /// A policy decision is required at configured stages.
    Required,
    /// Explicit local-only exemption for fixtures/demo profiles.
    DisabledForLocalDev,
}

/// Truth commit policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruthCommitPolicyDto {
    /// Predicate may commit observed truth after execution.
    Allowed,
    /// Predicate must not commit observed truth.
    Disallowed,
}

/// Schema hashes that bind generated formal facts to a concrete schema version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaHashesManifest {
    /// Subject schema hash (`sha256:...`).
    pub subject: String,
    /// Circumstance schema hash (`sha256:...`).
    pub circumstance: String,
}

/// Constraint/lease policy for a predicate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstraintPolicyManifest {
    /// Whether declared claims must be covered by leases.
    pub claim_coverage: ConstraintPolicyModeDto,
    /// How lease conflicts are detected.
    pub lease_conflicts: ConstraintPolicyModeDto,
}

/// Constraint policy mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintPolicyModeDto {
    /// The check is required.
    Required,
    /// The check is explicitly disabled.
    Disabled,
}

/// Applicability declaration for a merge protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeProtocolApplicabilityManifest {
    /// Merge protocol id.
    pub protocol_id: String,
    /// Operation kind or glob-like token the protocol applies to.
    pub applies_to: String,
}

/// A required-witness declaration on a predicate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredWitnessManifest {
    /// Stable id of the requirement, e.g. `readiness_before_promotion`.
    pub id: String,
    /// Lifecycle stage at which the witness must already exist.
    pub target_stage: String,
    /// Selector that resolves the witness against the audit journal.
    pub selector: WitnessSelectorManifest,
}

/// Selector describing which audit fact satisfies a required witness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WitnessSelectorManifest {
    /// Audit event kind that carries the fact (dotted token form).
    pub event_kind: String,
    /// Predicate that must have produced the fact.
    pub predicate: String,
    /// Fact kind the witness asserts.
    pub fact_kind: String,
    /// Scope expression (may contain `${subject.*}` template references).
    pub scope_expr: String,
}

/// A resource-claim declaration on a predicate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimManifest {
    /// Resource being claimed, e.g. `environment_write`.
    pub resource: String,
    /// Scope expression for the claim (may contain `${subject.*}`).
    pub scope_expr: String,
    /// Claim mode (shared / exclusive / token).
    pub mode: ClaimModeDto,
}

/// Boundary form of [`ConsequenceProfile`]; variant names match the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsequenceProfileDto {
    /// Hard-effect runtime execution.
    RuntimeExecution,
    /// Derived read; must not commit truth.
    ProjectionRead,
    /// Approval / review / escalation meta.
    OversightMeta,
    /// Scheduling / lease / drain meta.
    TopologyMeta,
    /// Witness / proof / replay-binding meta.
    EvidenceMeta,
    /// Explicitly outside-kernel behaviour.
    OutsideKernel,
}

impl ConsequenceProfileDto {
    /// Convert to the pure-kernel enum.
    #[must_use]
    pub fn to_core(self) -> ConsequenceProfile {
        match self {
            ConsequenceProfileDto::RuntimeExecution => ConsequenceProfile::RuntimeExecution,
            ConsequenceProfileDto::ProjectionRead => ConsequenceProfile::ProjectionRead,
            ConsequenceProfileDto::OversightMeta => ConsequenceProfile::OversightMeta,
            ConsequenceProfileDto::TopologyMeta => ConsequenceProfile::TopologyMeta,
            ConsequenceProfileDto::EvidenceMeta => ConsequenceProfile::EvidenceMeta,
            ConsequenceProfileDto::OutsideKernel => ConsequenceProfile::OutsideKernel,
        }
    }
}

/// Boundary form of [`LifecycleClass`]; tokens are lower-snake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleClassDto {
    /// Hard-effect, execution-bearing path.
    ExecutionBearing,
    /// Projection-only path.
    ProjectionOnly,
    /// Oversight/topology/evidence meta path.
    Meta,
}

impl LifecycleClassDto {
    /// Convert to the pure-kernel enum.
    #[must_use]
    pub fn to_core(self) -> LifecycleClass {
        match self {
            LifecycleClassDto::ExecutionBearing => LifecycleClass::ExecutionBearing,
            LifecycleClassDto::ProjectionOnly => LifecycleClass::ProjectionOnly,
            LifecycleClassDto::Meta => LifecycleClass::Meta,
        }
    }
}

/// Boundary form of [`ClaimMode`]; tokens are lower-snake (`exclusive`, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimModeDto {
    /// Exclusive write access (maps to [`ClaimMode::ExclusiveWrite`]).
    Exclusive,
    /// Shared read access (maps to [`ClaimMode::SharedRead`]).
    Shared,
    /// Token / quota acquisition.
    Token,
}

impl ClaimModeDto {
    /// Convert to the pure-kernel enum.
    #[must_use]
    pub fn to_core(self) -> ClaimMode {
        match self {
            ClaimModeDto::Exclusive => ClaimMode::ExclusiveWrite,
            ClaimModeDto::Shared => ClaimMode::SharedRead,
            ClaimModeDto::Token => ClaimMode::Token,
        }
    }
}
