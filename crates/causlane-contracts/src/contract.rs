//! Boundary contract surface (§7.1–7.3).
//!
//! Names the contracts crate's pure boundary logic — canonical serialization &
//! hashing (§7.1), bundle compilation/validation (§7.2) and template resolution
//! (§7.3) — as explicit, testable traits with one canonical authority each.
//! Every impl delegates to the existing pure functions in [`crate::canonical`],
//! [`crate::plan_hash`], [`crate::bundle`] and [`crate::template`]; nothing here
//! re-implements logic, so there is a single source of truth.

use serde::Serialize;

use causlane_core::Scope;

use crate::bundle::CompiledDispatchBundle;
use crate::canonical::{byte_hash, canonical_json_bytes};
use crate::registry::RegistryManifest;
use crate::template::{
    resolve_template, validate_template_expression, TemplateBindings, TemplateError,
};
use crate::ContractError;

/// The single canonical serialization & hashing policy version (§7.1). Bumped
/// only when the canonical byte form changes, so digests from different policy
/// versions are never conflated (ADR-0009).
pub const POLICY_VERSION: u32 = crate::canonical::CANONICAL_SERIALIZATION_VERSION;

/// §7.1 — canonical serialization of hash-critical material. Implementers emit
/// compact, deterministic UTF-8 JSON (sorted containers, fixed field order) so
/// that the same value always hashes to the same digest.
pub trait CanonicalSerialize {
    /// Serialize a value to canonical bytes suitable for hashing.
    ///
    /// # Errors
    /// Returns [`ContractError::Json`] if serialization fails.
    fn canonical_bytes<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, ContractError>;

    /// Serialize a value to a canonical UTF-8 JSON string.
    ///
    /// # Errors
    /// Returns [`ContractError::Json`] if serialization fails or the bytes are
    /// not valid UTF-8 (serde JSON output always is, so this is defensive).
    fn canonical_json<T: Serialize>(&self, value: &T) -> Result<String, ContractError>;

    /// The canonical serialization policy version in effect.
    fn policy_version(&self) -> u32;
}

/// §7.1 — stable content hashing over canonical bytes.
pub trait StableDigest {
    /// Compute the `sha256:<hex>` digest of a byte buffer.
    fn digest_sha256(&self, bytes: &[u8]) -> String;

    /// Serialize a value canonically and digest it in one step.
    ///
    /// # Errors
    /// Returns [`ContractError::Json`] if serialization fails.
    fn digest_value<T: Serialize>(&self, value: &T) -> Result<String, ContractError>;
}

/// §7.2 — compile a registry manifest into a content-addressed bundle.
pub trait BundleCompiler {
    /// Compile a manifest into a [`CompiledDispatchBundle`].
    ///
    /// # Errors
    /// Returns [`ContractError`] if the manifest is invalid or canonical
    /// serialization fails.
    fn compile_registry(
        &self,
        manifest: &RegistryManifest,
    ) -> Result<CompiledDispatchBundle, ContractError>;
}

/// §7.2 — validate a registry manifest. Validation is defined as "compiles
/// cleanly": the compiler performs the manifest validation, so a manifest is
/// valid iff [`CompiledDispatchBundle::compile`] accepts it. This keeps one
/// validation authority rather than a parallel copy.
pub trait BundleValidator {
    /// Validate a manifest by compiling it and discarding the bundle.
    ///
    /// # Errors
    /// Returns the same [`ContractError`] the compiler would on an invalid
    /// manifest.
    fn validate_manifest(&self, manifest: &RegistryManifest) -> Result<(), ContractError>;
}

/// §7.3 — resolve `${subject.*}` / `${circumstance.*}` template expressions.
/// Exact-only: missing path, empty value, unknown namespace and malformed
/// `${...` all fail.
pub trait TemplateResolver {
    /// Resolve an expression to a string.
    ///
    /// # Errors
    /// Returns [`TemplateError`] for malformed expressions, unknown namespaces,
    /// missing paths or empty values.
    fn resolve_string(
        &self,
        expr: &str,
        bindings: &TemplateBindings,
    ) -> Result<String, TemplateError>;

    /// Resolve an expression and wrap it as a [`Scope`].
    ///
    /// # Errors
    /// As [`TemplateResolver::resolve_string`].
    fn resolve_scope(
        &self,
        expr: &str,
        bindings: &TemplateBindings,
    ) -> Result<Scope, TemplateError>;

    /// Validate an expression's shape without resolving it.
    ///
    /// # Errors
    /// Returns [`TemplateError`] for malformed expressions.
    fn validate_expression(&self, expr: &str) -> Result<(), TemplateError>;
}

/// The canonical authority implementing every boundary contract above by
/// delegating to the crate's pure functions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BoundaryContracts;

impl CanonicalSerialize for BoundaryContracts {
    fn canonical_bytes<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, ContractError> {
        canonical_json_bytes(value)
    }

    fn canonical_json<T: Serialize>(&self, value: &T) -> Result<String, ContractError> {
        let bytes = canonical_json_bytes(value)?;
        String::from_utf8(bytes).map_err(|err| ContractError::Json(err.to_string()))
    }

    fn policy_version(&self) -> u32 {
        POLICY_VERSION
    }
}

impl StableDigest for BoundaryContracts {
    fn digest_sha256(&self, bytes: &[u8]) -> String {
        byte_hash(bytes)
    }

    fn digest_value<T: Serialize>(&self, value: &T) -> Result<String, ContractError> {
        Ok(byte_hash(&canonical_json_bytes(value)?))
    }
}

impl BundleCompiler for BoundaryContracts {
    fn compile_registry(
        &self,
        manifest: &RegistryManifest,
    ) -> Result<CompiledDispatchBundle, ContractError> {
        CompiledDispatchBundle::compile(manifest)
    }
}

impl BundleValidator for BoundaryContracts {
    fn validate_manifest(&self, manifest: &RegistryManifest) -> Result<(), ContractError> {
        CompiledDispatchBundle::compile(manifest).map(|_bundle| ())
    }
}

impl TemplateResolver for BoundaryContracts {
    fn resolve_string(
        &self,
        expr: &str,
        bindings: &TemplateBindings,
    ) -> Result<String, TemplateError> {
        resolve_template(expr, bindings)
    }

    fn resolve_scope(
        &self,
        expr: &str,
        bindings: &TemplateBindings,
    ) -> Result<Scope, TemplateError> {
        resolve_template(expr, bindings).map(Scope)
    }

    fn validate_expression(&self, expr: &str) -> Result<(), TemplateError> {
        validate_template_expression(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BoundaryContracts, BundleCompiler, BundleValidator, CanonicalSerialize, StableDigest,
        TemplateResolver, POLICY_VERSION,
    };
    use crate::registry::RegistryManifest;
    use crate::template::{TemplateBindings, TemplateError};
    use crate::ContractError;

    const REGISTRY: &str =
        include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");

    fn bindings() -> TemplateBindings {
        TemplateBindings::from_pairs(
            [("release_candidate_id".to_owned(), "rc_123".to_owned())],
            [("requested_by".to_owned(), "alice".to_owned())],
        )
    }

    // §7.1: canonical bytes are deterministic and the digest is well-formed.
    #[test]
    fn canonical_serialize_is_deterministic_and_digested() -> Result<(), ContractError> {
        let k = BoundaryContracts;
        let value = vec!["b", "a", "c"];
        let first = k.canonical_bytes(&value)?;
        let second = k.canonical_bytes(&value)?;
        assert_eq!(first, second);
        assert_eq!(k.canonical_json(&value)?, "[\"b\",\"a\",\"c\"]");
        assert_eq!(k.policy_version(), POLICY_VERSION);
        let digest = k.digest_value(&value)?;
        assert!(digest.starts_with("sha256:"));
        assert_eq!(digest, k.digest_sha256(&first));
        Ok(())
    }

    // §7.2: the example manifest compiles and validates; validation is "compiles".
    #[test]
    fn bundle_compiler_and_validator_agree() -> Result<(), ContractError> {
        let k = BoundaryContracts;
        let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
        let compiled = k.compile_registry(&manifest)?;
        assert!(!compiled.body.predicates.is_empty());
        assert!(k.validate_manifest(&manifest).is_ok());
        Ok(())
    }

    // §7.3: exact resolution; malformed / unknown-namespace / missing-path fail.
    #[test]
    fn template_resolver_is_exact_and_fail_closed() -> Result<(), TemplateError> {
        let k = BoundaryContracts;
        let binds = bindings();
        assert_eq!(
            k.resolve_string("${subject.release_candidate_id}", &binds)?,
            "rc_123"
        );
        assert_eq!(
            k.resolve_scope("${circumstance.requested_by}", &binds)?.0,
            "alice"
        );
        // Malformed expression (no closing brace).
        assert!(k.resolve_string("${subject.x", &binds).is_err());
        assert!(k.validate_expression("${subject.x").is_err());
        // Unknown namespace.
        assert!(k.resolve_string("${unknown.x}", &binds).is_err());
        // Missing path.
        assert!(k.resolve_string("${subject.nope}", &binds).is_err());
        Ok(())
    }
}
