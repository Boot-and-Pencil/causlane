# S06 — AuthZ, RBAC/ABAC/ReBAC, approvals и capabilities

**Status:** `advanced_in_repo`

**Purpose:** Сделать доступ частью dispatch protocol, а не middleware around endpoints.

## Milestones

### M06.1 — Authz policy model

- **Status:** `done_or_near_done`
- **Outcome:** AuthzPolicy, stages, policy_id/version, freshness/expiry, default deny.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M06.2 — Cedar adapter prototype

- **Status:** `done_or_near_done`
- **Outcome:** Embedded PDP mapping Predicate/Subject/Context to Cedar Principal/Action/Resource/Context.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M06.3 — Casbin/AuthZEN/OpenFGA sketches

- **Status:** `done_or_near_done`
- **Outcome:** Adapter contracts; do not bake one engine into core.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M06.4 — Approval as action

- **Status:** `done_or_near_done`
- **Outcome:** gate.approve/gate.deny bound to action_id+plan_hash+impact_set_hash.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M06.5 — Step-up and SoD

- **Status:** `done_or_near_done`
- **Outcome:** MFA/step-up, separation-of-duties, approval freshness.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M06.6 — Execution capability enforcement

- **Status:** `done_or_near_done`
- **Outcome:** Worker executes only with scoped capability derived from barrier.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M06.7 — Projection access/redaction

- **Status:** `done_or_near_done`
- **Outcome:** Read/projection authz, sensitive-field redaction policy.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

## Exit gate

RuntimeExecution action не пересекает barrier без свежего Allow, approval bound к action_id+plan_hash+impact_set_hash, executor работает только со scoped capability.
