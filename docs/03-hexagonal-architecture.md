# Hexagonal architecture

`causlane` should be structured so the kernel can be tested, replayed and formally modeled without real databases, queues, worker pools or network calls.

## Dependency direction

```text
Adapters -> Application -> Domain
```

Domain must not depend on adapters, storage, Tokio, SQL, HTTP, tracing or policy engines.

## Hexagon overview

```text
                            Driving adapters
                   CLI / HTTP / Axum / worker ingress
                                  |
                                  v
+------------------------------------------------------------------+
| Application layer                                                |
|                                                                  |
| Use cases:                                                       |
| - submit action                                                  |
| - compile plan                                                   |
| - admit to frontier                                              |
| - cross execution barrier                                        |
| - commit observed truth                                          |
| - replay trace                                                   |
| - explain blocker                                                |
|                                                                  |
| Ports:                                                          |
| - PlannerPort                                                    |
| - AuditLogPort                                                   |
| - ConstraintProviderPort                                         |
| - LeaseManagerPort                                               |
| - AuthorizerPort                                                 |
| - ExecutorPort                                                   |
| - ProjectionPort                                                 |
+------------------------------------------------------------------+
                                  |
                                  v
+------------------------------------------------------------------+
| Domain layer                                                     |
|                                                                  |
| Pure model:                                                      |
| - ActionCall                                                     |
| - ActionPlan                                                     |
| - Op                                                            |
| - EffectSignature                                                |
| - ConsequenceProfile                                             |
| - Lifecycle                                                      |
| - TransitionGuard                                                |
| - Witness                                                        |
| - Lease                                                          |
| - ConstraintDecision                                             |
| - DispatchDecision                                               |
+------------------------------------------------------------------+
                                  ^
                                  |
+------------------------------------------------------------------+
| Driven adapters                                                  |
|                                                                  |
| - SQLite/Postgres audit                                          |
| - Apalis/Fang job adapter                                        |
| - Restate/Temporal/Dapr/Conductor workflow adapters              |
| - Cedar/Casbin/AuthZEN/OpenFGA authz adapters                    |
| - tracing/OpenTelemetry observability                            |
| - filesystem/object-store bundle registry                        |
+------------------------------------------------------------------+
```

## Core crates

### `causlane-core`

Pure domain/application kernel. No async runtime requirement, no storage, no network.

Contains:

```text
value objects;
entities;
transition reducer;
consequence profile obligations;
ports traits;
use-case skeletons;
error types.
```

### `causlane-contracts`

Registry and compiled bundle model.

Contains:

```text
registry manifest;
bundle format;
compatibility checks;
schema/codegen hooks;
formal projection hooks.
```

### `causlane-runtime`

Runtime composition shell.

Contains:

```text
partition loops;
bounded queues;
constraint snapshots;
lane assignment;
lease-backed barrier flow;
adapter wiring.
```

### `causlane-replay`

Replay verifier.

Contains:

```text
trace reader;
protocol invariant checks;
witness/anchor validation;
normalized outcome comparison.
```

### `causlane-cli`

Developer/operator CLI.

Commands should eventually include:

```bash
causlane registry validate
causlane plan
causlane replay
causlane explain replay --bundle <bundle.json> --trace <trace.json> [--json]
causlane why-blocked --graph <graph.yaml|json> --op <action_id>:<op_index> [--json]
causlane why-not-parallel --graph <graph.yaml|json> --op <action_id>:<op_index> --with <action_id>:<op_index> [--json]
causlane graph export --graph <graph.yaml|json> --format <json|mermaid|dot> [--op <action_id>:<op_index>] [--out <path>]
```

## Port design rules

- Ports should expose semantic operations, not database/table/queue mechanics.
- Ports should return typed decisions and typed errors.
- Ports should not return prose-only rationale.
- Ports should be mockable without async runtime when possible.
- Hard-effect ports must require capabilities/leases, not raw job payloads.

## Adapter design rules

- Adapters must not weaken kernel invariants.
- Adapters must be certifiable by replay/contract tests.
- Adapters must preserve `action_id`, `plan_hash`, `audit_event_id`, `correlation_id` and witness references.
- Runtime adapters may fail independently, but audit failure for hard effects must fail closed.
