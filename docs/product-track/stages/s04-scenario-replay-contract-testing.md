# S04 — Сценарии, replay и contract testing

**Status:** `advanced_in_repo`

**Purpose:** Расширить executable scenario catalog и replay oracle до уровня основного devtool.

**Current state:** S04 закреплён в репозитории. Scenario catalog покрывает golden
success (включая conflict-free parallelism, projection и read-only sidecar через
mixed-predicate трейсы) и полную negative-серию; `causlane scenario compile`
эмитит trace + formal IR + replay expectation из одного источника; `replay verify
--explain` даёт точный invariant + причинное место; `causlane contract test`
(YAML-манифест) и mutation/fuzz harness держат оракул fail-closed. Exit-gate
этапа — «каждое новое predicate/feature требует scenario + replay expectation +
formal obligation/exemption» — обеспечивается этой машинерией (`scenario
compile`/`replay explain`/`contract test` + `formal-discipline-check`). Следующий
активный продуктовый этап — S05 constraint plane / frontier scheduler.

## Milestones

### M04.1 — Golden success scenarios

- **Status:** `done_or_near_done`
- **Outcome:** release_promote, approval, projection, conflict-free parallelism, read-only sidecar.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M04.2 — Negative scenario suite

- **Status:** `planned`
- **Outcome:** execution_without_barrier, observed_without_execution, projection_without_anchor, wrong plan, missing witness, conflicting leases.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M04.3 — Scenario-to-trace compiler

- **Status:** `done_or_near_done`
- **Outcome:** One scenario source emits trace, formal facts, replay expectation.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M04.4 — Replay explain output

- **Status:** `done_or_near_done`
- **Outcome:** Replay failures return exact invariant/error and causal location.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M04.5 — Contract testing harness

- **Status:** `done_or_near_done`
- **Outcome:** dispatch_test! / YAML runner for predicates and adapters.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M04.6 — Mutation/fuzz tests

- **Status:** `done_or_near_done`
- **Outcome:** Generate malformed traces and small worlds; ensure fail-closed.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

## Exit gate

Каждое новое predicate/feature требует scenario + replay expectation + formal obligation/exemption.
