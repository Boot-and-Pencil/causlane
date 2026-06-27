# PUB5 causlane-contracts Publication Evidence

**Status:** `causlane-contracts 0.0.1` published and indexed.

This hand-maintained evidence records the staged PUB5 upload for
`causlane-contracts` in the `0.0.1` pre-alpha workspace release. It is release
evidence only; it does not authorize publishing dependent crates outside the
staged runbook.

## Publication Scope

Reviewed source baseline:

```text
main_commit: 4772178b01d6d347da087a67747086a26f6a7747
date: 2026-06-27
host: dispatcher
runner: local repository workspace
crate: causlane-contracts
version: 0.0.1
```

Pre-upload gates:

```text
git status --short --branch: clean, main synced with origin/main
python3 tools/architecture-lint --json: pass
python3 tools/pre-publication-review-gate --json: pass
tools/schema-validate-all: pass
tools/publish-readiness --check: pass
./tools/cargo-dev deny check: pass, with duplicate-version warnings accepted as M11.5 backlog
GitHub CI run 28295978175: success
```

Dry-run and package-list evidence:

```text
docs/release/pub5-causlane-contracts-dry-run.md
files_packaged: 21
package_size: 127.8KiB
compressed_size: 32.5KiB
verify_compile: pass
```

## Upload Result

Command:

```bash
./tools/cargo-dev publish -p causlane-contracts --locked
```

Result:

```text
files_packaged: 21
package_size: 127.8KiB
compressed_size: 32.5KiB
verify_compile: pass
upload: success
published_at: 2026-06-27T17:08:10.324056Z
registry: crates-io
crate_url: https://crates.io/crates/causlane-contracts/0.0.1
docs_url: https://docs.rs/causlane-contracts/0.0.1
docs_rs_status_at_2026-06-27T17:10:04Z: HTTP 200
```

Index checks:

```text
./tools/cargo-dev search causlane-contracts --limit 5
  causlane-contracts = "0.0.1"

crates.io API:
  version: 0.0.1
  created_at: 2026-06-27T17:08:10.324056Z

downstream smoke project:
  downloaded causlane-contracts v0.0.1
  downloaded causlane-core v0.0.1
  cargo check: pass
```

## Next State

The publication state machine moves to:

```text
Indexed(causlane-contracts)
```

The next runbook crate is:

```text
causlane-runtime
```

Do not dry-run or publish crates that depend on `causlane-runtime` until
`causlane-runtime` has itself been published and indexed on crates.io.
