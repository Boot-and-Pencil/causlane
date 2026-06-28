# PUB5 causlane-replay Publication Evidence

**Status:** `causlane-replay 0.0.1` published and indexed.

This hand-maintained evidence records the staged PUB5 upload for
`causlane-replay` in the `0.0.1` pre-alpha workspace release. It is release
evidence only; it does not authorize publishing dependent crates outside the
staged runbook.

## Publication Scope

Reviewed source baseline:

```text
main_commit: 3c342388ef49b62a6fc06ad202421f52645bb0ed
date: 2026-06-28
host: dispatcher
runner: local repository workspace
crate: causlane-replay
version: 0.0.1
```

Pre-upload gates:

```text
git status --short --branch: clean, main synced with origin/main
GitHub CI run 28307240903: success
cli-checker check-repo: pass, no findings
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
python3 tools/pre-publication-review-gate --json: pass
python3 tools/publication-plan-doc-lint --json: pass
tools/context-pack-scan: pass, 718 files scanned
tools/schema-validate-all: pass
tools/product-track-bundle --check: pass
tools/product-track-status-check: pass
./tools/cargo-dev fmt --all --check: pass
./tools/cargo-dev check --workspace --all-targets --all-features --locked: pass
./tools/cargo-dev clippy --workspace --all-targets --all-features --locked -- -D warnings: pass
./tools/cargo-dev test --workspace --all-targets --all-features --locked: pass
RUSTUP_TOOLCHAIN=1.85.0 ./tools/cargo-dev check -p causlane-replay --all-targets --all-features --locked: pass
./tools/cargo-dev deny check: pass, with duplicate-version warnings accepted as M11.5 backlog
```

Dry-run immediately before upload:

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

## Upload Result

Command:

```bash
./tools/cargo-dev publish -p causlane-replay --locked
```

Result:

```text
files_packaged: 43
package_size: 298.5KiB
compressed_size: 53.7KiB
verify_compile: pass
upload: success
published_at: 2026-06-28T01:13:20.764456Z
registry: crates-io
crate_url: https://crates.io/crates/causlane-replay/0.0.1
docs_url: https://docs.rs/causlane-replay/0.0.1/causlane_replay/
docs_rs_status_at_2026-06-28T01:15:21Z: HTTP 200
```

Index checks:

```text
./tools/cargo-dev search causlane-replay --limit 5
  causlane-replay = "0.0.1"

crates.io API:
  version: 0.0.1
  created_at: 2026-06-28T01:13:20.764456Z
  yanked: false

downstream smoke project:
  downloaded causlane-replay v0.0.1
  cargo check --locked: pass
```

## Follow-up

The replay-scoped Rust `1.85.0` MSRV gate passed. The workspace-wide
all-features Rust `1.85.0` compatibility follow-up from
`docs/release/pub5-causlane-replay-dry-run.md` remains open for the optional
`causlane-runtime` Restate dependency chain.

## Next State

The publication state machine moves to:

```text
Indexed(causlane-replay)
```

The next runbook crate is:

```text
causlane-codegen
```

Do not dry-run or publish crates after `causlane-codegen` in the runbook until
`causlane-codegen` has itself passed dry-run, been published and been indexed on
crates.io.

Update 2026-06-28: the staged `causlane-codegen` dry-run passed. Evidence is
recorded in `docs/release/pub5-causlane-codegen-dry-run.md`; the next
irreversible action is publishing `causlane-codegen` after CI and maintainer
confirmation.
