# Changelog

All notable changes to this project will be documented in this file.

Causlane uses SemVer-compatible versioning while it remains pre-1.0. The first
public release is an experimental `0.0.1` package-family baseline.

## [0.0.1] - 2026-06-29

### Added

- Initial public pre-alpha crate family published to crates.io:
  `causlane-core`, `causlane-formal`, `causlane-contracts`,
  `causlane-runtime`, `causlane-replay`, `causlane-codegen`, `causlane` and
  `causlane-cli`.
- Semantic dispatch kernel scaffold.
- Registry and compiled bundle contracts.
- Bundle-bound replay verifier scaffold.
- Formal artifact generation scaffold.
- Runtime adapter skeletons.
- CLI tooling scaffold.
- Signed release tag `v0.0.1`.

### Notes

This release is experimental and not production-ready. APIs may change before
`0.1`.

Known limitations:

- Causlane is not a workflow engine, scheduler or job queue.
- Formal and replay evidence is receipt-backed pre-alpha evidence, not a
  complete formal proof.
- Workspace-wide all-features Rust `1.85` compatibility is not claimed because
  the optional Restate runtime dependency chain declares higher MSRVs.
- `cargo-deny` duplicate-version warnings remain tracked as convergence
  backlog.

This is the first public release, so there are no migration steps or breaking
changes from an earlier published version.
