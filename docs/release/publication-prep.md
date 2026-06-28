# Publication Preparation Plan

**Status:** publication-prep contract, not an upload approval.
**Current next action:** publish `causlane` only after green CI and explicit
maintainer confirmation. `causlane-core 0.0.1`, `causlane-formal 0.0.1`,
`causlane-contracts 0.0.1`, `causlane-runtime 0.0.1`, `causlane-replay 0.0.1`
and `causlane-codegen 0.0.1` have been published and indexed; the staged
`causlane` dry-run passed and is recorded in
`docs/release/pub5-causlane-dry-run.md`.

This document is the human-maintained release plan. The generated readiness
report lives in `docs/release/publish-readiness.md`; do not hand-edit that file.

The authoritative ordering gate is `docs/release/refactor-before-publication-gate.md`.
PUB0-PUB4 are recorded complete there for the current public baseline. If future
changes invalidate the gate evidence, crates.io upload is blocked even when
local metadata/readiness probes are green.

## Scope

The first public publication is a **pre-alpha bootstrap** release of the whole
workspace at `0.0.1`.

It is allowed to publish raw/experimental crates, but only if the package is
honest about maturity and does not contain accidental API, private context,
secrets, broken package declarations or misleading formal/protocol claims.

## Hard Blockers

The publication plan is not ready to execute while any of these are true:

- `tools/architecture-lint --json` reports errors;
- `./tools/cargo-dev deny check` has advisory, license, ban or source errors,
  or warnings that have not been accepted or tracked in release evidence;
- any declared Cargo binary points to a missing source file;
- `cargo package -p <crate> --list` contains private context, temporary patches,
  local checkpoints, generated scratch output or secrets;
- crate-local README/status text overclaims production readiness;
- `PUBLISHING.md` or the release runbook suggests dry-running dependent crates
  before their internal registry dependencies have been published;
- `docs/release/publish-readiness.md` is stale relative to `tools/publish-readiness --write`;
- public repository metadata points to a private, missing or wrong repository;
- secret/context scanning has not been run on the curated public baseline.

## Phase PUB0 — Refactor First

Goal: make the first public source snapshot intentional rather than an agent
checkpoint.

Required work:

- fix repository shape errors, including declared binary/source mismatches;
- remove or document duplicated validation logic;
- keep `causlane-core` pure and runtime-free;
- keep generated truth chain intact: registry/contracts -> compiled bundle ->
  replay/formal/codegen inputs -> receipts;
- keep dependency hygiene explicit: advisory/license/source gates pass, and
  duplicate-version warnings are reviewed rather than hidden;
- remove milestone/stage/patch-pack vocabulary from production identifiers;
- split or document large modules by authority boundary;
- confirm the facade has no accidental broad public re-export.

Exit gate:

```bash
python3 tools/architecture-lint --json | jq -e '.status == "pass"'
python3 tools/semantic-naming-scan --json
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

Repository maintainers may use checked-in wrappers such as `just check`,
`just clippy`, `just test` and `just refactor-readiness`, but public docs must
always provide portable Cargo equivalents.

## Phase PUB1 — Readability And Maintainability

Goal: make the codebase usable by humans and AI agents after publication.

Required work:

- semantic names over historical names;
- module-level docs for authority boundaries;
- stable error codes and diagnostics where already exposed;
- tests named after protocol semantics, not implementation accidents;
- comments that explain invariants and boundaries, not history of construction;
- one canonical owner for each protocol check unless an exception is documented.

Exit gate:

```bash
tools/semantic-naming-scan --json
python3 tools/architecture-lint --json
cargo doc --workspace --no-deps --locked
```

## Phase PUB2 — Public API Review

Goal: make the public surface reusable even though the release is pre-alpha.

Required work:

- review every `pub` item that appears in docs.rs;
- make `causlane` facade intentionally small;
- keep `causlane-core` as pure kernel API;
- keep optional runtime integrations out of default features;
- keep YAML parser dependencies internal to the document-boundary crates and
  do not expose parser error types as public Rust API;
- document unstable and internal surfaces;
- make examples compile against intended imports.

Exit gate:

```bash
cargo doc --workspace --no-deps --locked
cargo test --workspace --doc --locked
```

See `docs/api/public-api-review.md` for the crate-by-crate checklist.

## Phase PUB3 — Human And Agent Documentation

Required root docs:

- `README.md`;
- `AI_USAGE.md`;
- `AGENTS.md`;
- `CONTRIBUTING.md`;
- `SECURITY.md`;
- `CHANGELOG.md`;
- `PUBLISHING.md`;
- `RELEASE.md`;
- `LICENSE-MIT` and `LICENSE-APACHE`.

Required crate docs:

- every published crate has a crate-local README;
- every crate README says experimental/pre-alpha;
- every crate README states what the crate is not;
- every crate README points to the correct public repository.

Agent documentation must explicitly prohibit:

- weakening dispatch invariants;
- manual maintenance of generated formal truth;
- milestone/stage names in production identifiers;
- AI tools as commit authors or co-authors;
- publication without the runbook.

## Phase PUB4 — GitHub Repository Preparation

Recommended strategy: curated public baseline.

Required work:

- create a clean public history or a small curated commit series;
- complete the Gitleaks secret-scan evidence handoff in
  `docs/release/pub4-public-baseline-handoff.md`;
- run context-pack scanning;
- ensure issue/PR templates exist;
- ensure repository URL in Cargo metadata is public before crates.io upload;
- create branch protection before inviting external contributors.

Suggested public commit series:

```text
1. Initial public Causlane architecture and contracts
2. Add semantic kernel and contract crates
3. Add replay and formal/codegen scaffolding
4. Add runtime and CLI scaffolding
5. Add publication, contribution and AI-assistance policy docs
```

## Phase PUB5 — crates.io Full Publication

Publish the full workspace at `0.0.1` only after PUB0-PUB4 gates pass.

Important: do not dry-run all crates as one pre-publish batch. Dependent crates
cannot complete registry dry-run until their internal dependencies have actually
been published and indexed.

Before publishing crates beyond the `causlane-formal` bootstrap upload, rerun
the dependency hygiene gate and review the accepted debt:

- the YAML parser boundary was migrated to `noyalib 0.0.8` with
  `compat-serde-yaml`, which raises the workspace declared MSRV to `1.85`;
- `cargo-deny` duplicate-version warnings are currently treated as convergence
  backlog, not as hidden success.

The `causlane-contracts` dry-run and publication evidence are recorded in
`docs/release/pub5-causlane-contracts-dry-run.md` and
`docs/release/pub5-causlane-contracts-publication.md`.

The `causlane-replay` dry-run evidence records one follow-up: the
`causlane-replay` scoped Rust `1.85.0` MSRV gate passes, but workspace-wide
all-features Rust `1.85.0` compatibility is not claimable until the optional
`causlane-runtime` Restate dependency/MSRV policy is resolved with a versioned
follow-up.

The `causlane-replay` publication evidence is recorded in
`docs/release/pub5-causlane-replay-publication.md`.

The `causlane-codegen` dry-run and publication evidence are recorded in
`docs/release/pub5-causlane-codegen-dry-run.md` and
`docs/release/pub5-causlane-codegen-publication.md`.

The `causlane` dry-run evidence is recorded in
`docs/release/pub5-causlane-dry-run.md`.

Use the staged runbook in `docs/release/publish-all-crates-runbook.md` and
`PUBLISHING.md`.

## Phase PUB6 — Post-publication Stabilization

Immediately after upload:

- tag `v0.0.1`;
- run a downstream smoke project with `cargo add causlane@0.0.1`;
- record published package checksums/versions in release notes;
- update `CHANGELOG.md`;
- create issues for known pre-alpha limitations;
- return to the normal product roadmap.

## Definition Of Done

Publication preparation is complete when:

- architecture/refactor gates pass;
- docs and crate READMEs honestly state pre-alpha status;
- the public API review is recorded;
- the repository history is curated or explicitly accepted;
- package file lists are inspected;
- dependency/advisory/license/source policy is checked and residual warnings
  are recorded;
- the staged publish runbook is followed crate-by-crate;
- generated readiness docs are regenerated, not hand-edited.
