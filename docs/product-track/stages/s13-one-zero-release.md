# S13 — 1.0 release readiness

**Status:** `planned`

**Purpose:** Стабилизировать публичный API, semver, invariants, docs, formal/replay gates и эксплуатационные гарантии.

## Milestones

### M13.1 — API stabilization

- **Status:** `planned`
- **Outcome:** Core types, bundle format, event model, replay trace, adapter traits.
- **Definition of done:**
  - docs updated;
  - release/readiness gate passes;
  - limitations documented;
  - public API impact reviewed.

### M13.2 — Compatibility policy

- **Status:** `planned`
- **Outcome:** Schema migration, trace replay across versions, deprecation windows.
- **Definition of done:**
  - docs updated;
  - release/readiness gate passes;
  - limitations documented;
  - public API impact reviewed.

### M13.3 — Formal/replay release gate

- **Status:** `planned`
- **Outcome:** No release unless formal-ready, replay corpus, coverage matrix, exceptions valid.
- **Definition of done:**
  - docs updated;
  - release/readiness gate passes;
  - limitations documented;
  - public API impact reviewed.

### M13.4 — Production readiness docs

- **Status:** `planned`
- **Outcome:** Operations, performance tuning, failure modes, backup/restore audit.
- **Definition of done:**
  - docs updated;
  - release/readiness gate passes;
  - limitations documented;
  - public API impact reviewed.

### M13.5 — Security model docs

- **Status:** `planned`
- **Outcome:** Threat model, authz defaults, capability scope, redaction, supply chain.
- **Definition of done:**
  - docs updated;
  - release/readiness gate passes;
  - limitations documented;
  - public API impact reviewed.

### M13.6 — 1.0 release

- **Status:** `planned`
- **Outcome:** Publish stable crates and docs with honest guarantees.
- **Definition of done:**
  - docs updated;
  - release/readiness gate passes;
  - limitations documented;
  - public API impact reviewed.

## Exit gate

1.0 может быть выпущен без обещаний, которые ядро/формальные gates/replay/adapters не подтверждают.
