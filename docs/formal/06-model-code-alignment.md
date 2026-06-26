# Model/code alignment contract

> **Repository integration status:** proposed lifecycle discipline. This
> document is design/process authority only; current proof evidence remains the
> generated chain from compiled bundle and scenario through Formal IR, generated
> artifacts, receipts, stale-check and derived coverage.

## Goal

Make it mechanically hard for models to prove one thing while code implements another.

## Alignment chain

```text
Rust/domain contract types
  -> Registry manifest and compiled bundle
  -> Replay scenario and trace lowering
  -> Formal IR
  -> Target generators
  -> Tool receipts
  -> Coverage report
  -> Documentation projection
```

Any link may fail closed. No link may infer protocol facts from prose.

## Source-of-truth surfaces

| Surface | Authority for |
|---|---|
| `CompiledDispatchBundle` | predicate contracts, routes, profiles, policies, claims, obligations |
| `ReplayTrace` / `ReplayScenario` | concrete histories and negative controls |
| `FormalIr` | target-neutral model input |
| `GeneratedArtifact` | target-specific model facts/checks |
| `FormalReceipt` | provenance and tool result |
| `FormalCoverageReport` | derived lane/invariant status |
| `docs/invariants/coverage-matrix.json` | drift-checked projection, not independent authority |
| `docs/formal/proof-refinement-scope.json` | schema-validated claim-strength classification, not coverage credit |

## Required trait/contract additions

The 009 baseline already has `FormalIrBuilder`, `FormalGenerator`, `StaleChecker` and `CoverageReporter`. Add or formalize these contracts to tighten the lifecycle:

### `FormalObligationProvider`

```rust
pub trait FormalObligationProvider {
    fn obligations(&self) -> &[FormalObligation];
    fn obligation_ids_for_path(&self, changed_path: &str) -> Vec<FormalObligationId>;
}
```

Purpose: map changed files and model IDs to required gates.

### `ProtocolCriticalChangeDetector`

```rust
pub trait ProtocolCriticalChangeDetector {
    fn classify(&self, changed_files: &[PathBuf]) -> FormalImpactClass;
}
```

Purpose: make feature/fix gating automatic.

### `InvariantClaimValidator`

```rust
pub trait InvariantClaimValidator {
    fn validate_claims(&self, report: &FormalCoverageReport, docs: &CoverageMatrix) -> Result<(), DriftError>;
}
```

Purpose: reject docs overclaim.

### `ReplayOracle`

```rust
pub trait ReplayOracle {
    fn verify_strict(&self, bundle: &CompiledDispatchBundle, trace: &ReplayTrace) -> Result<ReplayVerdict, ReplayError>;
}
```

Purpose: make replay the executable contract surface for codegen/coverage, not an ad-hoc CLI behavior.

### `WitnessEvidenceResolver`

```rust
pub trait WitnessEvidenceResolver {
    fn resolve_required_witness(&self, requirement: &RequiredWitness, trace: &TraceIndex) -> Result<WitnessEvidence, EvidenceError>;
}
```

Must guarantee exact producer-grounded fact kind/scope/action/plan/impact binding.

### `ProjectionAnchorResolver`

```rust
pub trait ProjectionAnchorResolver {
    fn resolve_anchor(&self, anchor: &AnchorRef, trace: &TraceIndex) -> Result<ObservedTruthAnchor, AnchorError>;
}
```

Must reject projection anchors that do not point to observed truth or claim unattested fact/scope.

### `AuthzEvidenceResolver`

```rust
pub trait AuthzEvidenceResolver {
    fn resolve_decision(&self, selector: &AuthzSelector, trace: &TraceIndex, at: Timestamp) -> Result<AuthzDecisionEvidence, AuthzError>;
}
```

Must implement default-deny, Deny-wins, policy/version/stage/freshness/expiry.

### `CapabilityIssuer`

```rust
pub trait CapabilityIssuer {
    fn derive(&self, barrier: &ExecutionBarrier, op_index: u32) -> Result<ExecutionCapability, CapabilityError>;
    fn validate(&self, capability: &ExecutionCapability, barrier: &ExecutionBarrier) -> Result<(), CapabilityError>;
}
```

Must use canonical capability id and exact lease coverage.

### `ConflictOracle`, `MergeSemantics`, `DrainSemantics`

These contracts must share the same fail-closed answer: non-verified merge protocol never clears conflict; drain cannot be acquired over active overlap.

### `Lean4Generator`

```rust
pub trait Lean4Generator: FormalGenerator {
    fn theorem_inventory(&self, ir: &FormalIr) -> Vec<ProofObligation>;
}
```

Purpose: introduce Lean4 without a hand-maintained second specification.

### `FormalDisciplineGate`

```rust
pub trait FormalDisciplineGate {
    fn check(&self, input: DisciplineInput) -> Result<DisciplineReport, DisciplineError>;
}
```

Purpose: encode anti-theatre policy in a testable library rather than only a shell script.

## Crate placement

Recommended boundaries:

```text
causlane-core       : pure domain predicates and proof-friendly reducers
causlane-contracts  : bundle/registry/canonicalization/merge semantics
causlane-replay     : executable trace oracle and stable errors
causlane-codegen    : Formal IR and target generators
causlane-formal     : doctor/discipline/coverage/proof orchestration pure logic
causlane-cli        : filesystem/process boundary
```

Optional future split:

```text
causlane-formal-ir
causlane-formal-alloy
causlane-formal-p
causlane-formal-kani
causlane-formal-verus
causlane-formal-lean4
```

Do not split until code size or dependency isolation requires it.

## Check ID discipline

Every coverage claim must have a stable `check_id`:

```text
replay:projection_without_anchor_invalid
alloy:GeneratedAnchorFactGrounded
p:NoExecutionBeforeBarrier
kani:execution_requires_prior_barrier_nondet
verus:execution_started_requires_prior_barrier
lean4:valid_trace_execution_started_has_prior_barrier
```

A generator must expose obligations it actually emits. If a check ID is renamed or removed, tests must fail before docs can overclaim.

## Model adequacy relation

The desired long-term theorem stack:

```text
Concrete bundle/scenario
  lowers_to Formal IR
  generates target facts
  target facts instantiate abstract protocol theorem
  tool proves/checks target facts
  receipts bind hashes
  coverage derives claim
```

Lean4 should prove the abstract part; Verus/Kani/replay should connect Rust implementations to the abstract relation.
