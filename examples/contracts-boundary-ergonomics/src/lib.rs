#![forbid(unsafe_code)]
#![deny(warnings)]

use std::fmt;

use causlane_contracts::examples::{release_promote_impacts, release_promote_plan_material};
use causlane_contracts::{
    impact_set_hash, is_canonical_sha256_token, BoundaryContracts, BundleCompiler, BundleValidator,
    CanonicalSerialize, CompiledDispatchBundle, ContractError, RegistryManifest, StableDigest,
    TemplateBindings, TemplateError, TemplateResolver, POLICY_VERSION,
};

const REGISTRY_YAML: &str =
    include_str!("../../../contracts/examples/release_promote.registry.yaml");

/// Summary returned by the contracts boundary ergonomics example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractsBoundarySummary {
    /// Registry manifests parsed from YAML.
    pub parsed_manifests: usize,
    /// Bundle compilations verified through the boundary contract trait.
    pub compiled_bundles: usize,
    /// Bundle artifacts serialized and verified on reload.
    pub verified_artifacts: usize,
    /// Hash-critical values checked for canonical `sha256:` shape.
    pub canonical_hashes: usize,
    /// Template expressions resolved through the boundary resolver.
    pub resolved_templates: usize,
    /// Negative controls that failed closed as expected.
    pub negative_controls: usize,
}

/// Error type for the contracts boundary ergonomics example.
#[derive(Debug)]
pub enum ContractsBoundaryError {
    /// Registry, bundle or hash contract failed.
    Contract(ContractError),
    /// Template validation or resolution failed.
    Template(TemplateError),
    /// A hash-critical value was not a canonical `sha256:` token.
    NonCanonicalHash(&'static str),
    /// A bundle artifact accepted after tampering.
    TamperedBundleAccepted,
    /// A template expression accepted without a required binding.
    MissingTemplateAccepted,
    /// A template expression resolved to an unexpected value.
    TemplateResolutionMismatch {
        /// Template case label.
        label: &'static str,
        /// Actual resolved value.
        actual: String,
    },
    /// Mutating plan material did not change the plan hash.
    PlanHashMutationNotDetected,
    /// The compiled bundle roundtrip changed the bundle hash.
    BundleRoundtripMismatch,
    /// Canonical serialization or digest behavior diverged.
    CanonicalDigestMismatch,
}

impl fmt::Display for ContractsBoundaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contract(error) => write!(f, "contract boundary failed: {error}"),
            Self::Template(error) => write!(f, "template boundary failed: {error}"),
            Self::NonCanonicalHash(label) => write!(f, "non-canonical hash token: {label}"),
            Self::TamperedBundleAccepted => f.write_str("tampered bundle artifact was accepted"),
            Self::MissingTemplateAccepted => f.write_str("missing template binding was accepted"),
            Self::TemplateResolutionMismatch { label, actual } => {
                write!(f, "template {label} resolved to unexpected value {actual}")
            }
            Self::PlanHashMutationNotDetected => {
                f.write_str("mutated plan material did not change the plan hash")
            }
            Self::BundleRoundtripMismatch => {
                f.write_str("bundle artifact roundtrip changed the bundle hash")
            }
            Self::CanonicalDigestMismatch => {
                f.write_str("canonical digest did not match canonical bytes")
            }
        }
    }
}

impl std::error::Error for ContractsBoundaryError {}

impl From<ContractError> for ContractsBoundaryError {
    fn from(error: ContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<TemplateError> for ContractsBoundaryError {
    fn from(error: TemplateError) -> Self {
        Self::Template(error)
    }
}

/// Run the contracts boundary ergonomics example.
///
/// # Errors
/// Returns an error if any public contracts boundary operation diverges from the
/// expected deterministic, fail-closed behavior.
#[must_use = "the runnable example result must be checked"]
pub fn run_contracts_boundary() -> Result<ContractsBoundarySummary, ContractsBoundaryError> {
    let boundary = BoundaryContracts;
    let manifest = RegistryManifest::from_yaml_str(REGISTRY_YAML)?;
    boundary.validate_manifest(&manifest)?;
    let bundle = boundary.compile_registry(&manifest)?;

    let verified_artifacts = verify_bundle_artifact_roundtrip(&bundle)?;
    let canonical_hashes = verify_plan_and_impact_hashes(&bundle)?;
    let resolved_templates = verify_template_resolution(&boundary)?;
    verify_canonical_digest(&boundary, &bundle)?;

    let negative_controls = verify_tampered_bundle_is_rejected(&bundle)?
        + verify_missing_template_fails(&boundary)?
        + verify_plan_material_mutation_changes_hash(&bundle)?;

    Ok(ContractsBoundarySummary {
        parsed_manifests: 1,
        compiled_bundles: 1,
        verified_artifacts,
        canonical_hashes,
        resolved_templates,
        negative_controls,
    })
}

/// Verify bundle JSON artifact reload through public APIs.
///
/// # Errors
/// Returns an error if serialization, parsing or hash verification fails.
#[must_use = "bundle artifacts must be verified after serialization"]
pub fn verify_bundle_artifact_roundtrip(
    bundle: &CompiledDispatchBundle,
) -> Result<usize, ContractsBoundaryError> {
    let json = bundle.to_json_pretty()?;
    let reparsed = CompiledDispatchBundle::from_json_str(&json)?;
    if reparsed.bundle_hash != bundle.bundle_hash {
        return Err(ContractsBoundaryError::BundleRoundtripMismatch);
    }
    require_hash("bundle_hash", reparsed.bundle_hash.0.as_str())?;
    Ok(1)
}

/// Verify plan and impact hashing through the public worked-example material.
///
/// # Errors
/// Returns an error if hash computation fails or emits a non-canonical token.
#[must_use = "plan and impact hashes must be inspected"]
pub fn verify_plan_and_impact_hashes(
    bundle: &CompiledDispatchBundle,
) -> Result<usize, ContractsBoundaryError> {
    let material = release_promote_plan_material(bundle.bundle_hash.0.as_str());
    let plan_hash = material.compute_plan_hash()?;
    let impact_hash = impact_set_hash(&release_promote_impacts())?;
    require_hash("plan_hash", plan_hash.as_str())?;
    require_hash("impact_set_hash", impact_hash.0.as_str())?;
    Ok(2)
}

/// Verify exact template resolution through [`BoundaryContracts`].
///
/// # Errors
/// Returns an error if template validation or resolution fails.
#[must_use = "resolved templates must be inspected"]
pub fn verify_template_resolution(
    boundary: &BoundaryContracts,
) -> Result<usize, ContractsBoundaryError> {
    let bindings = template_bindings();
    boundary.validate_expression("environment:${subject.target_environment}")?;
    let env = boundary.resolve_string("environment:${subject.target_environment}", &bindings)?;
    if env != "environment:staging" {
        return Err(ContractsBoundaryError::TemplateResolutionMismatch {
            label: "target_environment",
            actual: env,
        });
    }
    let scope = boundary.resolve_scope(
        "release_candidate:${subject.release_candidate_id}",
        &bindings,
    )?;
    if scope.0 != "release_candidate:rc_123" {
        return Err(ContractsBoundaryError::TemplateResolutionMismatch {
            label: "release_candidate_id",
            actual: scope.0,
        });
    }
    Ok(2)
}

/// Verify canonical serialization and digest are aligned.
///
/// # Errors
/// Returns an error if canonical bytes, JSON or digest disagree.
#[must_use = "canonical digest behavior must be checked"]
pub fn verify_canonical_digest(
    boundary: &BoundaryContracts,
    bundle: &CompiledDispatchBundle,
) -> Result<(), ContractsBoundaryError> {
    if boundary.policy_version() != POLICY_VERSION {
        return Err(ContractsBoundaryError::CanonicalDigestMismatch);
    }
    let value = vec![
        bundle.body.bundle_id.as_str(),
        bundle.body.bundle_version.as_str(),
        bundle.bundle_hash.0.as_str(),
    ];
    let bytes = boundary.canonical_bytes(&value)?;
    let json = boundary.canonical_json(&value)?;
    if bytes != json.as_bytes() {
        return Err(ContractsBoundaryError::CanonicalDigestMismatch);
    }
    let from_value = boundary.digest_value(&value)?;
    let from_bytes = boundary.digest_sha256(&bytes);
    if from_value != from_bytes {
        return Err(ContractsBoundaryError::CanonicalDigestMismatch);
    }
    require_hash("canonical_digest", &from_value)?;
    Ok(())
}

/// Verify stale bundle artifacts fail closed.
///
/// # Errors
/// Returns an error if a tampered artifact is accepted.
#[must_use = "negative controls must be counted"]
pub fn verify_tampered_bundle_is_rejected(
    bundle: &CompiledDispatchBundle,
) -> Result<usize, ContractsBoundaryError> {
    let json = bundle.to_json_pretty()?;
    let tampered = json.replacen(
        "\"bundle_version\": \"0.0.0\"",
        "\"bundle_version\": \"0.0.1\"",
        1,
    );
    match CompiledDispatchBundle::from_json_str(&tampered) {
        Ok(_) => Err(ContractsBoundaryError::TamperedBundleAccepted),
        Err(ContractError::Validation(_)) => Ok(1),
        Err(error) => Err(ContractsBoundaryError::Contract(error)),
    }
}

/// Verify missing template bindings fail closed.
///
/// # Errors
/// Returns an error if a missing template path resolves.
#[must_use = "negative controls must be counted"]
pub fn verify_missing_template_fails(
    boundary: &BoundaryContracts,
) -> Result<usize, ContractsBoundaryError> {
    match boundary.resolve_string("${subject.missing}", &template_bindings()) {
        Ok(_) => Err(ContractsBoundaryError::MissingTemplateAccepted),
        Err(TemplateError::MissingPath { .. }) => Ok(1),
        Err(error) => Err(ContractsBoundaryError::Template(error)),
    }
}

/// Verify plan hashes are sensitive to plan material changes.
///
/// # Errors
/// Returns an error if a material mutation keeps the same plan hash.
#[must_use = "negative controls must be counted"]
pub fn verify_plan_material_mutation_changes_hash(
    bundle: &CompiledDispatchBundle,
) -> Result<usize, ContractsBoundaryError> {
    let material = release_promote_plan_material(bundle.bundle_hash.0.as_str());
    let baseline = material.compute_plan_hash()?;
    let mut mutated = material;
    if let Some(op) = mutated.ops.first_mut() {
        op.kind = "tampered_op_kind".to_owned();
    }
    let changed = mutated.compute_plan_hash()?;
    if baseline == changed {
        return Err(ContractsBoundaryError::PlanHashMutationNotDetected);
    }
    Ok(1)
}

fn template_bindings() -> TemplateBindings {
    TemplateBindings::from_pairs(
        [
            ("release_candidate_id".to_owned(), "rc_123".to_owned()),
            ("target_environment".to_owned(), "staging".to_owned()),
        ],
        [
            ("requested_by".to_owned(), "alice".to_owned()),
            ("reason".to_owned(), "ship it".to_owned()),
        ],
    )
}

fn require_hash(label: &'static str, value: &str) -> Result<(), ContractsBoundaryError> {
    if is_canonical_sha256_token(value) {
        Ok(())
    } else {
        Err(ContractsBoundaryError::NonCanonicalHash(label))
    }
}
