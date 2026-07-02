# Parallel Development Readiness

Repository: `causlane`
Owner role: `generic-runtime`

This repository participates in the Hopium Stage 11 parallel-development
stabilization model. The model is intentionally interface-first:
`hopium-foundation` and `hopium-contracts` define the shared vocabulary,
and downstream repositories consume those surfaces directly.

cross-version translation layers, local DTO copies, and old/new
adapters are not accepted. The repository must either depend on the
frozen surface directly or keep the concern local and isolated.

Role:

Generic dispatcher kernel/runtime crate, not a Hopium product component.

Local checks:

- `scripts/parallel-dev/check_parallel_dev.py`
- `scripts/parallel-dev/check_version_set.py`
- `scripts/parallel-dev/check_touch_ownership.py`
- `scripts/parallel-dev/check_long_lane_registry.py`
- `scripts/parallel-dev/summarize_parallel_dev_readiness.py`

The checks are wired into `scripts/check-migration-gate.sh` and can also
be run through `make -f Makefile.parallel-dev parallel-dev-check`.
Runtime reports are written to `.agent-state/parallel-dev/`.
