# causlane

`causlane` is the public facade crate for the causlane workspace: a typed,
auditable, replayable and consequence-aware dispatch kernel for Rust systems.

This crate intentionally stays small. Use `causlane::prelude` for common
imports and `causlane::core::{protocol, kernel, ports, prelude}` for explicit
access to curated kernel layers. Runtime adapters, replay tooling, formal
tooling and the CLI live in separate workspace crates.

## Status

This package is experimental and pre-alpha. It is not a production workflow
engine, job queue or scheduler. APIs may change before `0.1`.

Package file-list review is recorded for the `0.0.1` pre-alpha publication
track. Upload still follows the dependency-ordered staged runbook; this README
does not authorize crates.io publication by itself.

- Project overview: <https://github.com/Boot-and-Pencil/causlane>
- Publishing notes: <https://github.com/Boot-and-Pencil/causlane/blob/main/PUBLISHING.md>
- Package review: <https://github.com/Boot-and-Pencil/causlane/blob/main/docs/release/pub5-package-file-list-review.md>

## Minimal Import

```rust
use causlane::prelude::*;
```

`causlane::core` is a curated facade, not a full alias for `causlane-core`.
Testing helpers remain available through a direct `causlane-core` dependency and
`causlane_core::testing`.

## Features

This facade crate currently has no optional Cargo features. Default builds pull
only `causlane-core`; runtime adapters, replay tooling, formal tooling and CLI
dependencies stay in their own crates.
