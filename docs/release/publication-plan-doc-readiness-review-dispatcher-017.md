# Publication Plan Documentation Readiness Review — dispatcher-017

**Date:** 2026-06-24
**Snapshot:** `repomix-output-dispatcher-017.txt.gz`
**Status:** `not_ready_for_upload`, `ready_for_publication_docs_patch`

This review records the dispatcher-017 snapshot state. Later commits may have
resolved individual findings; use the live gates in `PUBLISHING.md` and
`docs/release/publication-prep.md` for current upload readiness.

## Summary

The publication documentation is broadly aligned with the intended direction:
refactor first, publish pre-alpha `0.0.1`, keep AI assistance human-owned, keep
`MIT OR Apache-2.0`, publish the whole workspace in dependency order, and keep
`publish-readiness.md` generated.

However, the plan needs correction before it can be used as a release guide.

## What Looks Good

- Root docs exist: `AI_USAGE.md`, `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md`,
  `CHANGELOG.md`, `PUBLISHING.md`, `RELEASE.md`.
- Crate-local READMEs exist for all workspace crates.
- Workspace version is already `0.0.1`.
- License expression and license files are aligned with permissive Rust norms.
- The generated publish-readiness report separates deterministic readiness from
  actual upload execution.
- Product track knows about S11/M11 publication work.

## Findings

### F-001 — Architecture lint failed in the dispatcher-017 snapshot

At the reviewed snapshot, `python3 tools/architecture-lint --json` reported
missing declared binary source files in `crates/causlane-cli/Cargo.toml`:

```text
causlane-formal -> src/bin/causlane-formal.rs
causlane-formal-discipline -> src/bin/causlane-formal-discipline.rs
```

This is a hard publication blocker. Either add the binary source files or remove
the declarations and update README/docs accordingly.

### F-002 — The runbook dry-runs all crates too early

The current runbook suggests running `cargo publish --dry-run` for all crates
before any upload. That can fail for dependent workspace crates because registry
publication resolves internal dependencies from crates.io, not local path-only
state.

Fix: package-list all crates first, then dry-run and publish one crate at a time
in dependency order.

### F-003 — Release strategy mixed internal-only `0.0.x` with public `0.0.1`

`0.0.1` is now intended as a public pre-alpha bootstrap. Product docs should no
longer say all `0.0.x` versions are internal-only.

### F-004 — S11 title and exit gate overstate alpha status

The current publication target is pre-alpha bootstrap, not public alpha API.
S11 should distinguish `0.0.1` bootstrap from later `0.1.x` alpha.

### F-005 — Public API review is a good start but not yet an approval

`docs/api/public-api-review.md` should be treated as a crate-by-crate checklist.
Publication should not imply the checklist is already complete.

### F-006 — Generated readiness docs must stay generated

`docs/release/publish-readiness.md` says it is generated. Patches should update
readiness logic and regenerate it, not hand-edit the Markdown.

## Patch Response

This patch pack updates human-maintained planning docs to:

- make refactor the explicit next action;
- add hard blockers;
- correct staged publication sequence;
- separate `0.0.1` pre-alpha from `0.1.x` alpha;
- strengthen public API and agent documentation;
- add a doc-lint script for publication-plan regressions.

It intentionally does not edit `docs/release/publish-readiness.md` because that
file is generated.
