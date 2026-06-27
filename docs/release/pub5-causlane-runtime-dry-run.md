# PUB5 causlane-runtime Dry-run Evidence

**Status:** `causlane-runtime` dry-run passed; no crates.io upload performed.

This hand-maintained evidence records the staged PUB5 dry-run for
`causlane-runtime` in the `0.0.1` pre-alpha workspace release. It is release
evidence only; it does not authorize upload by itself.

## Dry-run Scope

Reviewed source baseline:

```text
main_commit: ee14a87793ee2e90a91b384d52906a045a62ea4c
date: 2026-06-27
host: dispatcher
runner: local repository workspace
crate: causlane-runtime
version: 0.0.1
```

Pre-dry-run gates:

```text
./tools/cargo-dev search causlane-core --limit 5: causlane-core = "0.0.1"
./tools/cargo-dev search causlane-formal --limit 5: causlane-formal = "0.0.1"
./tools/cargo-dev search causlane-contracts --limit 5: causlane-contracts = "0.0.1"
cli-checker check-repo --format json: pass, finding_count=0, blocking_count=0
./tools/cargo-dev fmt --all --check: pass
./tools/cargo-dev check --workspace --all-targets --all-features --locked: pass
./tools/cargo-dev clippy --workspace --all-targets --all-features --locked -- -D warnings: pass
./tools/cargo-dev test --workspace --all-targets --all-features --locked: pass
REAL_CARGO="$(rustup which --toolchain 1.85.0 cargo)" REAL_RUSTC="$(rustup which --toolchain 1.85.0 rustc)" ./tools/cargo-dev check --workspace --locked: pass
./tools/cargo-dev deny check: pass, with duplicate-version warnings accepted as M11.5 backlog
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
python3 tools/pre-publication-review-gate --json: pass
python3 tools/publication-plan-doc-lint --json: pass
tools/context-pack-scan: pass, 715 files scanned
tools/product-track-bundle --check: pass
tools/product-track-status-check: pass
tools/schema-validate-all: pass
tools/formal-verify-all: pass, coverage status=pass
```

Package file-list inspection:

```bash
./tools/cargo-dev package -p causlane-runtime --list --locked
```

Result:

```text
files_packaged: 36
unexpected_files: none observed
exit_code: 0
```

Dry-run command:

```bash
./tools/cargo-dev publish -p causlane-runtime --dry-run --locked
```

Result:

```text
files_packaged: 36
package_size: 349.2KiB
compressed_size: 70.2KiB
verify_compile: pass
upload: skipped by --dry-run
exit_code: 0
```

## Next State

The publication state machine may move to:

```text
DryRunPassed(causlane-runtime)
```

The next irreversible action, if maintainers choose to continue after CI, is:

```bash
./tools/cargo-dev publish -p causlane-runtime --locked
```

Do not dry-run or publish crates that depend on `causlane-runtime` until
`causlane-runtime` has been published and indexed on crates.io.
