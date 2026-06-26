# causlane-replay

`causlane-replay` is the bundle-bound replay verifier for Causlane protocol
traces.

## Status

This crate is experimental and pre-alpha. It is a library verifier, not a
production audit store. APIs may change before `0.1`.

Publication status is tracked in the public repository: package file-list review
is recorded, and upload must follow the staged runbook in
<https://github.com/Boot-and-Pencil/causlane/blob/main/PUBLISHING.md>.

## Role In The Workspace

The crate loads JSON trace shapes into typed events and verifies them against
protocol invariants using bundle metadata, hashes and replay contracts.

## Public API Entry Points

- `ReplayTrace` for typed trace loading and verification.
- `ReplayVerdict` and `ReplayExplain` for machine and human diagnostics.
- `ReplayError` and `ReplayErrorCode` for stable failure codes.
- `ReplayContracts` and `ReplayOracle` for bundle-bound verification policy.

## Features

This crate currently has no optional Cargo features.
