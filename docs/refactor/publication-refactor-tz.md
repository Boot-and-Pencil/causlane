# Publication Refactor Track

## Goal

Prepare Causlane for first public publication without adding new runtime
functionality.

The first public source snapshot should be a deliberate baseline, not an
accidental checkpoint.

## Current Next Action

Run this refactor track before GitHub opening or crates.io upload.

The track is allowed to change structure, names, documentation, package metadata
and internal boundaries. It must not introduce new protocol semantics.

## Work Packages

### PRF-001 — Repository shape

- declared Cargo binaries exist;
- manifests are valid;
- package lists do not contain temporary artifacts;
- ignored files do not hide source that must be published;
- generated readiness reports are regenerated, not hand-edited.

Exit:

```bash
python3 tools/architecture-lint --json | jq -e '.status == "pass"'
```

### PRF-002 — Crate boundaries

- `causlane-core` remains pure;
- contracts, replay, codegen, runtime, CLI and facade crates keep separate responsibilities;
- CLI boundary does not become formal/replay semantic authority;
- adapters do not leak into core.

### PRF-003 — Semantic naming cleanup

Production identifiers must describe semantic role, not construction history.

Prefer:

```text
ExecutionBarrier
ReplayTraceVerifier
ProjectionAnchorViolation
LeaseConflict
```

Avoid production names derived from:

```text
stage names
milestone IDs
patch-pack IDs
agent checkpoint names
snapshot numbers
```

Historical labels may remain in ADRs, roadmap, release notes and formal impact
records when they are explicitly historical context.

### PRF-004 — Duplication and owner modules

Repeated invariant checks should have one owner module or a documented exception.

Examples:

- claim coverage logic should not drift between replay and runtime;
- formal receipt status should have one schema/source of truth;
- helper CLI commands should call shared application services.

### PRF-005 — Generated artifacts

Source formal models may be published. Generated outputs are included only when
they are deliberate examples or package inputs.

Never publish generated scratch outputs or local tool caches.

### PRF-006 — Public API narrowing

- facade exports are intentionally small;
- public glob re-exports are removed or exception-recorded;
- intended imports are documented;
- examples compile against intended imports.

## Acceptance

Portable checks:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
```

Repository-maintainer checks:

```bash
just refactor-readiness
tools/schema-validate-all
tools/publish-readiness --check
```

Formal toolchain checks are separate release gates when the full formal
environment is available.

## Non-goals

Do not use publication refactor to add:

- a workflow engine;
- a new runtime backend;
- new formal proof claims;
- new dispatch semantics;
- a crate rename;
- a hidden API compatibility break without release notes.
