# PUB5 causlane-codegen Dry-run Evidence

**Status:** `causlane-codegen` dry-run passed; no crates.io upload performed.

This hand-maintained evidence records the staged PUB5 dry-run for
`causlane-codegen` in the `0.0.1` pre-alpha workspace release. It is release
evidence only; it does not authorize upload by itself.

## Dry-run Scope

Reviewed source baseline:

```text
main_commit: 76626354eff77505dd172751799225f6d6f74065
date: 2026-06-28
host: dispatcher
runner: local repository workspace
crate: causlane-codegen
version: 0.0.1
```

Pre-dry-run gates:

```text
git status --short --branch: clean, main synced with origin/main
./tools/cargo-dev search causlane-contracts --limit 5: causlane-contracts = "0.0.1"
crates.io API for causlane-codegen 0.0.1: HTTP 404 before dry-run
cli-checker check-repo: pass, no findings
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
python3 tools/pre-publication-review-gate --json: pass
python3 tools/publication-plan-doc-lint --json: pass
tools/context-pack-scan: pass, 719 files scanned
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

Dry-run command:

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

## Next State

The publication state machine may move to:

```text
DryRunPassed(causlane-codegen)
```

The next irreversible action, if maintainers choose to continue after CI and
explicit confirmation, is:

```bash
./tools/cargo-dev publish -p causlane-codegen --locked
```

Do not dry-run or publish crates after `causlane-codegen` in the runbook until
`causlane-codegen` has been published and indexed on crates.io.
