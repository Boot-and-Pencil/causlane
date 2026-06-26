# Formal Impact Record: M11.1 No-Publish Blockers

Date: 2026-06-24

## Summary

This change clears repo-local no-publish readiness blockers for the `causlane`
facade crate while keeping actual crates.io publication deferred. The generated
readiness report now distinguishes local readiness from upload execution.

## Changed Surface

- `crates/causlane/README.md` provides the crate-local README included in the
  facade package.
- `docs/release/publish-readiness.json` moves to schema version 3 and adds the
  `publication_execution` section.
- `docs/release/publish-readiness.md` reports `readiness_status = "pass"` while
  keeping publication execution `deferred`.
- `tools/publish-readiness --check` treats unpublished internal workspace
  dependencies as warnings under the no-publish strategy, not local blockers.

## Coverage Effect

No runtime semantics, replay rules, formal proof obligations or invariant
coverage cells change. This is a release-readiness reporting change only.

## Public API Effect

No Rust public API changes. The facade still re-exports `causlane-core`, and
actual facade upload remains deferred until `causlane-core` is available from
crates.io.

## Negative Controls

No protocol negative control is required. Non-vacuity comes from schema
validation, generated Markdown drift checking, package file-list checking and
manual dry-run probes for `causlane-core` and the facade.

## Validation

- publish-readiness drift check
- publish-readiness schema validation
- facade package file-list check
- product-track status consistency check
- formal discipline check
- standard Rust, schema and release-gate checks
