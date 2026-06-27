# PUB5 causlane-contracts Dry-run Evidence

**Status:** `causlane-contracts` dry-run passed; no crates.io upload performed.

This hand-maintained evidence records the staged PUB5 dry-run for
`causlane-contracts` in the `0.0.1` pre-alpha workspace release. It is release
evidence only; it does not authorize upload by itself.

## Dry-run Scope

Reviewed source baseline:

```text
main_commit: 759e297952b90973f25d83b267f7c4edb459bd0d
date: 2026-06-27
host: dispatcher
runner: local repository workspace
crate: causlane-contracts
version: 0.0.1
```

Pre-dry-run gates:

```text
./tools/cargo-dev fmt --all --check: pass
./tools/cargo-dev check --workspace --all-targets --all-features --locked: pass
./tools/cargo-dev clippy --workspace --all-targets --all-features --locked -- -D warnings: pass
./tools/cargo-dev test --workspace --all-targets --all-features --locked: pass
REAL_CARGO="$(rustup which --toolchain 1.85.0 cargo)" REAL_RUSTC="$(rustup which --toolchain 1.85.0 rustc)" ./tools/cargo-dev check --workspace --locked: pass
./tools/cargo-dev deny check: pass, with duplicate-version warnings accepted as M11.5 backlog
tools/publish-readiness --check: pass
python3 tools/architecture-lint --json: pass
python3 tools/publication-plan-doc-lint --json: pass
tools/context-pack-scan: pass, 711 files scanned
tools/product-track-bundle --check: pass
tools/product-track-status-check: pass
tools/schema-validate-all: pass
tools/formal-verify-all: pass, coverage status=pass
```

Dependency hygiene spot-check:

```text
serde_yaml@0.9.34+deprecated: absent from locked dependency graph
unsafe-libyaml@0.2.11: absent from locked dependency graph
```

Package file-list inspection:

```bash
./tools/cargo-dev package -p causlane-contracts --list --locked
```

Result:

```text
files_packaged: 21
unexpected_files: none observed
new_expected_file: src/serde_numeric.rs
exit_code: 0
```

Dry-run command:

```bash
./tools/cargo-dev publish -p causlane-contracts --dry-run --locked
```

Result:

```text
files_packaged: 21
package_size: 127.8KiB
compressed_size: 32.5KiB
verify_compile: pass
upload: skipped by --dry-run
exit_code: 0
```

## Next State

The publication state machine may move to:

```text
DryRunPassed(causlane-contracts)
```

The next irreversible action, if maintainers choose to continue after CI, is:

```bash
./tools/cargo-dev publish -p causlane-contracts --locked
```

Do not publish dependent crates until `causlane-contracts` has been published
and indexed on crates.io.

Update 2026-06-27: `causlane-contracts 0.0.1` was published and indexed.
Publication evidence is recorded in
`docs/release/pub5-causlane-contracts-publication.md`.
