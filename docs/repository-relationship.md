# Repository Relationship

Status: boundary note for the re-comp multi-repository workspace. This file
does not claim that product code has been migrated.

## Role

`causlane` owns the generic dispatcher kernel/runtime. It must remain reusable
and should not absorb Hopium-specific business semantics.

## Provides To

- `hopium-refinery`: dispatcher kernel/runtime capability through approved
  integration boundaries.
- `hopium-platform`: dispatcher integration points when they are
  infrastructure-level.
- `cli-checker`: dispatcher-boundary policy targets.

## Consumes From

- Hopium-specific contracts only through narrow bridge surfaces owned outside
  the dispatcher kernel.

## Does Not Own

- Hopium foundation primitives.
- Shared Hopium ABI/DTO/schema source of truth.
- Product runtime behavior.
- Data acquisition or ingestion workflows.
- Strategy authoring semantics.
- Backtest engine behavior.
- Trading decision or execution logic.
- CLI checker policy implementation.

