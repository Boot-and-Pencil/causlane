# S10 — Formal depth и proof hardening

**Status:** `active_next`

**Purpose:** Углубить доказательства после стабилизации ядра: больше invariants, scopes, interleavings, preservation/refinement.

## Milestones

### M10.1 — Invariant expansion

- **Status:** `exists`
- **Outcome:** Shared invariant-id catalog and planned reservations for
  I-011..I-020 without expanding coverage credit.
- **Definition of done:**
  - active invariant ids remain I-001..I-010 for bundle/Formal IR/coverage;
  - planned invariant ids I-011..I-020 are accepted only by the obligation
    manifest;
  - schemas reuse one invariant-id definition instead of copied regexes;
  - docs/ADR/FIR record that planned ids are not proof coverage.

### M10.2 — P interleavings depth

- **Status:** `exists_expand`
- **Outcome:** P-first bounded controls exist for duplicate retry execution,
  authz revocation before barrier and stale constraint epoch admission; other
  race families remain planned.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M10.3 — Kani integration

- **Status:** `exists_expand`
- **Outcome:** machine-readable Kani profile drives fixture, unwind bounds, output format and repo-local CI/nightly/manual lane entrypoints; deeper proof semantics remain expansion work.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M10.4 — Verus/Lean proof hardening

- **Status:** `exists_expand`
- **Outcome:** Verus and Lean4 proof lanes have a machine-readable always-blocking contract; deeper proof semantics remain expansion work.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M10.5 — Coverage anti-theatre

- **Status:** `exists_expand`
- **Outcome:** Coverage matrix Markdown is fully drift-checked from the receipt-derived report; active docs point to generated coverage instead of hand-maintained live inventories.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M10.6 — Proof/refinement docs

- **Status:** `exists_expand`
- **Outcome:** Schema-validated proof/refinement scope classifies claim strength, and generated Markdown drift is checked by the formal gate.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

## Exit gate

Coverage matrix показывает blocking proof lanes для релизных invariants, stale proofs блокируют release, exceptions имеют expiry и owner.
