# Agent Guide For Causlane

This repository is AI-agent-friendly, but agents are not semantic authorities.
The protocol, contracts, generated receipts and maintainer-reviewed docs remain
the source of truth.

## Core Rules

1. Do not weaken dispatch invariants.
2. Do not create a second semantic authority.
3. Do not manually edit generated artifacts unless the file explicitly says it
   is hand-maintained.
4. Do not use milestone or stage names in production identifiers.
5. Do not use AI tools as commit authors or co-authors.
6. Do not publish crates without the release runbook.
7. Do not add direct runtime dependencies to `causlane-core`.
8. Do not add broad public re-exports without an ADR or explicit exception.
9. Do not overclaim formal or replay coverage without receipts.
10. Do not hand-edit generated publication readiness reports.

## Command Policy

Portable documentation should name standard Rust commands first. Repository
maintainers may use local wrappers such as `just` recipes and scripts under
`tools/` when they are present in this checkout, but public documentation must
not make private or agent-local tooling the only documented path.

Direct Cargo invocation may be restricted by local policy in this repository;
when that policy applies, use checked-in wrappers. Public docs must still show
portable Cargo equivalents.

## Naming Policy

Production names should describe semantic role, not project history.

Prefer names like:

```text
ReplayTraceVerifier
ProjectionAnchorViolation
ExecutionBarrier
LeaseTable
```

Avoid production identifiers derived from milestone labels, patch-pack names or
agent snapshot names. Historical labels may remain in roadmap, ADR and formal
impact documents when they are explicitly historical context.

## Generated Truth Chain

Formal artifacts must be generated from compiled bundles and scenarios. Do not
maintain a separate hand-written truth model.

```text
registry/contracts -> compiled bundle -> replay/formal/codegen inputs -> receipts
```

## Publication Rules For Agents

Before proposing publication-related changes, check:

- `docs/release/publication-prep.md`;
- `PUBLISHING.md`;
- `docs/release/publish-all-crates-runbook.md`;
- `docs/api/public-api-review.md`;
- `AI_USAGE.md`.

Never suggest publishing while:

- `architecture-lint` has errors;
- declared Cargo binaries are missing source files;
- package file lists have not been inspected;
- internal dependency order is unresolved;
- the repository URL is not public;
- generated readiness docs are stale.

Do not replace the staged publish sequence with an all-crates dry-run loop.
Dependent crates need their internal registry dependencies published first.
