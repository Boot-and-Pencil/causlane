//! Sensitive-field redaction (M06.7, S06) — the kernel masking MECHANISM only.
//!
//! The security doc states "No projection of sensitive truth without projection
//! authz" but defines no sensitivity taxonomy, classification tiers, or rule
//! grammar. So this module fixes ONLY the masking decision, fail-closed as an
//! **allowlist**: a projection field is revealed iff the host's
//! [`RedactionPolicy`] explicitly lists it as
//! `revealable`; every other field — sensitive, unknown, or merely unlisted — is
//! redacted. The host decides which paths are revealable for an (already
//! authorized) reader; the M07.5 class/profile layer compiles into this same
//! allowlist and never replaces the masking mechanism.
//!
//! [`FieldPath`] is an exact-match token: the kernel applies no normalization
//! (case, whitespace, trailing dots), so canonicalizing payload field paths is a
//! host obligation. Non-formal-bound: no codegen / formal references.

use std::collections::BTreeSet;

/// An opaque, host-defined field path within a projection payload (for example
/// `subject.ssn`). The kernel treats it as an exact-match token and assigns it no
/// meaning beyond set membership.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldPath(pub String);

/// A fail-closed redaction policy for one authorized projection read: the set of
/// field paths this reader may see in clear. Every projection field **not** in
/// `revealable` is redacted, so a path the host forgot to list, or an unknown
/// path, is masked rather than leaked. *Which* paths are revealable is host
/// policy; the kernel fixes only the redact-unless-revealable rule and invents no
/// sensitivity taxonomy. A [`BTreeSet`] (not a `String`-keyed map) keeps lookups
/// typed and the outcome order-independent.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RedactionPolicy {
    /// Field paths this reader may see unredacted. The allowlist; every other
    /// projection field is redacted (fail-closed default).
    pub revealable: BTreeSet<FieldPath>,
}

/// The kernel's per-field masking verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldVisibility {
    /// Explicitly revealable: shown in clear.
    Reveal,
    /// Not revealable: masked (fail-closed default).
    Redact,
}

impl FieldVisibility {
    /// Whether the field is revealed.
    #[must_use]
    pub fn is_revealed(&self) -> bool {
        matches!(self, FieldVisibility::Reveal)
    }
}

/// Decide one field path under the policy, fail-closed: revealed iff explicitly
/// listed in [`RedactionPolicy::revealable`], otherwise redacted.
#[must_use]
pub fn classify_field(policy: &RedactionPolicy, field: &FieldPath) -> FieldVisibility {
    if policy.revealable.contains(field) {
        FieldVisibility::Reveal
    } else {
        FieldVisibility::Redact
    }
}

/// The decided redaction over one projection's field paths: every requested path
/// partitioned into revealed vs. redacted. Order-independent (sets).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RedactionView {
    /// Paths the authorized reader may see in clear.
    pub revealed: BTreeSet<FieldPath>,
    /// Paths masked from the reader (fail-closed: not in `revealable`).
    pub redacted: BTreeSet<FieldPath>,
}

impl RedactionView {
    /// Whether nothing was redacted (a fully clear view).
    #[must_use]
    pub fn is_fully_revealed(&self) -> bool {
        self.redacted.is_empty()
    }
}

/// Partition `fields` under the policy, fail-closed: each path is revealed iff it
/// is listed `revealable`, otherwise redacted. `revealable` entries absent from
/// `fields` are simply unused — no field is fabricated.
#[must_use]
pub fn apply_redaction<'a, I>(policy: &RedactionPolicy, fields: I) -> RedactionView
where
    I: IntoIterator<Item = &'a FieldPath>,
{
    let mut view = RedactionView::default();
    for field in fields {
        match classify_field(policy, field) {
            FieldVisibility::Reveal => {
                view.revealed.insert(field.clone());
            }
            FieldVisibility::Redact => {
                view.redacted.insert(field.clone());
            }
        }
    }
    view
}

#[cfg(test)]
mod tests {
    use super::{
        apply_redaction, classify_field, FieldPath, FieldVisibility, RedactionPolicy, RedactionView,
    };
    use std::collections::BTreeSet;

    fn fp(path: &str) -> FieldPath {
        FieldPath(path.to_owned())
    }

    fn policy(revealable: &[&str]) -> RedactionPolicy {
        RedactionPolicy {
            revealable: revealable.iter().map(|path| fp(path)).collect(),
        }
    }

    fn set(paths: &[&str]) -> BTreeSet<FieldPath> {
        paths.iter().map(|path| fp(path)).collect()
    }

    // Fail-closed default: a field absent from `revealable` is redacted, including
    // an unknown path the host never named (the HIGH-1 witness).
    #[test]
    fn unlisted_field_is_redacted() {
        let p = policy(&["name"]);
        assert_eq!(classify_field(&p, &fp("ssn")), FieldVisibility::Redact);
        assert_eq!(classify_field(&p, &fp("dob")), FieldVisibility::Redact);
    }

    #[test]
    fn revealable_field_is_revealed() {
        let p = policy(&["name"]);
        assert_eq!(classify_field(&p, &fp("name")), FieldVisibility::Reveal);
        assert!(classify_field(&p, &fp("name")).is_revealed());
    }

    #[test]
    fn empty_policy_redacts_all() {
        let p = RedactionPolicy::default();
        let fields = [fp("a"), fp("b")];
        let view = apply_redaction(&p, &fields);
        assert!(view.revealed.is_empty());
        assert_eq!(view.redacted, set(&["a", "b"]));
        assert!(!view.is_fully_revealed());
    }

    // Exact-match tokens: the kernel applies no normalization, so a near-miss path
    // is redacted (canonicalization is a host obligation) — a decision, not an
    // accident.
    #[test]
    fn field_paths_are_exact_match_not_normalized() {
        let p = policy(&["a.b"]);
        assert_eq!(classify_field(&p, &fp("a.b")), FieldVisibility::Reveal);
        assert_eq!(classify_field(&p, &fp("a.b ")), FieldVisibility::Redact);
        assert_eq!(classify_field(&p, &fp("A.B")), FieldVisibility::Redact);
    }

    #[test]
    fn apply_redaction_is_order_independent() {
        let p = policy(&["a", "c"]);
        let forward = [fp("a"), fp("b"), fp("c"), fp("d")];
        let backward = [fp("d"), fp("c"), fp("b"), fp("a")];
        assert_eq!(
            apply_redaction(&p, &forward),
            apply_redaction(&p, &backward)
        );
    }

    /// Load-bearing property: over the field universe {a,b,c,d} and every subset
    /// assigned as `revealable`, `apply_redaction` matches an independent oracle
    /// (revealed = fields ∩ revealable; redacted = fields \ revealable), with
    /// non-vacuity over fully-revealed / partial / fully-redacted.
    #[test]
    fn apply_redaction_matches_oracle() {
        let names = ["a", "b", "c", "d"];
        let universe = [fp("a"), fp("b"), fp("c"), fp("d")];
        let universe_set: BTreeSet<FieldPath> = universe.iter().cloned().collect();
        let mut saw_full_reveal = false;
        let mut saw_partial = false;
        let mut saw_full_redact = false;

        for mask in 0u8..16 {
            let mut revealable: Vec<&str> = Vec::new();
            for (i, name) in names.into_iter().enumerate() {
                if mask & (1u8 << i) != 0 {
                    revealable.push(name);
                }
            }
            let p = policy(&revealable);
            let view = apply_redaction(&p, &universe);

            // Independent oracle: set intersection/difference, not the loop.
            let revealable_set: BTreeSet<FieldPath> =
                revealable.iter().map(|path| fp(path)).collect();
            let oracle = RedactionView {
                revealed: universe_set
                    .intersection(&revealable_set)
                    .cloned()
                    .collect(),
                redacted: universe_set.difference(&revealable_set).cloned().collect(),
            };
            assert_eq!(view, oracle);

            if view.redacted.is_empty() {
                saw_full_reveal = true;
            } else if view.revealed.is_empty() {
                saw_full_redact = true;
            } else {
                saw_partial = true;
            }
        }
        assert!(saw_full_reveal, "fully revealed never observed");
        assert!(saw_partial, "partial redaction never observed");
        assert!(saw_full_redact, "fully redacted never observed");
    }
}
