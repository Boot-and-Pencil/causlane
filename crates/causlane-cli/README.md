# causlane-cli

`causlane-cli` provides developer and operator command-line tools for Causlane.

## Status

This crate is experimental and pre-alpha. Command names and output shapes may
change before `0.1`.

## Role In The Workspace

The crate owns CLI boundaries for bundle checks, scenario projection, replay,
formal artifact orchestration and readiness helpers. Shared CLI library code is
kept here so binaries do not duplicate orchestration logic.

## Binaries

- `causlane`
- `causlane-formal`
- `causlane-formal-discipline`

## Non-goals

This crate is not the semantic authority for protocol validity, replay status
or formal coverage. It orchestrates checked library and tool boundaries; the
protocol truth remains in contracts, compiled bundles, replay/formal receipts
and maintainer-reviewed docs.

## Features

This crate currently has no optional Cargo features.
