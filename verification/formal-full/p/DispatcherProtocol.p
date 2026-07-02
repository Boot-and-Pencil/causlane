// EXPLORATORY SKETCH — NOT AUTHORITATIVE (contains no executable P yet).
// Do not build this out by hand. Real P modeling is deferred until the contract
// is hardened and a generator exists; see ../../docs/11-contract-hardening-plan.md
// and ADR-0014. It should evolve from generated bundle projections, not by hand.

// Intended machines:
// - Dispatcher
// - AuditLog
// - LeaseManager
// - Worker
// - ProjectionBuilder
// - TestDriver

// Intended monitors:
// - NoExecutionBeforeBarrier
// - NoObservedBeforeExecution
// - NoProjectionWithoutAnchor
// - DrainBlocksNewMutableAdmission
// - RetryDoesNotDuplicateHardExecution
