# Formal Impact Record: M10.3 Kani Lane Entrypoint

Date: 2026-06-24

## Summary

This change adds a provider-neutral formal lane entrypoint for M10.3. The Kani
profile remains the source of truth for lane names and unwind bounds, while
`tools/formal-verify-lane` gives CI/nightly/manual callers a stable command that
delegates to the existing `formal-verify-all` runner.

## Changed Surface

- `tools/formal-verify-lane` validates lane names from `formal/kani/profile.json`
  and delegates real execution to `tools/formal-verify-all --lane`.
- `tools/formal-doctor` reads valid lanes from the same profile.
- The Rust CLI doctor no longer duplicates the concrete non-local lane list:
  `local_smoke` is the local lane, and every other lane uses the publication
  contract checks.

## Coverage Effect

No active coverage changes. This is runner/interface wiring only; Kani proof
semantics and receipt-derived coverage remain unchanged.

## Validation

- `formal-verify-lane --dry-run` for local, CI, nightly and manual lanes
- `formal-doctor` JSON checks for local and remote lanes
- Rust unit tests for lane classification
- `formal-verify-all`
- coverage-matrix check
