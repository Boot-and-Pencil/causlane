# ADR-0002: Start with documentation, then formal models, then code

- Status: accepted
- Date: 2026-06-05

## Context

The project's primary value is a semantic/protocol contract. If code is written first, accidental implementation details can become architecture.

## Decision

Development order:

```text
normative docs
  -> executable scenarios
  -> formal models
  -> compiled bundle format
  -> small reference kernel
  -> adapters/runtime
```

## Consequences

This reduces the risk of writing a runtime that cannot be explained, replayed or formally attacked. It increases upfront design effort.

## Enforcement

Every normative MUST/SHOULD should map to at least one enforcement target:

```text
Alloy assertion;
P monitor;
Kani harness;
Verus theorem;
replay check;
runtime guard.
```
