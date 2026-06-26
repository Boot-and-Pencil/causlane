# S08 — Runtime shell и adapters

**Status:** `active_next`

**Purpose:** Подключить execution backends и persistence без превращения Causlane в workflow engine.

## Milestones

### M08.1 — In-process runtime

- **Status:** `done_or_near_done`
- **Outcome:** Tokio partition loops, bounded queues, semaphores for capacity, no global lock.
- **Definition of done:**
  - adapter behavior is covered by certification tests;
  - no semantic authority leaks into adapter;
  - failure mode documented;
  - feature flag documented.

### M08.2 — Audit adapters

- **Status:** `done_or_near_done`
- **Outcome:** In-memory, SQLite, Postgres append-only audit; group commit policies.
- **Definition of done:**
  - adapter behavior is covered by certification tests;
  - no semantic authority leaks into adapter;
  - failure mode documented;
  - feature flag documented.

### M08.3 — Executor port/adapters

- **Status:** `done_or_near_done`
- **Outcome:** Tower-like Service port; hard effects only with capability.
- **Definition of done:**
  - adapter behavior is covered by certification tests;
  - no semantic authority leaks into adapter;
  - failure mode documented;
  - feature flag documented.

### M08.4 — Apalis adapter

- **Status:** `done_or_near_done`
- **Outcome:** Rust-native jobs backend; certification tests.
- **Definition of done:**
  - adapter behavior is covered by certification tests;
  - no semantic authority leaks into adapter;
  - failure mode documented;
  - feature flag documented.

### M08.5 — Restate adapter

- **Status:** `done_or_near_done`
- **Outcome:** Durable handler/workflow adapter; optional feature.
- **Definition of done:**
  - adapter behavior is covered by certification tests;
  - no semantic authority leaks into adapter;
  - failure mode documented;
  - feature flag documented.

### M08.6 — Temporal/Dapr/Conductor adapters

- **Status:** `future`
- **Outcome:** Experimental/community boundary; no hard dependency.
- **Definition of done:**
  - adapter behavior is covered by certification tests;
  - no semantic authority leaks into adapter;
  - failure mode documented;
  - feature flag documented.

### M08.7 — Adapter certification

- **Status:** `done_or_near_done`
- **Outcome:** Bounded certification matrix for existing adapters; retry/cancel/truth orchestration deferred.
- **Definition of done:**
  - adapter behavior is covered by certification tests;
  - no semantic authority leaks into adapter;
  - failure mode documented;
  - feature flag documented.

### M08.8 — Shadow mode

- **Status:** `done_or_near_done`
- **Outcome:** Runtime shadow comparer over `InProcessRuntimeEvent`; compare expected vs actual without enforcement.
- **Definition of done:**
  - shadow comparison is covered by runtime tests;
  - mismatch reporting cannot enforce, retry, admit, or schedule runtime work;
  - failure mode documented as diagnostic-only mismatch output;
  - feature flag documented as `tokio-runtime`.

## Exit gate

Есть in-process runtime, SQL audit, tracing adapter, Cedar adapter и один job/durable backend adapter, прошедшие adapter certification.
