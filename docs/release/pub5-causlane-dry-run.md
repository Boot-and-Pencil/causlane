# PUB5 causlane Dry-run Evidence

**Status:** `causlane` dry-run passed; no crates.io upload performed.

This hand-maintained evidence records the staged PUB5 dry-run for `causlane` in
the `0.0.1` pre-alpha workspace release. It is release evidence only; it does
not authorize upload by itself.

## Dry-run Scope

Reviewed source baseline:

```text
main_commit: 8559850190b0b38bf88d7574eab82c4e444fc038
date: 2026-06-28
host: dispatcher
runner: local repository workspace
crate: causlane
version: 0.0.1
```

Pre-dry-run gates:

```text
git status --short --branch: clean, main synced with origin/main
GitHub CI run 28308603483: success
./tools/cargo-dev search causlane --limit 10: prior runbook crates indexed; causlane absent
crates.io API for causlane 0.0.1: HTTP 404 before dry-run
cli-checker check-repo --format json: pass, finding_count=0, blocking_count=0
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
python3 tools/pre-publication-review-gate --json: pass
python3 tools/publication-plan-doc-lint --json: pass
tools/context-pack-scan: pass, 721 files scanned
tools/schema-validate-all: pass
tools/product-track-bundle --check: pass
tools/product-track-status-check: pass
./tools/cargo-dev fmt --all --check: pass
./tools/cargo-dev check --workspace --all-targets --all-features --locked: pass
./tools/cargo-dev clippy --workspace --all-targets --all-features --locked -- -D warnings: pass
./tools/cargo-dev test --workspace --all-targets --all-features --locked: pass
RUSTUP_TOOLCHAIN=1.85.0 ./tools/cargo-dev check -p causlane --all-targets --all-features --locked: pass
./tools/cargo-dev deny check: pass, with duplicate-version warnings accepted as M11.5 backlog
```

Package file-list inspection:

```bash
./tools/cargo-dev package -p causlane --list --locked
```

Result:

```text
files_packaged: 10
unexpected_files: none observed
exit_code: 0
```

Dry-run command:

```bash
./tools/cargo-dev publish -p causlane --dry-run --locked
```

Result:

```text
files_packaged: 10
package_size: 35.7KiB
compressed_size: 10.3KiB
verify_compile: pass
upload: skipped by --dry-run
exit_code: 0
```

## Next State

The publication state machine may move to:

```text
DryRunPassed(causlane)
```

The next irreversible action, if maintainers choose to continue after CI and
explicit confirmation, is:

```bash
./tools/cargo-dev publish -p causlane --locked
```

Do not dry-run or publish `causlane-cli` until `causlane` has been published and
indexed on crates.io.

Update 2026-06-28: `causlane 0.0.1` was published and indexed. Evidence is
recorded in `docs/release/pub5-causlane-publication.md`; the next runbook crate
is `causlane-cli`.
