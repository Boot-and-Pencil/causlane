# PUB5 Package File-List Review

**Status:** package file-list inspection complete.

This hand-maintained evidence records the local package inventory review for
the first public `0.0.1` workspace release. It is release evidence only; the
generated readiness report remains `docs/release/publish-readiness.md`.

## Review Scope

Reviewed source baseline:

```text
main_commit: 6913e087544ab7517052583e590ffc6716e25fa9
operator: Vitalii Lobanov / vitalii-lobanov
date: 2026-06-27
host: dispatcher
```

Command pattern:

```bash
cargo package -p <crate> --list --locked
```

Publication order reviewed:

```text
causlane-core
causlane-formal
causlane-contracts
causlane-runtime
causlane-replay
causlane-codegen
causlane
causlane-cli
```

## Review Result

| Crate | Files | Verdict |
|---|---:|---|
| `causlane-core` | 45 | reviewed; package contents expected |
| `causlane-formal` | 7 | reviewed; package contents expected |
| `causlane-contracts` | 21 | reviewed; package contents expected |
| `causlane-runtime` | 36 | reviewed; package contents expected |
| `causlane-replay` | 43 | reviewed; package contents expected |
| `causlane-codegen` | 30 | reviewed; package contents expected |
| `causlane` | 10 | reviewed; package contents expected |
| `causlane-cli` | 44 | reviewed; package contents expected |

No package file list contained:

- `target/`, `local/`, `.tools/` or local cache paths;
- local backup bundles or pre-publication recovery refs;
- repomix or generated context-pack outputs;
- private scratch notes, private prompts or private endpoints;
- obvious credentials, tokens or secret material.

Accepted intentional inclusions:

- Cargo packaging metadata emitted by `cargo package --list`, including
  `.cargo_vcs_info.json`, `Cargo.lock` and `Cargo.toml.orig`;
- crate-local `README.md` files, Rust source files, tests and benches;
- vendored fixture copies used by crate-local tests and drift-guarded by
  `tools/pre-publication-review-gate`;
- `fixtures/.devinfra/tool-versions.json` in `causlane-cli`, used as a
  crate-local fixture rather than a workspace-root include.

Update 2026-06-27: package file-list counts were rechecked after the
`serde_yaml` -> `noyalib` migration. `causlane-contracts` now includes
`src/serde_numeric.rs`; the 21-file list is recorded in
`docs/release/pub5-causlane-contracts-dry-run.md`.

Update 2026-06-27: `causlane-cli` package contents were rechecked after the
devinfra checker metadata pin was updated to `cli-checker 0.1.20` in
`fixtures/.devinfra/tool-versions.json`. The file list remains 44 files, and
the fixture remains an expected crate-local test input.

## Next State

The workspace has moved beyond `PackageReviewed(all crates)`: `causlane-core`,
`causlane-formal`, `causlane-contracts` and `causlane-runtime` have been
published and indexed. The staged dry-run for `causlane-replay` passed; evidence
is recorded in `docs/release/pub5-causlane-replay-dry-run.md`.

The next irreversible command, if maintainers choose to continue after CI and
explicit confirmation, is:

```bash
./tools/cargo-dev publish -p causlane-replay --locked
```

Do not dry-run or publish crates that depend on `causlane-replay` until
`causlane-replay` has been published and indexed. Do not upload any crate
without following the one-crate procedure in
`docs/release/publish-all-crates-runbook.md`.
