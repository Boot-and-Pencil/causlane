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

The first public upload is `0.0.1`, not `0.1.0`. It is a pre-alpha bootstrap
release whose purpose is package availability, dependency deployment and public
provenance — not stable API commitment.

The public repository baseline, package file-list review, staged publication
evidence and signed tag evidence are recorded under `docs/release/`. All eight
runbook crates have been published and indexed on crates.io:

- `causlane-core 0.0.1`;
- `causlane-contracts 0.0.1`;
- `causlane-runtime 0.0.1`;
- `causlane-replay 0.0.1`;
- `causlane-codegen 0.0.1`;
- `causlane 0.0.1`;
- `causlane-cli 0.0.1`.

Signed tag `v0.0.1` points to the final PUB5 publication evidence commit. PUB6
post-publication evidence is recorded in
`docs/release/pub6-v0.0.1-post-publication.md`. The GitHub pre-release is
published at
<https://github.com/Boot-and-Pencil/causlane/releases/tag/v0.0.1>.

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
