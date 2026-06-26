//! Routing and lifecycle classification derived from consequence profiles.

use core::fmt;

use super::ConsequenceProfile;

/// The lifecycle class a predicate is routed through. Derived from its
/// [`crate::domain::ConsequenceProfile`] at bundle-compile time and carried in
/// the plan hash material (ADR-0009).
///
/// Parsing of the textual registry token into this enum is a *boundary*
/// concern and lives in `causlane-contracts` (via `serde`), not in this pure
/// kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LifecycleClass {
    /// Hard-effect path: admission → … → barrier → execution → observed → closed.
    ExecutionBearing,
    /// Derived-read path: admission → … → projected → closed (no barrier).
    ProjectionOnly,
    /// Oversight/topology/evidence meta path: admission → … → closed.
    Meta,
}

impl LifecycleClass {
    /// The canonical lower-snake token used in registries and hash material.
    #[must_use]
    pub fn as_token(self) -> &'static str {
        match self {
            Self::ExecutionBearing => "execution_bearing",
            Self::ProjectionOnly => "projection_only",
            Self::Meta => "meta",
        }
    }
}

impl fmt::Display for LifecycleClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_token())
    }
}

/// I-005: the lifecycle class a consequence profile routes through. A predicate's
/// route/class is *allowed* only if it equals this class — the route is derived
/// from the consequence profile, not chosen freely (ADR-0005).
#[must_use]
pub fn lifecycle_class_for_profile(profile: ConsequenceProfile) -> LifecycleClass {
    match profile {
        ConsequenceProfile::RuntimeExecution => LifecycleClass::ExecutionBearing,
        ConsequenceProfile::ProjectionRead => LifecycleClass::ProjectionOnly,
        ConsequenceProfile::OversightMeta
        | ConsequenceProfile::TopologyMeta
        | ConsequenceProfile::EvidenceMeta
        | ConsequenceProfile::OutsideKernel => LifecycleClass::Meta,
    }
}

/// I-005: whether a route's lifecycle `class` is allowed for `profile` — it must
/// be exactly the class the profile routes through.
#[must_use]
pub fn route_consistent_with_profile(class: LifecycleClass, profile: ConsequenceProfile) -> bool {
    class == lifecycle_class_for_profile(profile)
}

#[cfg(test)]
mod tests {
    use super::{
        lifecycle_class_for_profile, route_consistent_with_profile, ConsequenceProfile,
        LifecycleClass,
    };

    // I-005: each profile routes through exactly one class; a mismatched class is
    // not allowed.
    #[test]
    fn route_is_allowed_only_for_the_profiles_class() {
        assert_eq!(
            lifecycle_class_for_profile(ConsequenceProfile::RuntimeExecution),
            LifecycleClass::ExecutionBearing
        );
        assert_eq!(
            lifecycle_class_for_profile(ConsequenceProfile::ProjectionRead),
            LifecycleClass::ProjectionOnly
        );
        assert_eq!(
            lifecycle_class_for_profile(ConsequenceProfile::OversightMeta),
            LifecycleClass::Meta
        );
        assert!(route_consistent_with_profile(
            LifecycleClass::ExecutionBearing,
            ConsequenceProfile::RuntimeExecution
        ));
        // A projection-only route under a runtime-execution profile is rejected.
        assert!(!route_consistent_with_profile(
            LifecycleClass::ProjectionOnly,
            ConsequenceProfile::RuntimeExecution
        ));
    }
}
