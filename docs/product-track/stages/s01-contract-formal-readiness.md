# S01 — Контрактный фундамент и formal readiness

**Status:** `advanced_in_repo`

**Purpose:** Стабилизировать compiled bundle, canonical hash, scenario/replay, formal IR, receipts и stale-check.

## Milestones

### M01.1 — Toolchain doctor

- **Status:** `exists_harden`
- **Outcome:** Проверка Rust, Cargo, Java, Alloy, P, Kani, Verus, Lean4, Z3, jq, python3, just; machine-readable report.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M01.2 — Formal install/provisioning

- **Status:** `exists_harden`
- **Outcome:** Reproducible установка/проверка инструментов, pin versions, SHA/checksum, offline/CI story.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M01.3 — Canonical serialization v1

- **Status:** `exists`
- **Outcome:** Canonical JSON, content_hash, bundle_hash, plan_hash, impact_set_hash, stable formatting.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M01.4 — Bundle formal input v0.2

- **Status:** `exists_harden`
- **Outcome:** CompiledDispatchBundle содержит predicates, profiles, routes, effects, claims, witness requirements, authz policies, formal obligations.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M01.5 — Scenario catalog v1

- **Status:** `exists_expand`
- **Outcome:** Executable *.scenario.yaml + generated traces + positive/negative controls.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M01.6 — Replay oracle strict bundle mode

- **Status:** `exists_expand`
- **Outcome:** Replay проверяет trace against bundle, plan_hash, witnesses, anchors, leases, lifecycle, authz, claims.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M01.7 — Receipts/stale-check v2

- **Status:** `exists_harden`
- **Outcome:** Codegen/tool-run receipts, generated artifact hashes, stale-check-all, coverage derivation.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

## Exit gate

`just formal-ready` и `just formal-verify-all` проходят в чистой среде; coverage matrix выводится из receipts, а не из prose.
