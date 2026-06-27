# PUB5 causlane-core Publication Evidence

**Status:** `causlane-core 0.0.1` published and indexed.

This hand-maintained evidence records the first irreversible PUB5 upload for
the `0.0.1` pre-alpha workspace release. It is release evidence only; it does
not authorize publishing the remaining crates outside the staged runbook.

## Publication Scope

Reviewed source baseline:

```text
main_commit: ad3d13c5a463177579d572e65f9e37b5e9caaf46
date: 2026-06-26
host: dispatcher
runner: local repository workspace
crate: causlane-core
version: 0.0.1
```

Pre-upload gates:

```text
git status --short --branch: clean, main synced with origin/main
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
tools/context-pack-scan: pass, 708 git-visible files scanned
./tools/cargo-dev deny check: pass, with duplicate-version warnings accepted as M11.5 backlog
GitHub CI run 28261304430: success
```

Dry-run immediately before upload:

```bash
./tools/cargo-dev publish -p causlane-core --dry-run --locked
```

Result:

```text
files_packaged: 45
package_size: 415.8KiB
compressed_size: 93.8KiB
verify_compile: pass
upload: skipped by --dry-run
exit_code: 0
```

## Upload Result

Command:

```bash
./tools/cargo-dev publish -p causlane-core --locked
```

Result:

```text
upload: success
published_at: 2026-06-26T19:51:01Z
indexed_verified_at: 2026-06-26T19:51:10Z
registry: crates-io
crate_url: https://crates.io/crates/causlane-core/0.0.1
docs_url: https://docs.rs/causlane-core/0.0.1
```

Index checks:

```text
cargo search causlane-core --limit 5
  causlane-core = "0.0.1"

cargo info causlane-core
  downloaded causlane-core v0.0.1 from crates.io from a temporary directory
```

## Next State

The publication state machine moves to:

```text
Indexed(causlane-core)
```

The next runbook step, if maintainers choose to continue, is:

```bash
cargo publish -p causlane-formal --dry-run --locked
```

Update 2026-06-27: the M11.5 `serde_yaml`/`unsafe-libyaml` debt was resolved by
the `noyalib` migration before the `causlane-contracts` dry-run.
