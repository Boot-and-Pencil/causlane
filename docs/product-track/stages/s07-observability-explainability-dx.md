# S07 — Observability, explainability и DX

**Status:** `advanced_in_repo`

**Purpose:** Сделать систему понятной и приятной: explain(), CLI, graph export, redaction, support bundles, docs/cookbook.

## Milestones

### M07.1 — CLI explain/why

- **Status:** `done_or_near_done`
- **Outcome:** explain, why-blocked, why-not-parallel, graph export, replay diagnostics.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M07.2 — Graph export

- **Status:** `done_or_near_done`
- **Outcome:** Mermaid/DOT/JSON graph slices with blockers/witnesses/leases.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M07.3 — Tracing connector

- **Status:** `done_or_near_done`
- **Outcome:** Structured action/op spans; logs derived, not truth.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M07.4 — OpenTelemetry optional

- **Status:** `done_or_near_done`
- **Outcome:** OTLP logs/traces/metrics adapter; fail-open for telemetry only.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M07.5 — Redaction policy

- **Status:** `done_or_near_done`
- **Outcome:** Audit/log/projection/replay/support-bundle redaction classes.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M07.6 — Support bundle

- **Status:** `done_or_near_done`
- **Outcome:** Sanitized bundle with trace, graph slice, route rationale, environment/tool report.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M07.7 — Cookbook docs

- **Status:** `done_or_near_done`
- **Outcome:** Add action, approval, conflict, drain, replay, adapter, authz, projection recipes.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

## Exit gate

Пользователь может понять, почему action запущен, заблокирован, не параллелится, требует approval/drain/lease, и воспроизвести это в replay.
