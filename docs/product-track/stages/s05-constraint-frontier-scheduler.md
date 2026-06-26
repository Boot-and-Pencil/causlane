# S05 — Constraint Plane, graph/frontier и lanes/tiers

**Status:** `advanced_in_repo`

**Purpose:** Реализовать безопасную автоматическую диспетчеризацию параллельности на основе consequences, claims, leases, witnesses и constraints.

## Milestones

### M05.1 — Tier model

- **Status:** `done_or_near_done`
- **Outcome:** Admission/planning/dispatch/barrier/execution/observation/projection/closure as authority stages.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M05.2 — Lane model

- **Status:** `done_or_near_done`
- **Outcome:** Lane capacity/capability/fairness without semantic authority.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M05.3 — ConstraintSpec/Provider

- **Status:** `done_or_near_done`
- **Outcome:** Requirement/Claim/Lease, snapshots/epochs, Allow/Wait/Deny/Restrict decisions.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M05.4 — Graph indexes

- **Status:** `done_or_near_done`
- **Outcome:** wait_by_fact, wait_by_scope, active_by_write_scope, ready_by_lane, incremental rebuild.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M05.5 — Safe frontier selection

- **Status:** `done_or_near_done`
- **Outcome:** Ready antichain, no hard deps, no conflicts, lane/resource budgets.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M05.6 — Drain/fence protocol

- **Status:** `done_or_near_done`
- **Outcome:** domain/global drain epochs, safe points, disjoint domains, frozen sidecars.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M05.7 — Runtime constraint updates

- **Status:** `done_or_near_done`
- **Outcome:** capacity/quota/freeze/rate-limit updates with epoch, rebuild frontier, no truth rewrite.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M05.8 — why-not-parallel

- **Status:** `done_or_near_done`
- **Outcome:** Machine-readable blocker/rationale for concurrency decisions.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

## Exit gate

Dispatcher может объяснить `ready`, `blocked`, `why-not-parallel` и выдать safe antichain без конфликтующих mutable writes.
