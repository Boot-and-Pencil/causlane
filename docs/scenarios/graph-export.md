# Graph export

M07.2 exports the CLI graph snapshot consumed by `why-blocked` /
`why-not-parallel` into one deterministic model rendered as JSON, Mermaid or DOT.

The exporter is an adapter: it delegates readiness and blockers to `GraphIndex`,
`select_frontier` and `why_not_parallel_from_index`. Witnesses and leases in the
snapshot are opaque labels for display; replay/barrier/capability validation
remains the authority for their correctness.

## Snapshot

```yaml
produced_facts: []
active_ops: [holder:0]
lanes:
  - lane_id: main
    capacity: unbounded
ops:
  - action_id: holder
    op_index: 0
    lane: main
    requires: []
    writes: [scope:one]
    witnesses: [witness-a]
    leases: [lease-a]
  - action_id: blocked
    op_index: 0
    lane: main
    requires: [fact:ready]
    writes: [scope:one]
```

## Commands

```bash
causlane graph export --graph graph.yaml --format json
causlane graph export --graph graph.yaml --format mermaid --op blocked:0
causlane graph export --graph graph.yaml --format dot --out graph.dot
```

## JSON shape

```json
{
  "schema_version": 1,
  "command": "graph export",
  "lanes": [
    { "lane_id": "main", "capacity": "unbounded" }
  ],
  "ops": [
    {
      "op": { "action_id": "blocked", "op_index": 0, "display": "blocked:0" },
      "lane": "main",
      "status": "blocked",
      "active": false,
      "ready": false,
      "requires": ["fact:ready"],
      "writes": ["scope:one"],
      "blockers": [
        { "kind": "blocked_on_fact", "fact": "fact:ready" },
        { "kind": "active_scope_conflict", "scope": "scope:one", "held_by": "holder:0" }
      ]
    }
  ],
  "edges": [
    {
      "kind": "active_scope_conflict",
      "from": "op:holder:0",
      "to": "op:blocked:0",
      "label": "active writer scope:one"
    }
  ]
}
```

## Text renderers

Mermaid starts with `flowchart TD` and labels each op with lane, status and
blocker/witness/lease annotations. DOT starts with `digraph causlane_graph` and
uses the same edge model.
