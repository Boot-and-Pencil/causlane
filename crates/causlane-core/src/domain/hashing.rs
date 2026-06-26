//! Content-addressed hash newtypes used across the contract surface.
//!
//! These are deliberately opaque string wrappers in the pure kernel: the
//! actual digest computation (SHA-256 over a canonical serialization) is a
//! boundary concern and lives in `causlane-contracts` (ADR-0009, ADR-0014).

use core::fmt;

/// Hash of a compiled dispatch bundle (`sha256:...`). Feeds plan hash material.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BundleHash(pub String);

/// Hash of an opaque content blob, e.g. a subject or circumstance snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContentHash(pub String);

/// Hash of the canonical set of planned impacts. Approvals bind to this so they
/// track the set of hard consequences rather than every plan detail (I-009).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImpactSetHash(pub String);

/// Hash of a single canonicalized audit event, used to pin truth anchors.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EventHash(pub String);

macro_rules! impl_display {
    ($($ty:ty),* $(,)?) => {
        $(impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        })*
    };
}

impl_display!(BundleHash, ContentHash, ImpactSetHash, EventHash);
