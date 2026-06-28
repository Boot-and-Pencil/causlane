# PUB5 causlane-runtime Publication Evidence

**Status:** `causlane-runtime 0.0.1` published and indexed.

This hand-maintained evidence records the staged PUB5 upload for
`causlane-runtime` in the `0.0.1` pre-alpha workspace release. It is release
evidence only; it does not authorize publishing dependent crates outside the
staged runbook.

## Publication Scope

Reviewed source baseline:

```text
main_commit: 1854d9800452b13a8f881d190f0f3e480b6da835
date: 2026-06-28
host: dispatcher
runner: local repository workspace
crate: causlane-runtime
version: 0.0.1
```

Pre-upload gates:

```text
git status --short --branch: clean, main synced with origin/main
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
tools/context-pack-scan: pass, 716 files scanned
GitHub CI run 28305705376: success
```

Dry-run immediately before upload:

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

## Upload Result

Command:

```bash
./tools/cargo-dev publish -p causlane-runtime --locked
```

Result:

```text
files_packaged: 36
package_size: 349.2KiB
compressed_size: 70.2KiB
verify_compile: pass
upload: success
published_at: 2026-06-27T23:59:47.881445Z
registry: crates-io
crate_url: https://crates.io/crates/causlane-runtime/0.0.1
docs_url: https://docs.rs/causlane-runtime/0.0.1/causlane_runtime/
docs_rs_status_at_2026-06-28T00:03:29Z: HTTP 200
```

Index checks:

```text
./tools/cargo-dev search causlane-runtime --limit 5
  causlane-runtime = "0.0.1"

crates.io API:
  version: 0.0.1
  created_at: 2026-06-27T23:59:47.881445Z

downstream smoke project:
  downloaded causlane-runtime v0.0.1
  cargo check --locked: pass
```

## Next State

The publication state machine moves to:

```text
Indexed(causlane-runtime)
```

The next runbook crate is:

```text
causlane-replay
```

Do not dry-run or publish crates that depend on `causlane-replay` until
`causlane-replay` has itself been published and indexed on crates.io.

Update 2026-06-28: the staged `causlane-replay` dry-run passed. Evidence is
recorded in `docs/release/pub5-causlane-replay-dry-run.md`; the next
irreversible action is publishing `causlane-replay` after CI and maintainer
confirmation.
