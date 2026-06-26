# ADR-0024: Quality gates and architecture lints

## Status

Accepted.

## Context

Some repository invariants are cheaper and clearer to check with a bootstrap
script than with Rust tests: declared binary source files, duplicate JSON schema
keys, line budget, core dependency boundaries and public glob re-exports.

## Decision

Add `tools/architecture-lint`, a dependency-free Python gate that runs before
Rust tooling is required. It reports errors and warnings in human or JSON form.

Initial hard errors:

- declared Cargo binary path missing;
- forbidden runtime dependency in `causlane-core`;
- duplicate keys or invalid JSON in contract schema files.

Initial warnings:

- module exceeds the line budget;
- public glob re-export.

`--strict` fails warnings and becomes a blocking gate after public API narrowing
and module decomposition have removed or explicitly documented the warnings.

## Consequences

- Fresh checkouts can detect repository-shape defects before cargo/formal runs.
- Architecture warnings are tracked rather than hidden.
- R0 can pass with the current known public-glob backlog, while R2/R3 own
  strict cleanup.
