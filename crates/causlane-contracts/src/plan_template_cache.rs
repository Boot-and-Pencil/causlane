//! Pure in-memory cache for plan/template identity.
//!
//! The cache is a memoization layer over [`PlanHashMaterial`]. It does not
//! resolve templates, compile bundles, perform I/O, or define a second hashing
//! path.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use causlane_core::{ImpactSetHash, PlanHash};

use crate::{
    canonical_json_hash, impact_set_hash, is_canonical_sha256_token, ContractError,
    PlanHashMaterial,
};

/// Schema version for [`PlanTemplateCacheKey`].
pub const PLAN_TEMPLATE_CACHE_KEY_SCHEMA_VERSION: u32 = 1;

/// Typed hash of a [`PlanTemplateCacheKey`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlanTemplateCacheKeyHash(String);

impl PlanTemplateCacheKeyHash {
    /// Borrow the canonical `sha256:...` key hash.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable reference to a compile-affecting snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanTemplateSnapshotRef {
    /// Stable snapshot identifier.
    pub snapshot_id: String,
    /// Canonical `sha256:...` content hash of the snapshot.
    pub snapshot_hash: String,
}

impl PlanTemplateSnapshotRef {
    /// Build and validate a snapshot reference.
    ///
    /// # Errors
    /// Returns [`ContractError::Validation`] when the id is empty or the hash is
    /// not a canonical lowercase SHA-256 token.
    #[must_use = "snapshot references must be validated before cache use"]
    pub fn new(
        snapshot_id: impl Into<String>,
        snapshot_hash: impl Into<String>,
    ) -> Result<Self, ContractError> {
        let snapshot = Self {
            snapshot_id: snapshot_id.into(),
            snapshot_hash: snapshot_hash.into(),
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    fn validate(&self) -> Result<(), ContractError> {
        if self.snapshot_id.trim().is_empty() {
            return Err(ContractError::Validation(
                "plan template snapshot id must not be empty".to_owned(),
            ));
        }
        if !is_canonical_sha256_token(&self.snapshot_hash) {
            return Err(ContractError::Validation(format!(
                "plan template snapshot {} has non-canonical hash",
                self.snapshot_id
            )));
        }
        Ok(())
    }
}

/// Canonical cache key for a pure plan template lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanTemplateCacheKey {
    /// Version of this cache-key schema.
    pub key_schema_version: u32,
    /// Canonical plan-hash material.
    pub plan_hash_material: PlanHashMaterial,
    /// Compile-affecting snapshot references, canonicalized by id/hash.
    pub snapshot_refs: Vec<PlanTemplateSnapshotRef>,
}

impl PlanTemplateCacheKey {
    /// Build a canonical cache key.
    ///
    /// # Errors
    /// Returns [`ContractError::Validation`] when a snapshot ref is invalid or
    /// the same snapshot id appears with more than one hash.
    #[must_use = "cache keys must be validated before lookup"]
    pub fn new(
        plan_hash_material: PlanHashMaterial,
        snapshot_refs: impl IntoIterator<Item = PlanTemplateSnapshotRef>,
    ) -> Result<Self, ContractError> {
        Ok(Self {
            key_schema_version: PLAN_TEMPLATE_CACHE_KEY_SCHEMA_VERSION,
            plan_hash_material,
            snapshot_refs: normalize_snapshot_refs(snapshot_refs)?,
        })
    }

    /// Hash this key with the repository's canonical JSON hasher.
    ///
    /// # Errors
    /// Returns [`ContractError::Json`] if canonical serialization fails.
    #[must_use = "cache key hashes must be used for lookups"]
    pub fn key_hash(&self) -> Result<PlanTemplateCacheKeyHash, ContractError> {
        Ok(PlanTemplateCacheKeyHash(canonical_json_hash(self)?))
    }
}

/// Materialized cache entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTemplateCacheEntry {
    /// Canonical hash of the cache key.
    pub key_hash: PlanTemplateCacheKeyHash,
    /// Plan hash computed from [`PlanHashMaterial`].
    pub plan_hash: PlanHash,
    /// Impact-set hash computed from the material's planned impacts.
    pub impact_set_hash: ImpactSetHash,
}

/// Result of a cache lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTemplateCacheLookup {
    /// Materialized entry for the key.
    pub entry: PlanTemplateCacheEntry,
    /// Whether this lookup reused a prior entry.
    pub hit: bool,
}

/// Pure in-memory cache for plan/template identity.
#[derive(Debug, Clone, Default)]
pub struct PlanTemplateCache {
    entries: HashMap<PlanTemplateCacheKeyHash, PlanTemplateCacheEntry>,
}

impl PlanTemplateCache {
    /// Create an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of cached entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return a cached entry, or compute and insert one with the canonical
    /// plan-hash and impact-set helpers.
    ///
    /// # Errors
    /// Returns [`ContractError`] when canonical key hashing, plan hashing, or
    /// impact-set hashing fails.
    #[must_use = "cache lookups report whether the entry was reused"]
    pub fn lookup_or_insert(
        &mut self,
        key: &PlanTemplateCacheKey,
    ) -> Result<PlanTemplateCacheLookup, ContractError> {
        let key_hash = key.key_hash()?;
        if let Some(entry) = self.entries.get(&key_hash) {
            return Ok(PlanTemplateCacheLookup {
                entry: entry.clone(),
                hit: true,
            });
        }

        let entry = PlanTemplateCacheEntry {
            key_hash: key_hash.clone(),
            plan_hash: key.plan_hash_material.compute_plan_hash()?,
            impact_set_hash: impact_set_hash(&key.plan_hash_material.planned_impacts)?,
        };
        let _previous = self.entries.insert(key_hash, entry.clone());
        Ok(PlanTemplateCacheLookup { entry, hit: false })
    }
}

fn normalize_snapshot_refs(
    snapshot_refs: impl IntoIterator<Item = PlanTemplateSnapshotRef>,
) -> Result<Vec<PlanTemplateSnapshotRef>, ContractError> {
    let mut refs = snapshot_refs.into_iter().collect::<Vec<_>>();
    for snapshot in &refs {
        snapshot.validate()?;
    }
    refs.sort_by(|left, right| {
        left.snapshot_id
            .cmp(&right.snapshot_id)
            .then_with(|| left.snapshot_hash.cmp(&right.snapshot_hash))
    });
    refs.dedup();

    if let Some([left, _right]) = refs
        .windows(2)
        .find(|pair| matches!(pair, [left, right] if left.snapshot_id == right.snapshot_id))
    {
        return Err(ContractError::Validation(format!(
            "plan template snapshot id {} appears with multiple hashes",
            left.snapshot_id
        )));
    }
    Ok(refs)
}

#[cfg(test)]
mod tests {
    use crate::{
        examples::release_promote_plan_material, impact_set_hash, ContractError, PlanTemplateCache,
        PlanTemplateCacheKey, PlanTemplateSnapshotRef,
    };

    const BUNDLE_HASH: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SNAPSHOT_HASH: &str =
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const OTHER_HASH: &str =
        "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

    #[test]
    fn same_material_and_snapshots_reuses_entry() -> Result<(), ContractError> {
        let key = key(base_material(), [snapshot("constraint@1", SNAPSHOT_HASH)?])?;
        let mut cache = PlanTemplateCache::new();

        let first = cache.lookup_or_insert(&key)?;
        let second = cache.lookup_or_insert(&key)?;

        assert!(!first.hit);
        assert!(second.hit);
        assert_eq!(first.entry, second.entry);
        assert_eq!(cache.len(), 1);
        Ok(())
    }

    #[test]
    fn entry_matches_direct_plan_and_impact_hashes() -> Result<(), ContractError> {
        let material = base_material();
        let expected_plan = material.compute_plan_hash()?;
        let expected_impact = impact_set_hash(&material.planned_impacts)?;
        let key = key(material, [snapshot("constraint@1", SNAPSHOT_HASH)?])?;

        let entry = PlanTemplateCache::new().lookup_or_insert(&key)?.entry;

        assert_eq!(entry.plan_hash, expected_plan);
        assert_eq!(entry.impact_set_hash, expected_impact);
        assert_eq!(entry.key_hash, key.key_hash()?);
        Ok(())
    }

    #[test]
    fn key_changes_when_plan_identity_fields_change() -> Result<(), ContractError> {
        let base_hash =
            key(base_material(), [snapshot("constraint@1", SNAPSHOT_HASH)?])?.key_hash()?;
        let mut variants = Vec::new();

        let mut bundle_changed = base_material();
        bundle_changed.bundle_hash = OTHER_HASH.to_owned();
        variants.push(("bundle_hash", bundle_changed));

        let mut planner_changed = base_material();
        planner_changed.planner_fingerprint = "changed".to_owned();
        variants.push(("planner_fingerprint", planner_changed));

        let mut predicate_changed = base_material();
        predicate_changed.predicate_version += 1;
        variants.push(("predicate_version", predicate_changed));

        let mut subject_changed = base_material();
        subject_changed.subject_fingerprint = OTHER_HASH.to_owned();
        variants.push(("subject_fingerprint", subject_changed));

        for (field, material) in variants {
            let changed = key(material, [snapshot("constraint@1", SNAPSHOT_HASH)?])?;
            assert_ne!(base_hash, changed.key_hash()?, "{field} should change key");
        }
        Ok(())
    }

    #[test]
    fn key_changes_when_snapshot_hash_changes() -> Result<(), ContractError> {
        let base = key(base_material(), [snapshot("constraint@1", SNAPSHOT_HASH)?])?;
        let changed = key(base_material(), [snapshot("constraint@1", OTHER_HASH)?])?;

        assert_ne!(base.key_hash()?, changed.key_hash()?);
        Ok(())
    }

    #[test]
    fn snapshot_order_is_canonicalized() -> Result<(), ContractError> {
        let left = key(
            base_material(),
            [
                snapshot("z-snapshot", OTHER_HASH)?,
                snapshot("a-snapshot", SNAPSHOT_HASH)?,
            ],
        )?;
        let right = key(
            base_material(),
            [
                snapshot("a-snapshot", SNAPSHOT_HASH)?,
                snapshot("z-snapshot", OTHER_HASH)?,
            ],
        )?;

        assert_eq!(left.snapshot_refs, right.snapshot_refs);
        assert_eq!(left.key_hash()?, right.key_hash()?);
        Ok(())
    }

    #[test]
    fn invalid_snapshot_refs_are_rejected() {
        assert!(matches!(
            PlanTemplateSnapshotRef::new("", SNAPSHOT_HASH),
            Err(ContractError::Validation(_))
        ));
        assert!(matches!(
            PlanTemplateSnapshotRef::new("constraint@1", "sha256:TODO"),
            Err(ContractError::Validation(_))
        ));
    }

    #[test]
    fn duplicate_snapshot_id_with_different_hash_is_rejected() -> Result<(), ContractError> {
        let result = key(
            base_material(),
            [
                snapshot("constraint@1", SNAPSHOT_HASH)?,
                snapshot("constraint@1", OTHER_HASH)?,
            ],
        );

        assert!(matches!(result, Err(ContractError::Validation(_))));
        Ok(())
    }

    fn base_material() -> crate::PlanHashMaterial {
        release_promote_plan_material(BUNDLE_HASH)
    }

    fn snapshot(id: &str, hash: &str) -> Result<PlanTemplateSnapshotRef, ContractError> {
        PlanTemplateSnapshotRef::new(id, hash)
    }

    fn key(
        material: crate::PlanHashMaterial,
        snapshots: impl IntoIterator<Item = PlanTemplateSnapshotRef>,
    ) -> Result<PlanTemplateCacheKey, ContractError> {
        PlanTemplateCacheKey::new(material, snapshots)
    }
}
