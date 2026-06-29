# facade-kernel-ergonomics

Pre-alpha runnable example for downstream use of the `causlane` facade. It uses
only the facade crate dependency and exercises admission, barrier policy and
frontier selection through curated facade imports.

Flow:

```text
ActionCall -> admit_call -> requires_execution_barrier -> GraphIndex -> select_frontier
```

Run it from the repository root:

```bash
python3 tools/examples-check
```
