# S02 — Формальные модели v1

**Status:** `advanced_in_repo`

**Purpose:** Сделать Alloy/P/Kani/Verus/Lean4 не демонстрацией, а рабочим verification contour для ключевых инвариантов.

## Milestones

### M02.1 — Alloy structural checks v1

- **Status:** `exists_harden`
- **Outcome:** Generated facts + generic core assertions for lifecycle, anchors, route/profile, conflict frontier, witness/approval binding.
- **Definition of done:**
  - generated artifact exists;
  - tool-run receipt exists;
  - stale-check passes;
  - negative control exists where applicable;
  - coverage matrix can cite concrete check_id.

### M02.2 — P protocol monitors v1

- **Status:** `exists_harden`
- **Outcome:** Generated monitors for barrier-before-execution, drain/admission races, retry/idempotency, lease release/expiry.
- **Definition of done:**
  - generated artifact exists;
  - tool-run receipt exists;
  - stale-check passes;
  - negative control exists where applicable;
  - coverage matrix can cite concrete check_id.

### M02.3 — Kani bounded harnesses v1

- **Status:** `exists_harden`
- **Outcome:** Generated/handwritten harnesses for reducers, trace validation, lease table, capability derivation, parser boundaries.
- **Definition of done:**
  - generated artifact exists;
  - tool-run receipt exists;
  - stale-check passes;
  - negative control exists where applicable;
  - coverage matrix can cite concrete check_id.

### M02.4 — Verus proof facet v1

- **Status:** `exists_harden`
- **Outcome:** Abstract preservation proofs: lifecycle, overlay monotonicity, replay soundness, lease map invariants.
- **Definition of done:**
  - generated artifact exists;
  - tool-run receipt exists;
  - stale-check passes;
  - negative control exists where applicable;
  - coverage matrix can cite concrete check_id.

### M02.5 — Lean4 proof applications v1

- **Status:** `exists_harden`
- **Outcome:** Generated scenario-bound theorem applications checked in the full formal gate.
- **Definition of done:**
  - generated artifact exists;
  - tool-run receipt exists;
  - stale-check passes;
  - negative control exists where applicable;
  - coverage matrix can cite concrete check_id.

### M02.6 — Negative controls discipline

- **Status:** `exists_harden`
- **Outcome:** Each invariant lane has expected-failure controls; accidental failure is not evidence.
- **Definition of done:**
  - generated artifact exists;
  - tool-run receipt exists;
  - stale-check passes;
  - negative control exists where applicable;
  - coverage matrix can cite concrete check_id.

### M02.7 — Formal exceptions policy

- **Status:** `exists_harden`
- **Outcome:** Every allowed formal-lane exception has owner, rationale, expiry, profile, allowed scope.
- **Definition of done:**
  - generated artifact exists;
  - tool-run receipt exists;
  - stale-check passes;
  - negative control exists where applicable;
  - coverage matrix can cite concrete check_id.

## Exit gate

Для I-001/I-002/I-003/I-006/I-008/I-009 есть generated artifact + tool receipt + stale-check + negative controls.
