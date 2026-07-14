# Repository Relationship

Status: boundary note for the re-comp multi-repository workspace. This file
does not claim that product code has been migrated.

## Role

`causlane` owns the generic dispatcher kernel/runtime. It must remain reusable
and should not absorb Hopium-specific business semantics. For Stage 11 contract
closure, the preferred direction is to deepen generic host-dispatch capability
inside `causlane` while product repositories translate their own contracts into
that generic API outside this repository.

## Provides To

- `hopium-refinery`: dispatcher kernel/runtime capability through approved
  integration boundaries.
- `hopium-platform`: dispatcher integration points when they are
  infrastructure-level.
- `cli-checker`: dispatcher-boundary policy targets.

## Consumes From

- No Hopium-specific contracts, DTOs, schemas, or foundation primitives.
- Host/product repositories may consume `causlane` and map their own contracts
  into `HostDispatchContext` and `HostTaskSpec`.

## Does Not Own

- Hopium foundation primitives.
- Shared Hopium ABI/DTO/schema source of truth.
- Product runtime behavior.
- Data acquisition or ingestion workflows.
- Strategy authoring semantics.
- Backtest engine behavior.
- Trading decision or execution logic.
- CLI checker policy implementation.
- Product-specific bridge or compatibility adapters.

## E01 Canonical Governance Projection

The fingerprinted cross-repository projection for this owner is
`hopium.repository-governance.v1` in `hopium-contracts` release
`contracts-v0.2.32` (`48ecc9b5999e53e315f101dc0e11fba1f1be3098`).
It records explicit ownership/non-ownership and current/target public edges;
no target edge authorizes a private cross-repository import. This document
retains owner-specific detail. A boundary change must update both surfaces
through a new fingerprinted contract release.
