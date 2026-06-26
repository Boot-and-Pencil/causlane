# causlane-contracts

`causlane-contracts` contains registry, compiled bundle, canonical hashing and
scenario contract types for Causlane.

## Status

This crate is experimental and pre-alpha. It is not a production registry
service. APIs and serialized shapes may change before `0.1`.

## Role In The Workspace

The crate turns registry and bundle documents into typed contract values and
computes content-addressed hashes used by replay, formal artifact generation and
runtime checks.

## Public API Entry Points

- Bundle and registry types such as `CompiledDispatchBundle`,
  `RegistryManifest` and `BundleInvariantReport`.
- Canonical JSON helpers and hash helpers for bundle and plan data.
- Example builders for tests and documentation.

## Features

This crate currently has no optional Cargo features.

