# PUB5 Package File-List Review

**Status:** package file-list inspection complete.

This hand-maintained evidence records the local package inventory review for
the first public `0.0.1` workspace release. It is release evidence only; the
generated readiness report remains `docs/release/publish-readiness.md`.

## Review Scope

Reviewed source baseline:

```text
main_commit: 60a8a8607e8afa5a83efab35c6b441256fc871c2
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

The first one-crate dry-run has since passed for `causlane-core` and is recorded
in `docs/release/pub5-causlane-core-dry-run.md`. The next irreversible command,
if maintainers choose to continue, is:

```bash
cargo publish -p causlane-core --locked
```

Do not dry-run dependent crates until their internal registry dependencies have
been published and indexed. Do not upload any crate without following the
one-crate procedure in `docs/release/publish-all-crates-runbook.md`.
