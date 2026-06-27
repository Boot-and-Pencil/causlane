//! Registry and compiled bundle contracts.
//!
//! This crate is the *boundary* where stringly-typed registry/bundle documents
//! are parsed into typed values and where content-addressed hashing
//! (`bundle_hash`, `plan_hash`, `impact_set_hash`) is computed. The pure kernel
//! (`causlane-core`) stays free of serialization concerns (ADR-0004, ADR-0014).

#![forbid(unsafe_code)]
#![deny(warnings)]

mod hmac;
pub mod invariants;
mod sha256;

pub mod attestation;
pub mod bundle;
pub mod canonical;
pub mod contract;
pub mod examples;
pub mod plan_hash;
pub mod plan_template_cache;
pub mod registry;
#[doc(hidden)]
pub mod serde_numeric;
pub mod template;

#[cfg(test)]
mod tests;

pub use bundle::{
    merge_decision, resolve_mergeable_scopes, AuthzDecisionSelectorSpec, BundleArtifact,
    CompiledDispatchBundle, CompiledPredicate, EffectTemplateSpec, KernelMergeSemantics,
    LeasePolicySpec, MergeAlgebra, MergeDecision, MergeProtocolSpec, MergeProtocolStatus,
    MergeSemantics,
};
pub use canonical::{
    byte_hash, canonical_json_bytes, canonical_json_hash, CANONICAL_SERIALIZATION_VERSION,
};
pub use contract::{
    BoundaryContracts, BundleCompiler, BundleValidator, CanonicalSerialize, StableDigest,
    TemplateResolver, POLICY_VERSION,
};
pub use invariants::{
    is_active_invariant_id, is_known_invariant_id, is_planned_invariant_id, ACTIVE_INVARIANT_IDS,
    ACTIVE_INVARIANT_RANGE, KNOWN_INVARIANT_RANGE, PLANNED_INVARIANT_IDS,
};
pub use plan_hash::{
    content_hash, impact_set_hash, is_canonical_sha256_token, CanonicalClaim, CanonicalEffect,
    CanonicalImpact, CanonicalOp, PlanHashMaterial,
};
pub use plan_template_cache::{
    PlanTemplateCache, PlanTemplateCacheEntry, PlanTemplateCacheKey, PlanTemplateCacheKeyHash,
    PlanTemplateCacheLookup, PlanTemplateSnapshotRef, PLAN_TEMPLATE_CACHE_KEY_SCHEMA_VERSION,
};
pub use registry::{
    AuthzModeDto, AuthzPolicyManifest, BarrierPolicyDto, ClaimManifest, ClaimModeDto,
    ConsequenceProfileDto, ConstraintPolicyManifest, ConstraintPolicyModeDto, EffectHardnessDto,
    EffectTemplateManifest, LifecycleClassDto, MergeProtocolApplicabilityManifest,
    PredicateManifest, ProjectionPolicyDto, RegistryManifest, RequiredWitnessManifest,
    RouteDerivationDto, SchemaHashesManifest, TruthCommitPolicyDto, WitnessSelectorManifest,
};
pub use template::{
    resolve_template, validate_template_expression, TemplateBinding, TemplateBindings,
    TemplateError, TemplateNamespace,
};

use core::fmt;

/// Errors raised while parsing or compiling contract documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    /// A YAML document failed to parse (message rendered from the YAML parser).
    Yaml(String),
    /// A JSON document failed to (de)serialize (message from `serde_json`).
    Json(String),
    /// A computed or supplied plan hash was not well-formed.
    PlanHash(causlane_core::PlanHashError),
    /// A registry/bundle document parsed but violates contract rules.
    Validation(String),
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::Yaml(msg) => write!(f, "yaml error: {msg}"),
            ContractError::Json(msg) => write!(f, "json error: {msg}"),
            ContractError::PlanHash(err) => write!(f, "invalid plan hash: {err:?}"),
            ContractError::Validation(msg) => write!(f, "validation error: {msg}"),
        }
    }
}

impl std::error::Error for ContractError {}

impl From<noyalib::compat::serde_yaml::Error> for ContractError {
    fn from(err: noyalib::compat::serde_yaml::Error) -> Self {
        ContractError::Yaml(err.to_string())
    }
}

impl From<serde_json::Error> for ContractError {
    fn from(err: serde_json::Error) -> Self {
        ContractError::Json(err.to_string())
    }
}

impl From<causlane_core::PlanHashError> for ContractError {
    fn from(err: causlane_core::PlanHashError) -> Self {
        ContractError::PlanHash(err)
    }
}
