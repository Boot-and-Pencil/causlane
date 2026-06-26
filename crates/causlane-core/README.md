# causlane-core

`causlane-core` is the pure semantic dispatch kernel for Causlane.

## Status

This crate is experimental and pre-alpha. It is not a production workflow
engine, job queue or scheduler. APIs may change before `0.1`.

## Role In The Workspace

`causlane-core` owns the domain model, lifecycle rules, kernel contracts and
ports that other crates consume. It intentionally has no async runtime, database,
HTTP, workflow-engine or telemetry dependency.

## Public API Entry Points

- `causlane_core::prelude` for common imports.
- `causlane_core::protocol` for protocol data and lifecycle concepts.
- `causlane_core::kernel` for kernel contracts and pure rule helpers.
- `causlane_core::ports` for host integration traits.
- `causlane_core::testing` for test helpers and deterministic fixtures.

## Features

This crate currently has no optional Cargo features.

