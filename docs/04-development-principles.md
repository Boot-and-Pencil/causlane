# Development principles

## 1. Contract first

Docs, scenarios and formal models define the contract. Code implements it.

## 2. Small boring kernel

The kernel should be deterministic, reviewable and boring.

Complexity belongs in:

```text
registries;
planners;
adapters;
constraint providers;
policy engines;
projections;
formal/replay tooling.
```

## 3. No hidden authority surfaces

The following must not become truth sources:

```text
worker state;
queues;
logs;
metrics;
UI state;
projection tables;
execution graph;
policy decisions alone.
```

## 4. Plan is data

`ActionPlan` and `Op` are data. Side effects only happen through execution adapters after required barriers.

## 5. No hard effect without write-ahead

A hard effect must not start unless the required dispatch log, planned impact, barrier and leases are durably recorded.

## 6. Make illegal states unrepresentable where practical

Use typed IDs, enums and newtypes. Avoid stringly-typed protocol state in the kernel.

## 7. Every important decision must be explainable

The system should answer:

```text
why admitted?
why denied?
why waiting?
why this lane?
why not parallel?
which witness?
which lease?
which constraint epoch?
which plan hash?
```

## 8. Constraints may tighten, never weaken core invariants

Dynamic constraints, overlays and policy decisions may only restrict or add obligations. They must not remove dispatch log, barrier, witness or truth-anchor requirements.

## 9. Separate dependency from conflict

Dependency means B semantically requires A. Conflict means A and B cannot overlap but order is chosen dynamically.

## 10. Formal models must not become second truth

All formal projections should be generated from or explicitly tied to the same compiled bundle used by runtime/replay/tests.

## 11. Optimize hot path without bypassing correctness

Use compiled contracts, indexed scopes, sharded partitions, bounded queues and batched durable writes. Do not optimize by skipping barriers or audit.

## 12. Prefer gradual adoption

Support shadow mode, dry-run, simulation and replay before hard enforcement.
