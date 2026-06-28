# PUB5 causlane Publication Evidence

**Status:** `causlane 0.0.1` published and indexed.

This hand-maintained evidence records the staged PUB5 upload for `causlane` in
the `0.0.1` pre-alpha workspace release. It is release evidence only; it does
not authorize publishing `causlane-cli` outside the staged runbook.

## Publication Scope

Reviewed source baseline:

```text
main_commit: f9761c50ccfda2db63f3b93153cf59806f2eda47
date: 2026-06-28
host: dispatcher
runner: local repository workspace
crate: causlane
version: 0.0.1
```

Pre-upload gates:

```text
git status --short --branch: clean, main synced with origin/main
GitHub CI run 28317392460: success
./tools/cargo-dev search causlane --limit 10: prior runbook crates indexed; causlane absent
crates.io API for causlane 0.0.1 before upload: HTTP 404
cli-checker check-repo --format json: pass, finding_count=0, blocking_count=0
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
python3 tools/pre-publication-review-gate --json: pass
python3 tools/publication-plan-doc-lint --json: pass
tools/context-pack-scan: pass, 722 files scanned
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

Dry-run immediately before upload:

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

## Upload Result

Command:

```bash
./tools/cargo-dev publish -p causlane --locked
```

Result:

```text
files_packaged: 10
package_size: 35.7KiB
compressed_size: 10.3KiB
verify_compile: pass
upload: success
published_at: 2026-06-28T09:40:20.252158Z
registry: crates-io
crate_url: https://crates.io/crates/causlane/0.0.1
docs_url: https://docs.rs/causlane/0.0.1/causlane/index.html
docs_rs_status_at_2026-06-28T09:41:32Z: HTTP 200, doc_status=true
```

Index checks:

```text
./tools/cargo-dev search causlane --limit 10
  causlane = "0.0.1"

crates.io API:
  version: 0.0.1
  created_at: 2026-06-28T09:40:20.252158Z
  yanked: false
  checksum: 1441c014b854832be9de97de63506543df9ab7195dc73b4aa46d4679bba8b60d

downstream smoke project:
  cargo add causlane@0.0.1: resolved causlane v0.0.1
  downloaded causlane v0.0.1
  downloaded causlane-core v0.0.1
  /workspace/repo/tools/cargo-dev check --locked: pass
```

The downstream smoke check used the checked-in `tools/cargo-dev` wrapper because
this devinfra host blocks direct `cargo check`. The wrapper emitted two
temporary-project `tools/devctl` lookup warnings before and after Cargo, but the
wrapped `cargo check --locked` completed successfully with exit code 0.

## Next State

The publication state machine moves to:

```text
Indexed(causlane)
```

The next runbook crate is:

```text
causlane-cli
```

Update 2026-06-28: the staged `causlane-cli` dry-run passed. Evidence is
recorded in `docs/release/pub5-causlane-cli-dry-run.md`. The next irreversible
action is publishing `causlane-cli` after green CI and maintainer confirmation.
