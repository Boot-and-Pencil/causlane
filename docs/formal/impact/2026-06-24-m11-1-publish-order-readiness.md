# Formal Impact Record: M11.1 Publish Order Readiness

Date: 2026-06-24

## Summary

This change expands the M11.1 publish-readiness gate with a machine-derived
workspace publication order. The report now records dependency tiers, the
facade dependency closure and the facade publish sequence without publishing
crates, reserving names or changing crate semantics.

## Changed Surface

- `docs/release/publish-readiness.json` moves to schema version 2 and adds the
  `workspace_publication` section.
- `docs/release/publish-readiness.md` projects the generated dependency tiers
  and facade publish sequence for release readers.
- `contracts/schema/publish_readiness.schema.json` validates the expanded
  readiness report.
- `tools/publish-readiness --check` fails if the generated order or Markdown
  projection drifts from current `cargo metadata` and `cargo package --list`
  inputs.

## Coverage Effect

No runtime semantics, replay rules, formal proof obligations or invariant
coverage cells change. The publication order is a release-readiness projection
over package metadata, not protocol evidence.

## Public API Effect

No Rust public API changes. The facade still depends on `causlane-core`, and
the report still marks facade publication as blocked until that dependency is
available from crates.io and remaining facade blockers are cleared.

## Negative Controls

No protocol negative control is required. Non-vacuity comes from schema
validation, generated Markdown drift checking, and cycle-aware topological
derivation from workspace normal path dependencies.

## Validation

- publish-readiness drift check
- publish-readiness schema validation
- product-track status consistency check
- formal discipline check
- standard Rust, schema and release-gate checks
