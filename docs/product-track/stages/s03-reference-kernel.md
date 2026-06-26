# S03 — Reference kernel и executable semantics

**Status:** `advanced_in_repo`

**Purpose:** Превратить контракт в маленькое чистое ядро: lifecycle reducer, guards, frontier abstractions, replay oracle.

**Current state:** S03 закреплён в репозитории: `KernelContracts` является
единой runtime/replay authority для lifecycle, capability, lease/conflict, drain
и anchor decisions. Следующий активный продуктовый этап — S04 scenario/replay
contract testing.

## Milestones

### M03.1 — Pure lifecycle reducer

- **Status:** `done_or_near_done`
- **Outcome:** No async/I/O; transition(state,event,contracts)->decision; used by replay/runtime/tests.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M03.2 — Consequence profile obligations

- **Status:** `done_or_near_done`
- **Outcome:** RuntimeExecution/ProjectionRead/Oversight/Topology/Evidence obligations in code and bundle.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M03.3 — Effect signature core

- **Status:** `done_or_near_done`
- **Outcome:** reads/writes/produces/requires/invalidates/conflict_domains/claims.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M03.4 — Barrier/capability core

- **Status:** `done_or_near_done`
- **Outcome:** ExecutionBarrier, ExecutionCapability, scoped execution permission, executor guard.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M03.5 — LeaseTable core

- **Status:** `done_or_near_done`
- **Outcome:** Exclusive/shared/token leases, no-overlap, expiry/revocation, claim coverage.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M03.6 — Domain errors and stable codes

- **Status:** `done_or_near_done`
- **Outcome:** Typed errors with stable codes for replay/explain/CLI/tests.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

## Exit gate

Core reducer и validators можно использовать без runtime/adapters; все protocol-critical функции покрыты unit/property/Kani checks.
