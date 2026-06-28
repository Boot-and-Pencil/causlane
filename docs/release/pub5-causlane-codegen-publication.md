# PUB5 causlane-codegen Publication Evidence

**Status:** `causlane-codegen 0.0.1` published and indexed.

This hand-maintained evidence records the staged PUB5 upload for
`causlane-codegen` in the `0.0.1` pre-alpha workspace release. It is release
evidence only; it does not authorize publishing dependent crates outside the
staged runbook.

## Publication Scope

Reviewed source baseline:

```text
main_commit: 7d8534c990c657b579c138dd76758b2956b4d642
date: 2026-06-28
host: dispatcher
runner: local repository workspace
crate: causlane-codegen
version: 0.0.1
```

Pre-upload gates:

```text
git status --short --branch: clean, main synced with origin/main
pre-publish checkpoint: 7d8534c990c657b579c138dd76758b2956b4d642 pushed to origin/main
GitHub CI run 28308488495: success
./tools/cargo-dev search causlane-contracts --limit 5: causlane-contracts = "0.0.1"
crates.io API for causlane-codegen 0.0.1 before upload: HTTP 404
cli-checker check-repo --format json: pass, finding_count=0, blocking_count=0
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
python3 tools/pre-publication-review-gate --json: pass
python3 tools/publication-plan-doc-lint --json: pass
tools/context-pack-scan: pass, 720 files scanned
tools/schema-validate-all: pass
tools/product-track-bundle --check: pass
tools/product-track-status-check: pass
./tools/cargo-dev fmt --all --check: pass
./tools/cargo-dev check --workspace --all-targets --all-features --locked: pass
./tools/cargo-dev clippy --workspace --all-targets --all-features --locked -- -D warnings: pass
./tools/cargo-dev test --workspace --all-targets --all-features --locked: pass
RUSTUP_TOOLCHAIN=1.85.0 ./tools/cargo-dev check -p causlane-codegen --all-targets --all-features --locked: pass
./tools/cargo-dev deny check: pass, with duplicate-version warnings accepted as M11.5 backlog
```

Package file-list inspection:

```bash
./tools/cargo-dev package -p causlane-codegen --list --locked
```

Result:

```text
files_packaged: 30
unexpected_files: none observed
exit_code: 0
```

Dry-run immediately before upload:

```bash
./tools/cargo-dev publish -p causlane-codegen --dry-run --locked
```

Result:

```text
files_packaged: 30
package_size: 307.8KiB
compressed_size: 67.8KiB
verify_compile: pass
upload: skipped by --dry-run
exit_code: 0
```

## Upload Result

Command:

```bash
./tools/cargo-dev publish -p causlane-codegen --locked
```

Result:

```text
files_packaged: 30
package_size: 307.8KiB
compressed_size: 67.8KiB
verify_compile: pass
upload: success
published_at: 2026-06-28T02:12:17.908438Z
registry: crates-io
crate_url: https://crates.io/crates/causlane-codegen/0.0.1
docs_url: https://docs.rs/causlane-codegen/0.0.1/causlane_codegen/
docs_rs_status_at_2026-06-28T02:13:19Z: HTTP 200
```

Index checks:

```text
./tools/cargo-dev search causlane-codegen --limit 5
  causlane-codegen = "0.0.1"

crates.io API:
  version: 0.0.1
  created_at: 2026-06-28T02:12:17.908438Z
  yanked: false
  checksum: 36c643419715d4d417c3a4d4377e887ec9f366fbc8574c2eb64a8a037de49e9e

downstream smoke project:
  cargo add causlane-codegen@0.0.1: resolved causlane-codegen v0.0.1
  /workspace/repo/tools/cargo-dev check --locked: pass
```

The downstream smoke check used the checked-in `tools/cargo-dev` wrapper because
this devinfra host blocks direct `cargo check`. The wrapper emitted two
temporary-project `tools/devctl` lookup warnings before and after Cargo, but the
wrapped `cargo check --locked` completed successfully with exit code 0.

## Next State

The publication state machine moves to:

```text
Indexed(causlane-codegen)
```

The next runbook crate is:

```text
causlane
```

Do not dry-run or publish `causlane-cli` until `causlane` has itself passed
dry-run, been published and been indexed on crates.io.
