# causlane-codegen

`causlane-codegen` generates formal artifacts from compiled Causlane dispatch
bundles.

## Status

This crate is experimental and pre-alpha. Generated output contracts may change
before `0.1`.

## Role In The Workspace

The crate consumes `CompiledDispatchBundle` artifacts and emits Alloy, P, Kani,
Lean4 and Verus-oriented artifacts that remain tied to the same
content-addressed truth consumed by replay and runtime checks.

## Public API Entry Points

- `generate_alloy_facts`
- `generate_alloy_facts_with_scenario`
- `generate_p_monitor`
- `generate_kani_harness`
- `generate_lean4_proof`
- `generate_verus_proof`
- `FormalTarget`, `GeneratedArtifact` and `FormalReceipt`

## Non-goals

This crate does not prove dispatcher invariants by itself and does not maintain
a separate hand-written formal truth model. It projects compiled bundle and
scenario inputs into generated artifacts; claim strength remains tied to the
generated receipts and external formal tools.

## Features

This crate currently has no optional Cargo features.
