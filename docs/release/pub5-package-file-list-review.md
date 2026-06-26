# PUB5 Package File-List Review

**Status:** package file-list inspection complete; no crates.io dry-run or
upload performed.

This hand-maintained evidence records the local package inventory review for
the first public `0.0.1` workspace release. It is release evidence only; the
generated readiness report remains `docs/release/publish-readiness.md`.

## Review Scope

Reviewed source baseline:

```text
main_commit: 2decd4dc308ceb3c0a92c563b3c779d320ba5bde
operator: Vitalii Lobanov / vitalii-lobanov
date: 2026-06-26
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
| `causlane-contracts` | 20 | reviewed; package contents expected |
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

## Next State

The workspace may move from `LocalReady` to `PackageReviewed(all crates)` in
`docs/release/publish-sequence-state-machine.md`.

The next command in the staged publication runbook is the first one-crate dry
run:

```bash
cargo publish -p causlane-core --dry-run --locked
```

Do not dry-run dependent crates until their internal registry dependencies have
been published and indexed. Do not upload any crate without following the
one-crate procedure in `docs/release/publish-all-crates-runbook.md`.
