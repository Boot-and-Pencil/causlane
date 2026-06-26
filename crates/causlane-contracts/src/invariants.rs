//! Stable formal invariant id catalog.
//!
//! `ACTIVE_INVARIANT_IDS` are the only ids that may appear in compiled bundle
//! formal obligations and Formal IR today. `PLANNED_INVARIANT_IDS` are reserved
//! for future planning/manifests and must not receive coverage credit until a
//! concrete lane carries checks for them.

/// Stable active invariant id range label for diagnostics.
pub const ACTIVE_INVARIANT_RANGE: &str = "I-001..I-010";

/// Stable known invariant id range label for diagnostics and planning surfaces.
pub const KNOWN_INVARIANT_RANGE: &str = "I-001..I-020";

/// Invariant ids with active generated/replay/proof coverage rows.
pub const ACTIVE_INVARIANT_IDS: &[&str] = &[
    "I-001", "I-002", "I-003", "I-004", "I-005", "I-006", "I-007", "I-008", "I-009", "I-010",
];

/// Invariant ids reserved for expansion planning.
pub const PLANNED_INVARIANT_IDS: &[&str] = &[
    "I-011", "I-012", "I-013", "I-014", "I-015", "I-016", "I-017", "I-018", "I-019", "I-020",
];

/// Return true when `id` names an invariant with active coverage rows.
#[must_use]
pub fn is_active_invariant_id(id: &str) -> bool {
    ACTIVE_INVARIANT_IDS.contains(&id)
}

/// Return true when `id` names a planned invariant reservation.
#[must_use]
pub fn is_planned_invariant_id(id: &str) -> bool {
    PLANNED_INVARIANT_IDS.contains(&id)
}

/// Return true when `id` is either active or reserved for planned expansion.
#[must_use]
pub fn is_known_invariant_id(id: &str) -> bool {
    is_active_invariant_id(id) || is_planned_invariant_id(id)
}

#[cfg(test)]
mod tests {
    use super::{
        is_active_invariant_id, is_known_invariant_id, is_planned_invariant_id,
        ACTIVE_INVARIANT_IDS, PLANNED_INVARIANT_IDS,
    };

    #[test]
    fn invariant_catalog_separates_active_from_planned_ids() {
        assert_eq!(ACTIVE_INVARIANT_IDS.len(), 10);
        assert_eq!(PLANNED_INVARIANT_IDS.len(), 10);

        assert!(is_active_invariant_id("I-001"));
        assert!(is_active_invariant_id("I-010"));
        assert!(!is_active_invariant_id("I-011"));

        assert!(is_planned_invariant_id("I-011"));
        assert!(is_planned_invariant_id("I-020"));
        assert!(!is_planned_invariant_id("I-010"));

        assert!(is_known_invariant_id("I-001"));
        assert!(is_known_invariant_id("I-020"));
        assert!(!is_known_invariant_id("I-021"));
        assert!(!is_known_invariant_id("I-999"));
    }
}
