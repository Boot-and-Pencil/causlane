# Formal protocol catalog

> **Repository integration status:** proposed lifecycle discipline. This
> document is design/process authority only; current proof evidence remains the
> generated chain from compiled bundle and scenario through Formal IR, generated
> artifacts, receipts, stale-check and derived coverage.

This catalog names the protocols that must be modeled and, where appropriate, proved. The names are stable IDs used by Formal Impact Records and obligation manifests.

## Catalog target vs current implemented coverage

This protocol catalog names the long-term proof/check targets. It is not a live
coverage report. Current implemented coverage, including Lean4 theorem
applications, is the generated coverage matrix/report; Lean4 targets listed for
protocols are planned obligations until fresh proof/all receipts and the derived
coverage report say otherwise.

## PR-000 Authority pipeline protocol

**Participants:** registry compiler, scenario emitter, Formal IR builder, target generators, tool runners, stale checker, coverage reporter.

**State:**

```text
RegistrySource
  -> CompiledBundle(bundle_hash)
  -> ScenarioTrace(scenario_hash, optional bundle_hash)
  -> FormalIR(formal_ir_hash)
  -> GeneratedArtifact(artifact_hash)
  -> CodegenReceipt
  -> ToolRunReceipt
  -> CoverageReport
  -> DriftCheckedDocs
```

**Safety properties:** no generated artifact may be accepted for a different bundle/IR/scenario; no coverage cell may pass without real tool evidence; stale artifacts fail closed.

**Proof/check targets:** Kani for stale-check/report helpers; Lean4 for abstract authority relation; tooling for real exit codes.

## PR-001 Dispatch lifecycle protocol

**Events:**

```text
action.admitted
action.planned
dispatch.logged
execution.barrier_logged
execution.started
execution.completed
observed_truth.committed
projection.emitted
lifecycle.closed
```

**Safety properties:** execution requires prior barrier; observed truth requires prior execution; projection requires observed-truth anchor; closed is terminal.

**Proof/check targets:** replay, Alloy, P, Kani, Verus, Lean4.

## PR-002 Witness/attestation protocol

**Events:** `gate.approved`, `gate.denied`, evidence events, witness refs in barrier.

**Payload contract:**

```text
producer_event_id
producer_event_kind
producer_fact_kind
producer_scope
requirement_id
binds_to_action_id
binds_to_plan_hash
binds_to_impact_set_hash
```

**Safety properties:** a witness ref cannot assert a fact its producer did not attest; scope must resolve exactly; witness must be prior to barrier; plan/action/impact binding is exact.

**Proof/check targets:** replay exact oracle; Alloy/P grounding; Kani/Verus selector exactness; Lean4 exact-binding theorem.

## PR-003 Projection-anchor protocol

**Events:** `observed_truth.committed`, `projection.emitted`.

**Payload contract:** projection anchors include observed event id plus claimed fact kind and scope.

**Safety properties:** anchor source is observed truth; claimed anchor fact/scope equals observed event attestation; projection cannot become a new source of truth.

**Proof/check targets:** replay, Alloy/P grounding, Kani/Verus truth preservation, Lean4 anchor soundness.

## PR-004 Authz evidence protocol

**Events:** `authz.decision_recorded`, `execution.barrier_logged` referencing decision ids.

**State:**

```text
NoDecision -> Allow | Deny -> Expired | Stale
```

**Safety properties:** default deny; Deny wins; Allow must bind action/plan/predicate/stage/policy version; Allow must be fresh at barrier time and issued before barrier.

**Proof/check targets:** replay for temporal/policy authority; Alloy/P for structural Allow reference; Kani/Verus for policy resolver; Lean4 default-deny and Deny-wins theorem.

## PR-005 Lease/claim protocol

**Events:** `constraint.lease_granted`, `constraint.lease_released`, barrier lease refs.

**State:**

```text
Requested -> Granted(active) -> Released | Expired
```

**Safety properties:** every required claim has active lease coverage; released/expired leases cannot cover barrier; conflict predicate is fail-closed.

**Proof/check targets:** replay, P, Kani, Verus, Lean4.

## PR-006 Merge protocol

**State:**

```text
Absent
DeclaredButUnverified
Verified
Disabled
```

**Safety properties:** only `Verified` permits overlapping mutable writes; all other states conflict. A verified merge protocol must eventually carry algebraic laws for its domain.

**Lean4 proof obligations:** associativity, commutativity where required, idempotence where required, identity/absorber laws if declared, monotonicity with respect to observed-truth order.

**Verus proof obligations:** the Rust `MergeSemantics` resolver returns `Mergeable` only for verified applicability.

## PR-007 Drain protocol

**Events:** `drain.fence_requested`, `drain.fence_acquired`, lease release/expiry.

**Safety properties:** a drain fence for a scope may be acquired only when prior overlapping active mutable leases have cleared; once drain is pending, new mutable admissions in that scope are blocked according to policy.

**Proof/check targets:** replay negative controls, P interleavings, Kani drain predicate, Lean4 temporal theorem if drain becomes complex.

## PR-008 Execution capability protocol

**State:**

```text
BarrierLogged -> CapabilityDerived -> CapabilitySpent -> ExecutionStarted
```

**Payload contract:** capability id, barrier event id, action id, plan hash, op index, lease ids, expiration, optional attestation.

**Safety properties:** capability id is canonical for barrier+op; capability validates against same barrier; named leases are in barrier; expired capability fails; hard executor accepts only capability, not raw barrier/leases.

**Proof/check targets:** Kani/Verus for derive/validate; replay/Alloy/P for structural binding; Lean4 single-origin capability theorem.

## PR-009 Replay protocol

**Input:** compiled bundle + trace, strict bundle hash required for authoritative verification.

**Safety properties:** if replay accepts strictly, trace satisfies all replay-modeled protocol invariants for that bundle; every failure maps to stable error code.

**Proof/check targets:** replay negative controls, Kani bounded reducer checks, Verus/Lean4 soundness theorem over abstract trace.

## PR-010 Scenario/negative-control protocol

**State:**

```text
Catalogued -> EmittedTrace -> ExpectedPass | ExpectedFail(error_code) -> GateResult
```

**Safety properties:** each negative control must be refuted by the responsible lane; a passing invalid scenario is a gate failure; exact error code drift is a gate failure.

## PR-011 Exception/waiver protocol

**State:**

```text
Requested -> Approved(expiry, allowed_profiles, forbidden_profiles) -> Expired
```

**Safety properties:** exceptions cannot apply to forbidden profiles; expired exception fails; exception cannot mark a lane as passed, only `non_blocking_skipped` or equivalent honest status.

## PR-012 Feature/fix protocol

**State:**

```text
ChangeIntent -> FormalImpactRecord -> ObligationDelta -> NegativeControl -> ModelDelta -> CodeDelta -> Gate
```

**Safety properties:** protocol-critical code cannot be merged before obligation/model delta exists; bug fixes that affect safety add failing negative control first.

## PR-013 Migration/deprecation protocol

**State:**

```text
OldSchema -> CompatibilityMap -> MigratedSchema -> OldReceiptsStaleOrRefined
```

**Safety properties:** migration preserves invariant interpretation or intentionally invalidates old receipts; no field removal can silently make a proof vacuous.
