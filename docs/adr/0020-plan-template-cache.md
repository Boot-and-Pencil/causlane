# ADR-0020: Plan/template cache is pure memoization over canonical material

## Status

Accepted.

## Context

M09.5 needs plan/template reuse without weakening ADR-0009 plan-hash
canonicalization or introducing another template-resolution authority. The
existing contract layer already owns canonical JSON hashing, plan-hash material
and impact-set hashing.

## Decision

`causlane-contracts` exposes `PlanTemplateCache` as an in-memory memoization
layer over:

```text
PlanHashMaterial + PlanTemplateSnapshotRef[] -> PlanTemplateCacheKeyHash
```

`PlanTemplateCacheKey` is canonicalized and hashed with the existing
`canonical_json_hash` helper. `PlanTemplateSnapshotRef` requires a non-empty
snapshot id and a canonical lowercase `sha256:` snapshot hash. Snapshot refs are
sorted by id/hash, exact duplicates collapse, and one snapshot id cannot appear
with multiple hashes.

The cache entry computes:

- `plan_hash` via `PlanHashMaterial::compute_plan_hash`;
- `impact_set_hash` via `impact_set_hash(material.planned_impacts)`.

## Consequences

- Cache hits cannot bypass canonical plan/impact hashing.
- Snapshot refs prevent stale cache reuse across compile-affecting inputs.
- Snapshot refs are cache-key inputs only; they do not silently extend
  `PlanHashMaterial`. If a snapshot affects compiled plan identity, the planner
  must encode that in the material according to ADR-0009.
- No runtime LRU/TTL policy, durable cache, distributed cache, generated schema,
  scenario, Formal IR change or new template resolver is introduced in M09.5.
