# 07. Service workstream: cleanup, settings, tooling, hygiene

This workstream is not secondary. It prevents the project from becoming unverifiable or unusable.

## Repository and toolchain

- Keep `.devinfra/tool-versions.json`, `rust-toolchain.toml`, `justfile`, formal installer and doctor aligned.
- `tools/formal-doctor --json` must be usable before Rust/Cargo availability where possible.
- Pin versions and checksums for Alloy, Z3, Kani, Verus, Lean4/P where feasible.
- Split profiles: `base`, `rust`, `ci`, `formal`, `proof`, `all`.

## Generated artifacts

- Generated files must include source bundle hash, formal IR hash, scenario hash, generator version, artifact hash.
- Generated artifacts must be ignored/committed according to explicit policy, not ad hoc.
- Stale-check must block formal coverage claims.

## Cleanup and anti-drift

- Remove obsolete readiness docs or mark superseded with date and authority pointer.
- Keep roadmap status machine-derived where possible.
- Keep ADR index updated.
- Keep examples runnable or explicitly marked placeholder.
- Delete exploratory sketches when replaced by generated artifacts, or move to `sketches/`.

## Context-pack hygiene

- Never include secrets, tokens, private keys, local paths with sensitive info, large generated outputs unless intentional.
- Run context scan before sharing.
- Redact raw policy claims/identity payloads in support bundles.

## CI lanes

- Fast PR: fmt/check/test, replay small corpus, formal stale-check, docs links.
- Formal PR: generated Alloy/P/Kani smoke, receipts, coverage matrix.
- Nightly: larger scopes/unwind/interleavings, benchmark suite, fuzz/mutation tests.
- Release: proof/all profile, adapter certification, docs/security gates.

## Documentation cleanup cadence

Every stage exit includes:

- update `docs/formal-readiness-status.md` or successor;
- update coverage matrix;
- update roadmap status;
- update ADR if decision changed;
- update examples/readme if user-facing behavior changed.
