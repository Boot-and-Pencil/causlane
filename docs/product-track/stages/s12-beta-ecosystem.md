# S12 — Beta integrations и ecosystem validation

**Status:** `active_next`

**Purpose:** Проверить полезность на нескольких реальных интеграциях и улучшить DX/adapters.

## Milestones

### M12.1 — Reference integration 1

- **Status:** `exists_harden`
- **Outcome:** Rust service with API+worker+audit+projection.
- **Evidence:** `examples/reference-integration` runs an in-repo API+worker+audit+projection slice through public host-dispatch, runtime audit and guarded projection APIs.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M12.2 — Reference integration 2

- **Status:** `planned`
- **Outcome:** Agent/tool execution or CI/CD/release orchestration.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M12.3 — Migration/shadow docs

- **Status:** `planned`
- **Outcome:** How to adopt incrementally without rewrite.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M12.4 — Adapter ecosystem

- **Status:** `planned`
- **Outcome:** Document external adapter interface, compatibility/certification.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

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
