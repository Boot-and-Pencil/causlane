# Formal models implementation brief

Start formal model implementation only from a green `just formal-ready`.

## FM-001 Alloy bundle-bound structural model

Extend generated Alloy facts from the compiled bundle. The generic core remains
a reusable checker, not the authority by itself. New checks should consume
`source_bundle_hash`, scenario hash, formal obligations and generated facts.

Initial targets: I-001, I-002, I-003, I-005, I-006, I-009.

## FM-002 P protocol/interleaving model

Generate P monitors from the compiled bundle and scenario catalog. Focus on
interleavings around barrier, lease release/expiry, drain, and lifecycle close.

Initial targets: I-001, I-006, I-007, I-008, I-010.

## FM-003 Kani bounded Rust checks

Add bounded checks around pure Rust reducers and validators:

```text
reduce_lifecycle
ExecutionCapability::derive_from_barrier
LeaseTable grant/release/coverage
template resolver
ReplayErrorCode mapping
```

## FM-004 Verus abstract preservation proofs

Keep proofs over abstract kernel state. Do not model adapters, job queues or
workflow engines. Preserve the contract boundary:

```text
compiled bundle -> replay/codegen/formal facts -> receipts/stale-check
```
