# Runbook: Publish All Causlane Crates

This runbook publishes the whole workspace as a pre-alpha `0.0.1` release.
It assumes the repository has already passed the publication preparation plan in
`docs/release/publication-prep.md` **and** the hard refactor-first gate in
`docs/release/refactor-before-publication-gate.md`.

## 0. Safety Rules

- Do not publish from a dirty worktree.
- Do not publish with `--allow-dirty`.
- Do not publish before inspecting package file lists.
- Do not rely on `cargo yank` to remove secrets; yanking does not erase source.
- Do not dry-run all crates as one pre-upload batch.
- Do not dry-run dependent crates before their internal dependencies have been
  published to crates.io and indexed.
- Do not hand-edit `docs/release/publish-readiness.md`; regenerate it.

## 1. Repository Gate

```bash
git status --short
python3 tools/architecture-lint --json | jq -e '.status == "pass"'
tools/schema-validate-all
tools/publish-readiness --check
```

Portable Rust checks:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

Maintainers may use checked-in wrappers for equivalent gates.

## 2. Version And Metadata

Confirm:

```toml
[workspace.package]
version = "0.0.1"
license = "MIT OR Apache-2.0"
repository = "https://github.com/Boot-and-Pencil/causlane"
homepage = "https://github.com/Boot-and-Pencil/causlane"
```

Confirm all internal workspace path dependencies use the same registry-compatible
version:

```toml
causlane-core = { path = "../causlane-core", version = "0.0.1" }
```

## 3. Package File List Review

This step can run for all crates before publication because it inspects local
package contents only.

```bash
for p in causlane-core causlane-formal causlane-contracts causlane-runtime causlane-replay causlane-codegen causlane causlane-cli; do
  cargo package -p "$p" --list
  echo "--- reviewed $p ---"
done
```

Reject the release if any package contains:

- private prompts, context packs or scratch notes;
- `.tools/` binaries or local caches;
- generated temporary output that is not a deliberate package input/example;
- secrets, credentials, tokens or private endpoints;
- missing README/license/source files;
- declared binary paths without source files.

## 4. Publish Order

Use this order:

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

The order is derived from normal workspace dependencies. Publish a crate only
after every internal registry dependency it needs is already available on
crates.io.

## 5. One-crate Publish Procedure

For each crate in order:

```bash
crate=causlane-core

cargo publish -p "$crate" --dry-run --locked
cargo publish -p "$crate" --locked
```

Then wait for registry/index propagation before moving to dependents:

```bash
cargo search "$crate"
cargo info "$crate"
```

If `cargo info` is unavailable in the local Cargo version, use `cargo search`
and a downstream temporary project after the dependency has propagated.

## 6. Full Sequence Template

```bash
for crate in causlane-core causlane-formal causlane-contracts causlane-runtime causlane-replay causlane-codegen causlane causlane-cli; do
  echo "==> dry-run $crate"
  cargo publish -p "$crate" --dry-run --locked

  echo "==> publish $crate"
  cargo publish -p "$crate" --locked

  echo "==> wait for registry index: $crate"
  cargo search "$crate"
  # Optional when supported:
  # cargo info "$crate"
done
```

If a dry-run fails because an internal dependency is missing from crates.io,
stop. Do not skip the dependency order.

## 7. Tag

After all crates publish successfully:

```bash
git tag -s v0.0.1 -m "Causlane 0.0.1"
git push origin main --tags
```

## 8. Downstream Smoke Test

```bash
tmp=$(mktemp -d)
cd "$tmp"
cargo new causlane-smoke
cd causlane-smoke
cargo add causlane@0.0.1
cargo check
```

If the CLI crate is intended to be installed:

```bash
cargo install causlane-cli --version 0.0.1 --locked
causlane --help
```

## 9. Failure Handling

If publish fails before upload, fix the repository and retry.

If publish succeeds but the uploaded crate is broken:

```bash
cargo yank <crate> --version 0.0.1
```

Then publish `0.0.2` after the fix. Do not attempt to overwrite `0.0.1`.

If a secret was uploaded, treat it as compromised; yanking is not a secret
removal mechanism.
