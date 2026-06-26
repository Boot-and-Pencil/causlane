# Publishing To crates.io

Causlane is a multi-crate workspace. Publication is a public, irreversible
supply-chain event for each uploaded version.

## Status

The first public upload target is an experimental **pre-alpha** workspace release
at:

```text
0.0.1
```

Do not publish `0.0.0`.

## License

The workspace uses:

```text
MIT OR Apache-2.0
```

Both `LICENSE-MIT` and `LICENSE-APACHE` must be present at the repository root
before publication.

## Required Before Any Upload

The publication refactor track is the hard prerequisite for upload; treat
`docs/release/refactor-before-publication-gate.md` as release evidence, not
advisory prose. PUB0-PUB4 are recorded complete, PUB4 public-baseline evidence
is recorded in `docs/release/pub4-public-baseline-handoff.md`, and PUB5 package
file-list review is recorded in
`docs/release/pub5-package-file-list-review.md`.

Run the gates below immediately before upload. If any publication-facing source,
metadata, package include list or generated readiness output changes, repeat the
affected evidence step before publishing.

```bash
git status --short
python3 tools/architecture-lint --json | jq -e '.status == "pass"'
python3 tools/pre-publication-review-gate --json | jq -e '.status == "pass"'
tools/schema-validate-all
tools/publish-readiness --check
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

A generated readiness report that says `publication_execution.status =
"deferred"` is not an upload approval. It only records deterministic
repository-local readiness.

The public GitHub baseline is already open. Before uploading PUB5 crates,
confirm the PUB4 handoff is still valid for the selected baseline; it selects
Gitleaks as the required secret scanner and records the exact baseline, scan
evidence, branch-protection confirmation and release owner.

## Publication Order

```text
1. causlane-core
2. causlane-formal
3. causlane-contracts
4. causlane-runtime
5. causlane-replay
6. causlane-codegen
7. causlane
8. causlane-cli
```

This order matters. A crate must not be dry-run/published until its internal
registry dependencies have already been published to crates.io and indexed.

## Package File Lists

File-list inspection can be done for all crates before upload:

```bash
for p in causlane-core causlane-formal causlane-contracts causlane-runtime causlane-replay causlane-codegen causlane causlane-cli; do
  cargo package -p "$p" --list --locked
  echo "reviewed package file list for $p"
done
```

Review every list manually. The current review is recorded in
`docs/release/pub5-package-file-list-review.md`; repeat it if package contents
change before upload.

## Staged Dry-run And Publish

Do not run `cargo publish --dry-run` for all workspace crates as one pre-publish
batch. Registry dry-run resolves published versions of internal dependencies, so
dependent crates may fail until earlier crates are actually uploaded.

Use the one-crate procedure:

```bash
crate=causlane-core
cargo publish -p "$crate" --dry-run --locked
cargo publish -p "$crate" --locked
cargo search "$crate"
```

Then continue with the next crate in the publication order.

## Public Repository Requirement

Before upload, the repository URL in Cargo metadata must resolve publicly:

```text
https://github.com/Boot-and-Pencil/causlane
```

Do not publish metadata that points to a private, missing or wrong repository.

## AI-assisted Development

AI assistance is allowed, but AI tools are not authors, co-authors, reviewers or
signers. Human maintainers are responsible for every published crate version.
See `AI_USAGE.md`.

## Never

- Never publish with a dirty worktree.
- Never publish with `--allow-dirty`.
- Never publish missing declared binary sources.
- Never publish package lists containing private context or secrets.
- Never hand-edit generated readiness reports.
- Never rely on `cargo yank` to remove uploaded secrets.

## Detailed Runbook

See `docs/release/publish-all-crates-runbook.md`.
