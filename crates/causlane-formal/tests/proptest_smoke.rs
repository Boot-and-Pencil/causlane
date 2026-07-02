//! Smoke-level property tests for `causlane-formal`.
//!
//! This is a minimal scaffold whose only job is to exercise the property-test
//! pipeline (proptest) on the CI machine. Real, protocol-meaningful properties
//! are a tracked pre-publication stage — see
//! `docs/release/refactor-before-publication-gate.md` (PUB1: verification/fuzz/property
//! adoption).

use causlane_formal::{FormalProfile, Requirement};
use proptest::prelude::*;

proptest! {
    /// `Requirement::from_tokens` is total (never panics) and deterministic for
    /// arbitrary token input.
    #[test]
    fn from_tokens_total_and_deterministic(tokens in proptest::collection::vec(any::<String>(), 0..16)) {
        let first = Requirement::from_tokens(&tokens);
        let second = Requirement::from_tokens(&tokens);
        prop_assert_eq!(first, second);
    }
}

#[test]
fn every_profile_token_set_parses() {
    for profile in [
        FormalProfile::Custom,
        FormalProfile::Base,
        FormalProfile::Rust,
        FormalProfile::Proof,
        FormalProfile::All,
    ] {
        // Canonical token set for each profile must parse without panicking.
        let _requirement = Requirement::from_tokens(&profile.requirement_tokens());
    }
}
