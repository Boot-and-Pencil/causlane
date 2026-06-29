# S12 — Beta integrations и ecosystem validation

**Status:** `active_next`

**Purpose:** Проверить полезность на нескольких реальных интеграциях и улучшить DX/adapters.

## Milestones

### M12.1 — Reference integration 1

- **Status:** `done_or_near_done`
- **Outcome:** Rust service with API+worker+audit+projection.
- **Evidence:** `examples/reference-integration` runs an in-repo API+worker+audit+projection slice through public host-dispatch, runtime audit and guarded projection APIs; `docs/product-track/reference-integration-matrix.json` is checked by `tools/reference-integration-check`.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M12.2 — Reference integration 2

- **Status:** `done_or_near_done`
- **Outcome:** Agent/tool execution or CI/CD/release orchestration.
- **Evidence:** `examples/release-orchestration` runs a bounded CI/CD release orchestration graph through public host-dispatch and runtime audit APIs; `docs/product-track/reference-integration-matrix.json` is checked by `tools/reference-integration-check`.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M12.3 — Migration/shadow docs

- **Status:** `done_or_near_done`
- **Outcome:** How to adopt incrementally without rewrite.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.
- **Evidence:** `docs/scenarios/migration-shadow.md` and
  `docs/product-track/migration-shadow-adoption.json` document the bounded
  migration/shadow adoption path; `tools/migration-shadow-doc-check` pins the
  doc to existing shadow API and runtime negative-control tests.

### M12.4 — Adapter ecosystem

- **Status:** `done_or_near_done`
- **Outcome:** Document external adapter interface, compatibility/certification.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.
- **Evidence:** `docs/scenarios/adapter-ecosystem.md` and
  `docs/product-track/adapter-ecosystem-guide.json` document the external
  adapter interface and certification expectations; `tools/adapter-ecosystem-doc-check`
  pins those claims to the existing M08.7 certification matrix.

### M12.5 — DX feedback loop

- **Status:** `planned`
- **Outcome:** Simplify common paths, reduce ceremony, improve error messages.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M12.6 — Semver pre-1.0 freeze plan

- **Status:** `planned`
- **Outcome:** Identify APIs slated for stabilization.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

## Exit gate

Есть 2–3 reference integrations, migration/shadow-mode story и feedback-driven API hardening.
