# PUB5 causlane-cli Dry-run Evidence

**Status:** `causlane-cli` dry-run passed; no crates.io upload performed.

This hand-maintained evidence records the staged PUB5 dry-run for
`causlane-cli` in the `0.0.1` pre-alpha workspace release. It is release
evidence only; it does not authorize upload by itself.

## Dry-run Scope

Reviewed source baseline:

```text
main_commit: 0d40bbb495935a19aacc96bb81d591817658ed14
date: 2026-06-28
host: dispatcher
runner: local repository workspace
crate: causlane-cli
version: 0.0.1
```

Pre-dry-run gates:

```text
git status --short --branch: clean, main synced with origin/main
GitHub CI run 28318251254: success
./tools/cargo-dev search causlane-cli --limit 10: no exact causlane-cli result
crates.io API for causlane-cli 0.0.1: HTTP 404 before dry-run
cli-checker check-repo --format json: pass, finding_count=0, blocking_count=0
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
python3 tools/pre-publication-review-gate --json: pass
python3 tools/publication-plan-doc-lint --json: pass
tools/context-pack-scan: pass, 723 files scanned
tools/schema-validate-all: pass
tools/product-track-bundle --check: pass
tools/product-track-status-check: pass
./tools/cargo-dev fmt --all --check: pass
./tools/cargo-dev check --workspace --all-targets --all-features --locked: pass
./tools/cargo-dev clippy --workspace --all-targets --all-features --locked -- -D warnings: pass
./tools/cargo-dev test --workspace --all-targets --all-features --locked: pass
RUSTUP_TOOLCHAIN=1.85.0 ./tools/cargo-dev check -p causlane-cli --all-targets --all-features --locked: pass
./tools/cargo-dev deny check: pass, with duplicate-version warnings accepted as M11.5 backlog
```

Package file-list inspection:

```bash
./tools/cargo-dev package -p causlane-cli --list --locked
```

Result:

```text
files_packaged: 44
unexpected_files: none observed
exit_code: 0
```

Dry-run command:

```bash
./tools/cargo-dev publish -p causlane-cli --dry-run --locked
```

Result:

```text
files_packaged: 44
package_size: 303.0KiB
compressed_size: 62.7KiB
verify_compile: pass
upload: skipped by --dry-run
exit_code: 0
```

Post-dry-run registry checks:

```text
./tools/cargo-dev search causlane-cli --limit 10: no exact causlane-cli result
crates.io API for causlane-cli 0.0.1: HTTP 404
```

## Next State

The publication state machine may move to:

```text
DryRunPassed(causlane-cli)
```

The next irreversible action, if maintainers choose to continue after CI and
explicit confirmation, is:

```bash
./tools/cargo-dev publish -p causlane-cli --locked
```

Do not tag `v0.0.1`, run final downstream install evidence, or move to PUB6
until `causlane-cli` has been published and indexed on crates.io.
