# Release Process

This document describes the release process for Causlane.

## Release Types

- `0.0.x`: public or private experimental bootstrap releases. APIs are unstable
  and crates may be published only with explicit pre-alpha status.
- `0.1.x`: first usable public alpha API surface.
- `1.0.0`: stable API and compatibility policy.

## Required Before Release

Portable Rust checks:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

Repository-specific gates:

```bash
python3 tools/architecture-lint --json | jq -e '.status == "pass"'
python3 tools/pre-publication-review-gate --json | jq -e '.status == "pass"'
tools/schema-validate-all
tools/publish-readiness --check
```

If formal/toolchain claims are included in release notes, run the corresponding
formal gates and include receipt status. Do not overclaim formal coverage.

For a real crates.io release, run the full package-list and staged dry-run
sequence in `PUBLISHING.md` and `docs/release/publish-all-crates-runbook.md`.

## First Public Release

The first planned public upload is `0.0.1`, not `0.1.0`. Treat it as a pre-alpha
bootstrap release whose purpose is package availability, dependency deployment
and public provenance — not stable API commitment.

The public repository baseline and package file-list review are recorded, and
`causlane-core`, `causlane-formal`, `causlane-contracts` and
`causlane-runtime` and `causlane-replay` have been published and indexed. The
next staged step is the one-crate dry-run for `causlane-codegen`.

## Tagging

After successful crates.io publication:

```bash
git tag -s vX.Y.Z -m "Causlane X.Y.Z"
git push origin main --tags
```

## Release Notes

Every release must state:

- maturity status;
- crates published;
- known limitations;
- public API stability level;
- formal/replay coverage status;
- security/provenance notes;
- breaking changes;
- migration notes if applicable.

## Failure Handling

If a crate was uploaded with a serious bug, publish a fix as a new version.
`cargo yank` can prevent new dependency resolution for the yanked version, but it
cannot overwrite or delete uploaded source.
