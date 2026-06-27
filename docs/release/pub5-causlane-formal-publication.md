# PUB5 causlane-formal Publication Evidence

**Status:** `causlane-formal 0.0.1` published and indexed.

This hand-maintained evidence records the staged PUB5 upload for
`causlane-formal` in the `0.0.1` pre-alpha workspace release. It is release
evidence only; it does not authorize publishing YAML-facing crates before the
M11.5 dependency-hygiene decision is resolved or explicitly accepted.

## Publication Scope

Reviewed source baseline:

```text
main_commit: 133f7e82c5eea30062645037c9800641749d6c8a
date: 2026-06-27
host: dispatcher
runner: local repository workspace
crate: causlane-formal
version: 0.0.1
```

Pre-upload gates:

```text
git status --short --branch: clean, main synced with origin/main
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
tools/context-pack-scan: pass, 709 git-visible files scanned
./tools/cargo-dev deny check: pass, with duplicate-version warnings accepted as M11.5 backlog
GitHub CI run 28267737503: success
```

Dry-run immediately before upload:

```bash
./tools/cargo-dev publish -p causlane-formal --dry-run --locked
```

Result:

```text
files_packaged: 7
package_size: 31.6KiB
compressed_size: 9.4KiB
verify_compile: pass
upload: skipped by --dry-run
exit_code: 0
```

## Upload Result

Command:

```bash
./tools/cargo-dev publish -p causlane-formal --locked
```

Result:

```text
upload: success
published_at: 2026-06-26T22:03:55Z
indexed_verified_at: 2026-06-26T22:04:07Z
registry: crates-io
crate_url: https://crates.io/crates/causlane-formal/0.0.1
docs_url: https://docs.rs/causlane-formal/0.0.1
docs_rs_status_at_2026-06-26T22:06:38Z: HTTP 200
```

Index checks:

```text
cargo search causlane-formal --limit 5
  causlane-formal = "0.0.1"

cargo info causlane-formal
  downloaded causlane-formal v0.0.1 from crates.io from a temporary directory
```

## Next State

The publication state machine moves to:

```text
Indexed(causlane-formal)
```

The next runbook crate is `causlane-contracts`.

Update 2026-06-27: the M11.5 YAML parser debt was resolved by the `noyalib`
migration, and the `causlane-contracts` dry-run passed. Current evidence is in
`docs/release/pub5-causlane-contracts-dry-run.md`.
