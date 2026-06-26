# Contributing To Causlane

Causlane is experimental and pre-alpha. Contributions are welcome, but the
project prioritizes semantic clarity, replayability, formal/readiness discipline
and small kernel boundaries over feature velocity.

## Before Contributing

Read:

- `README.md`
- `AI_USAGE.md`
- `AGENTS.md`
- `docs/04-development-principles.md`
- `docs/adr/`

## Development Rules

- Keep `causlane-core` pure: no async runtime, database, HTTP, workflow engine,
  policy engine or telemetry dependency.
- Prefer explicit public API layers over broad glob re-exports.
- Add scenario, replay or formal evidence for protocol changes.
- Do not edit generated formal artifacts manually.
- Keep names semantic, not milestone-based.
- Update docs and ADRs for architectural decisions.
- Keep generated readiness reports generated; do not hand-edit them.

## AI-Assisted Contributions

AI assistance is allowed. Contributors remain responsible for submitted changes.

Do not list AI tools as `Co-authored-by:` commit trailers. Use `Assisted-by:` or
pull request disclosure for material AI assistance.

## Checks

For ordinary Rust development, use the standard Rust toolchain commands:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

Maintainers working inside this repository can use equivalent checked-in
wrappers when available:

```bash
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
```

Release-related changes should also validate schemas and publish readiness with
the repository scripts recorded in `PUBLISHING.md`.

## Publication-related Contributions

Publication preparation changes must not add runtime features. They may improve:

- repository shape;
- crate boundaries;
- public API clarity;
- docs and README quality;
- secret/context hygiene;
- release runbooks;
- package metadata;
- generated-readiness tooling.

Before opening a publication PR, verify:

```bash
python3 tools/architecture-lint --json | jq -e '.status == "pass"'
tools/schema-validate-all
tools/publish-readiness --check
```

Actual crates.io upload is not part of ordinary contribution flow; it is a
maintainer action performed through `PUBLISHING.md`.
