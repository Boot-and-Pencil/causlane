//! Redaction class profiles (M07.5, S07).
//!
//! M06.7 fixes the kernel masking mechanism: a [`RedactionPolicy`] is a
//! fail-closed allowlist of exact [`FieldPath`] tokens. This module adds the
//! typed policy layer that host observability surfaces can share: fields are
//! classified once, then compiled into the same fail-closed allowlist. It does
//! not perform JSON/value shaping and does not introduce a second masking
//! engine.

use std::collections::{BTreeMap, BTreeSet};

use super::redaction::{FieldPath, RedactionPolicy};

/// Host surface that consumes a redaction profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RedactionSurface {
    /// Audit/event views shown outside the trusted journal authority.
    Audit,
    /// Derived logs.
    Log,
    /// Projection payload reads.
    Projection,
    /// Replay diagnostics.
    Replay,
    /// Support bundles and diagnostic archives.
    SupportBundle,
}

/// Redaction class assigned by the host to one field path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RedactionClass {
    /// Safe for broad disclosure.
    Public,
    /// Operational metadata useful for diagnostics.
    Operational,
    /// Restricted business or tenant-scoped data.
    Restricted,
    /// Secrets, credentials, raw identities, or similarly sensitive material.
    Secret,
}

/// One host-declared field classification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassifiedField {
    /// Exact field path token. No normalization is applied by the kernel.
    pub path: FieldPath,
    /// Host-assigned redaction class for `path`.
    pub class: RedactionClass,
}

impl ClassifiedField {
    /// Create a classified field declaration.
    #[must_use]
    pub fn new(path: FieldPath, class: RedactionClass) -> Self {
        Self { path, class }
    }
}

/// Fail-closed class allowlist: only fields whose effective class declarations
/// are all listed here may compile into [`RedactionPolicy::revealable`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RedactionClassPolicy {
    /// Classes this profile may reveal. Empty means reveal nothing.
    pub revealable_classes: BTreeSet<RedactionClass>,
}

impl RedactionClassPolicy {
    /// Build a class policy from an explicit allowlist.
    #[must_use]
    pub fn revealable<I>(classes: I) -> Self
    where
        I: IntoIterator<Item = RedactionClass>,
    {
        Self {
            revealable_classes: classes.into_iter().collect(),
        }
    }

    /// Common profile: reveal only fields explicitly classified as public.
    #[must_use]
    pub fn public_only() -> Self {
        Self::revealable([RedactionClass::Public])
    }

    /// Whether this class may be revealed by the policy.
    #[must_use]
    pub fn reveals(&self, class: RedactionClass) -> bool {
        self.revealable_classes.contains(&class)
    }
}

/// A complete redaction profile for one host surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceRedactionProfile {
    /// Consumer surface. This tags the profile; it is not a second policy ladder.
    pub surface: RedactionSurface,
    /// Class allowlist for this profile.
    pub policy: RedactionClassPolicy,
    /// Host-declared field classifications.
    pub fields: Vec<ClassifiedField>,
}

impl SurfaceRedactionProfile {
    /// Create a surface profile.
    #[must_use]
    pub fn new(
        surface: RedactionSurface,
        policy: RedactionClassPolicy,
        fields: Vec<ClassifiedField>,
    ) -> Self {
        Self {
            surface,
            policy,
            fields,
        }
    }

    /// Compile this profile into the M06.7 fail-closed allowlist mechanism.
    #[must_use]
    pub fn to_redaction_policy(&self) -> RedactionPolicy {
        compile_redaction_policy(self)
    }
}

/// Compile a typed class profile into the kernel redaction mechanism.
///
/// Duplicate declarations for the same path are handled fail-closed: the path is
/// revealable only if **every** declaration's class is revealable. This prevents
/// an accidental weaker duplicate classification from overriding a stronger one.
#[must_use]
pub fn compile_redaction_policy(profile: &SurfaceRedactionProfile) -> RedactionPolicy {
    let mut per_path: BTreeMap<FieldPath, bool> = BTreeMap::new();

    for field in &profile.fields {
        let revealable = profile.policy.reveals(field.class);
        per_path
            .entry(field.path.clone())
            .and_modify(|existing| *existing &= revealable)
            .or_insert(revealable);
    }

    RedactionPolicy {
        revealable: per_path
            .into_iter()
            .filter_map(|(path, revealable)| revealable.then_some(path))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compile_redaction_policy, ClassifiedField, RedactionClass, RedactionClassPolicy,
        RedactionSurface, SurfaceRedactionProfile,
    };
    use crate::{apply_redaction, FieldPath, RedactionPolicy, RedactionView};
    use std::collections::BTreeSet;

    fn fp(path: &str) -> FieldPath {
        FieldPath(path.to_owned())
    }

    fn cf(path: &str, class: RedactionClass) -> ClassifiedField {
        ClassifiedField::new(fp(path), class)
    }

    fn set(paths: &[&str]) -> BTreeSet<FieldPath> {
        paths.iter().map(|path| fp(path)).collect()
    }

    fn sample_fields() -> Vec<ClassifiedField> {
        vec![
            cf("summary", RedactionClass::Public),
            cf("worker.host", RedactionClass::Operational),
            cf("tenant.plan", RedactionClass::Restricted),
            cf("identity.token", RedactionClass::Secret),
        ]
    }

    #[test]
    fn default_class_policy_reveals_nothing() {
        let profile = SurfaceRedactionProfile::new(
            RedactionSurface::SupportBundle,
            RedactionClassPolicy::default(),
            sample_fields(),
        );
        assert_eq!(profile.to_redaction_policy(), RedactionPolicy::default());
    }

    #[test]
    fn public_only_profile_reveals_only_public_fields() {
        let profile = SurfaceRedactionProfile::new(
            RedactionSurface::Log,
            RedactionClassPolicy::public_only(),
            sample_fields(),
        );
        let fields = [
            fp("summary"),
            fp("worker.host"),
            fp("tenant.plan"),
            fp("identity.token"),
            fp("unclassified.extra"),
        ];

        assert_eq!(
            apply_redaction(&compile_redaction_policy(&profile), &fields),
            RedactionView {
                revealed: set(&["summary"]),
                redacted: set(&[
                    "worker.host",
                    "tenant.plan",
                    "identity.token",
                    "unclassified.extra"
                ]),
            }
        );
    }

    #[test]
    fn compiler_matches_allowlist_oracle_for_all_classes() {
        let classes = [
            RedactionClass::Public,
            RedactionClass::Operational,
            RedactionClass::Restricted,
            RedactionClass::Secret,
        ];

        for mask in 0u8..16 {
            let revealable_classes: Vec<RedactionClass> = classes
                .into_iter()
                .enumerate()
                .filter_map(|(i, class)| (mask & (1u8 << i) != 0).then_some(class))
                .collect();
            let profile = SurfaceRedactionProfile::new(
                RedactionSurface::Projection,
                RedactionClassPolicy::revealable(revealable_classes.clone()),
                sample_fields(),
            );
            let compiled = compile_redaction_policy(&profile);
            let oracle = RedactionPolicy {
                revealable: sample_fields()
                    .into_iter()
                    .filter_map(|field| {
                        revealable_classes
                            .contains(&field.class)
                            .then_some(field.path)
                    })
                    .collect(),
            };
            assert_eq!(compiled, oracle);
        }
    }

    #[test]
    fn every_surface_compiles_through_same_mechanism() {
        let surfaces = [
            RedactionSurface::Audit,
            RedactionSurface::Log,
            RedactionSurface::Projection,
            RedactionSurface::Replay,
            RedactionSurface::SupportBundle,
        ];
        let expected = RedactionPolicy {
            revealable: set(&["summary"]),
        };

        for surface in surfaces {
            let profile = SurfaceRedactionProfile::new(
                surface,
                RedactionClassPolicy::public_only(),
                sample_fields(),
            );
            assert_eq!(compile_redaction_policy(&profile), expected);
        }
    }

    #[test]
    fn duplicate_path_classifications_are_fail_closed() {
        let profile = SurfaceRedactionProfile::new(
            RedactionSurface::SupportBundle,
            RedactionClassPolicy::public_only(),
            vec![
                cf("identity.email", RedactionClass::Public),
                cf("identity.email", RedactionClass::Secret),
            ],
        );

        assert_eq!(
            compile_redaction_policy(&profile),
            RedactionPolicy::default()
        );
    }
}
