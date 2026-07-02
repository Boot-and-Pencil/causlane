# Parallel Development Readiness Evidence

Repository: `causlane`
Owner role: `generic-runtime`

Current policy:

- foundation/contracts are the source of shared language and contracts;
- cross-version translation layers and local copies of shared DTO/schema vocabulary
  are forbidden;
- long-running lanes must be declared even when they are not executable
  yet;
- repo-local migration gate runs the parallel-development checks unless
  explicitly skipped.

Latest generated JSON reports live under `.agent-state/parallel-dev/`
and are intentionally not committed.
