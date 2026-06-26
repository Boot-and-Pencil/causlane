# Scenario: why two actions cannot run in parallel

## A

```yaml
predicate: environment.update_config
subject:
  environment: staging
effects:
  writes:
    - environment:staging
```

## B

```yaml
predicate: release.promote_candidate
subject:
  release_candidate_id: rc_123
  target_environment: staging
effects:
  writes:
    - environment:staging
    - release_candidate:rc_123
```

## Relation

There is no semantic dependency A -> B or B -> A.

There is a conflict:

```text
not overlap(A.execution_interval, B.execution_interval)
```

## CLI graph snapshot

M07.1 consumes a small typed snapshot. M07.2 reuses this input shape for
Mermaid/DOT/JSON graph exports:

```yaml
produced_facts: []
active_ops: []
lanes:
  - lane_id: default
    capacity: unbounded
ops:
  - action_id: environment_update
    op_index: 0
    lane: default
    requires: []
    writes: [environment:staging]
  - action_id: release_promote
    op_index: 0
    lane: default
    requires: []
    writes: [environment:staging, release_candidate:rc_123]
```

## Expected CLI JSON

```json
{
  "command": "why-not-parallel",
  "left": {
    "action_id": "environment_update",
    "op_index": 0,
    "display": "environment_update:0"
  },
  "right": {
    "action_id": "release_promote",
    "op_index": 0,
    "display": "release_promote:0"
  },
  "can_run_parallel": false,
  "pair_conflict": {
    "kind": "write_scope_conflict",
    "scope": "environment:staging"
  },
  "left_reasons": [],
  "right_reasons": [
    {
      "kind": "frontier_write_scope_conflict",
      "scope": "environment:staging",
      "with": "environment_update:0"
    }
  ]
}
```

The same relation in human output is:

```text
not parallel: environment_update:0 and release_promote:0
  - write_scope_conflict scope=environment:staging
  - right: frontier_write_scope_conflict scope=environment:staging with=environment_update:0
```

Legacy semantic shape:

```yaml
can_run_parallel: false
reason:
  kind: write_scope_conflict
  scope: environment:staging
relation: conflict_not_dependency
possible_resolutions:
  - serialize dynamically through lease order
  - define merge protocol
  - split write scopes if actually independent
  - request domain drain for exclusive action
```
