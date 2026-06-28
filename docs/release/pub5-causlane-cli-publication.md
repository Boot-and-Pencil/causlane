# PUB5 causlane-cli Publication Evidence

**Status:** `causlane-cli 0.0.1` published and indexed.

This hand-maintained evidence records the staged PUB5 upload for
`causlane-cli` in the `0.0.1` pre-alpha workspace release. It is release
evidence only; it does not authorize tagging or PUB6 stabilization outside the
runbook.

## Publication Scope

Reviewed source baseline:

```text
main_commit: 3951743169e69c7554b9460181e6ed5e53d8668b
date: 2026-06-28
host: dispatcher
runner: local repository workspace
crate: causlane-cli
version: 0.0.1
```

Pre-upload gates:

```text
git status --short --branch: clean, main synced with origin/main
GitHub CI run 28318558837: success
./tools/cargo-dev search causlane-cli --limit 10: causlane-cli absent before upload
crates.io API for causlane-cli 0.0.1 before upload: HTTP 404
cli-checker check-repo --format json: pass, finding_count=0, blocking_count=0
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
python3 tools/pre-publication-review-gate --json: pass
python3 tools/publication-plan-doc-lint --json: pass
tools/context-pack-scan: pass, 724 files scanned
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

Dry-run immediately before upload:

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

## Upload Result

Command:

```bash
./tools/cargo-dev publish -p causlane-cli --locked
```

Result:

```text
files_packaged: 44
package_size: 303.0KiB
compressed_size: 62.7KiB
verify_compile: pass
upload: success
published_at: 2026-06-28T10:10:10.592697Z
registry: crates-io
crate_url: https://crates.io/crates/causlane-cli/0.0.1
docs_url: https://docs.rs/causlane-cli/0.0.1/causlane_cli/
docs_rs_status_at_2026-06-28T10:12:16Z: HTTP 200
```

Index checks:

```text
./tools/cargo-dev search causlane-cli --limit 10
  causlane-cli = "0.0.1"

crates.io API:
  version: 0.0.1
  created_at: 2026-06-28T10:10:10.592697Z
  yanked: false
  checksum: 2617fe7db29129a582e558c4b7a667cad1e3c34069de2f678db8f78132062265

downstream smoke project:
  /workspace/repo/tools/cargo-dev add causlane-cli@0.0.1: resolved causlane-cli v0.0.1
  downloaded causlane-cli v0.0.1
  /workspace/repo/tools/cargo-dev check --locked: pass

published CLI install smoke:
  ./tools/cargo-dev install causlane-cli --version 0.0.1 --locked --root <tmp>: pass
  installed binaries: causlane, causlane-formal, causlane-formal-discipline
  causlane formal doctor --profile base --json: pass, status=ok
```

The downstream smoke check used the checked-in `tools/cargo-dev` wrapper because
this devinfra host blocks direct Cargo invocation. The wrapper emitted
temporary-project `tools/devctl` lookup warnings before and after Cargo, but
the wrapped `cargo check --locked` completed successfully with exit code 0.

## Next State

The publication state machine moves to:

```text
Indexed(causlane-cli)
```

The staged PUB5 crate upload sequence for `0.0.1` is complete. The next
runbook action, after CI on this evidence commit, is tagging `v0.0.1` and
beginning PUB6 post-publication stabilization.
