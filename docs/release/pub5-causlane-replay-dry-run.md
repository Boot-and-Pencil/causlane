# PUB5 causlane-replay Dry-run Evidence

**Status:** `causlane-replay` dry-run passed; no crates.io upload performed.

This hand-maintained evidence records the staged PUB5 dry-run for
`causlane-replay` in the `0.0.1` pre-alpha workspace release. It is release
evidence only; it does not authorize upload by itself.

## Dry-run Scope

Reviewed source baseline:

```text
main_commit: 4ef552c2cff0b7a56663a0bb099dad05c3a975e3
date: 2026-06-28
host: dispatcher
runner: local repository workspace
crate: causlane-replay
version: 0.0.1
```

Pre-dry-run gates:

```text
git status --short --branch: clean, main synced with origin/main
./tools/cargo-dev search causlane-core --limit 5: causlane-core = "0.0.1"
./tools/cargo-dev search causlane-contracts --limit 5: causlane-contracts = "0.0.1"
./tools/cargo-dev search causlane-runtime --limit 5: causlane-runtime = "0.0.1"
cli-checker check-repo: pass, no findings
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
python3 tools/pre-publication-review-gate --json: pass
python3 tools/publication-plan-doc-lint --json: pass
tools/context-pack-scan: pass, 717 files scanned
tools/schema-validate-all: pass
./tools/cargo-dev fmt --all --check: pass
./tools/cargo-dev check --workspace --all-targets --all-features --locked: pass
./tools/cargo-dev clippy --workspace --all-targets --all-features --locked -- -D warnings: pass
./tools/cargo-dev test --workspace --all-targets --all-features --locked: pass
RUSTUP_TOOLCHAIN=1.85.0 ./tools/cargo-dev check -p causlane-replay --all-targets --all-features --locked: pass
./tools/cargo-dev deny check: pass, with duplicate-version warnings accepted as M11.5 backlog
tools/product-track-bundle --check: pass
tools/product-track-status-check: pass
tools/formal-verify-all: pass, coverage status=pass
```

Package file-list inspection:

```bash
./tools/cargo-dev package -p causlane-replay --list --locked
```

Result:

```text
files_packaged: 43
unexpected_files: none observed
exit_code: 0
```

Dry-run command:

```bash
./tools/cargo-dev publish -p causlane-replay --dry-run --locked
```

Result:

```text
files_packaged: 43
package_size: 298.5KiB
compressed_size: 53.7KiB
verify_compile: pass
upload: skipped by --dry-run
exit_code: 0
```

## Follow-up

The replay-scoped MSRV gate passed on Rust `1.85.0`. A broader workspace
all-features MSRV probe on Rust `1.85.0` is not currently claimable because the
optional `causlane-runtime` Restate dependency chain includes crates with higher
declared MSRVs, including `restate-sdk 0.10.0`, `jsonwebtoken 10.4.0` and ICU
`2.2.x` packages.

This dry-run does not change `causlane-runtime` because `causlane-runtime
0.0.1` is already published. Resolve the runtime Restate dependency/MSRV policy
with a versioned follow-up before claiming workspace-wide Rust `1.85`
all-features compatibility.

## Next State

The publication state machine may move to:

```text
DryRunPassed(causlane-replay)
```

The next irreversible action, if maintainers choose to continue after CI and
explicit confirmation, is:

```bash
./tools/cargo-dev publish -p causlane-replay --locked
```

Do not dry-run or publish crates that depend on `causlane-replay` until
`causlane-replay` has been published and indexed on crates.io.
