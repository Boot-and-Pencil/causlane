# Contributing To Causlane

Causlane is experimental and pre-alpha. Contributions are welcome, but the
project prioritizes semantic clarity, replayability, formal/readiness discipline
and small kernel boundaries over feature velocity.

## Before Contributing

Start with these files:

- `README.md`
- `AI_USAGE.md`
- `AGENTS.md`
- `docs/04-development-principles.md`
- `docs/product-track/12-milestone-execution-runbook.md`
- `docs/adr/`

This file is a routing guide. Protocol contracts, compiled bundles, generated
receipts, formal artifacts and maintainer-reviewed ADRs remain the source of
truth.

## Contribution Workflow

Classify the change before writing code:

| Change | Examples | Required evidence |
|---|---|---|
| Docs-only | wording, links, examples prose | docs/status checks |
| Tooling or docs process | runbooks, non-authoritative scripts | product-track/status checks |
| Protocol-critical | bundle, replay, lifecycle, authz, barrier, leases | scenario/replay evidence, formal impact record, negative control |
| Runtime or adapter authority | executor, audit, capabilities, backend adapters | protocol evidence plus runtime or certification tests |
| Public API or release | exported Rust API, semver, publication docs | compatibility/release evidence |

For milestone work, follow
`docs/product-track/12-milestone-execution-runbook.md`. For protocol-critical
work, add or identify the evidence surface before implementation.

## Development Rules

- Keep `causlane-core` pure: no async runtime, database, HTTP, workflow engine,
  policy engine or telemetry dependency.
- Prefer explicit public API layers over broad glob re-exports.
- Add scenario, replay or formal evidence for protocol changes.
- Do not edit generated formal artifacts manually.
- Keep names semantic, not milestone-based.
- Update docs and ADRs for architectural decisions.
- Keep generated readiness reports generated; do not hand-edit them.

## ADRs

Use `docs/templates/adr-template.md` when a change affects architecture,
authority boundaries, public API shape, or long-lived project policy.

An ADR should state how the decision is enforced across docs, formal artifacts,
replay, runtime behavior and tests. Do not use an ADR to bypass generated
truth-chain evidence.

## New Predicate Or Contract Behavior

New meaningful action families enter through the registry and contract
pipeline. Use `docs/templates/new-predicate-checklist.md` and make sure the
change names:

- subject and circumstance schemas;
- consequence profile, route and lifecycle class;
- effect signature, required witnesses, claims and leases;
- barrier, truth, projection and AuthZ policy;
- scenario, replay expectation and formal obligations or an explicit gap.

Execution-bearing predicates also need adapter certification tests or a
documented reason they are outside the current runtime surface.

## Formal Evidence

For protocol-critical changes, formal obligation comes before implementation.
Use:

- `docs/templates/formal-impact-record.md` for formal impact records;
- `docs/templates/formal-obligation-record.md` for obligation shape;
- `docs/formal/05-feature-fix-gating.md` for gating policy.

Every new invariant or enforcement rule needs a discriminating negative control:
an invalid scenario, runtime test, formal check, or tool-level drift/coverage
check that would fail if enforcement were missing.

## Runtime And Adapter Changes

Adapters must stay outside the semantic core. They may spend scoped
capabilities, persist audit events, export telemetry and integrate backend
runtimes; they must not create semantic admissibility, observed truth, AuthZ,
lease or witness authority.

Use `docs/product-track/adapter-certification-matrix.json` and
`docs/product-track/milestones/m08.7-adapter-certification.md` as the bounded
certification evidence for current adapters. New execution-bearing adapters
must document unsupported consequence profiles and add certification scenarios.

## AI-Assisted Contributions

AI assistance is allowed. Contributors remain responsible for submitted
changes, including correctness, security, licensing and provenance.

Do not list AI tools as `Co-authored-by:` commit trailers. Use `Assisted-by:` or
pull request disclosure for material AI assistance. Do not use AI tools as
commit authors, maintainers, reviewers or signers, and do not commit prompts,
credentials or private context packs.

## Checks

For ordinary Rust development, use the standard Rust toolchain commands:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

Maintainers working inside this repository can use equivalent checked-in
wrappers when available:

```bash
./tools/cargo-dev fmt --all -- --check
just check
just clippy
just test
```

Protocol, formal, runtime or release-related changes should add the relevant
specialized gate from the milestone runbook, such as replay/scenario checks,
formal verification, adapter certification, schema validation or publish
readiness.

## Publication-related Contributions

Publication preparation changes must not add runtime features. They may improve:

- repository shape;
- crate boundaries;
- public API clarity;
- docs and README quality;
- secret/context hygiene;
- release runbooks;
- package metadata;
- generated-readiness tooling.

Before merging publication-preparation changes, verify:

```bash
python3 tools/architecture-lint --json | jq -e '.status == "pass"'
python3 tools/pre-publication-review-gate --json | jq -e '.status == "pass"'
tools/schema-validate-all
tools/publish-readiness --check
```

Actual crates.io upload is not part of ordinary contribution flow; it is a
maintainer action performed through `PUBLISHING.md`.
