//! Pure kernel contract surface (§7).
//!
//! These traits give the dispatcher's pure invariants a single named authority
//! so the formal lanes (Alloy/P/Kani/Verus) and the executable replay oracle
//! verify *the same* contract the runtime enforces, rather than parallel
//! re-implementations. Every method here is pure — no I/O — and delegates to the
//! domain primitives ([`reduce_lifecycle`], [`claim_modes_conflict`],
//! [`ExecutionCapability::derive_from_barrier`], [`projection_anchor_source_is_observed`]).
//!
//! [`KernelContracts`] is the canonical zero-sized implementation; consumers
//! should route conflict/lifecycle/capability/anchor decisions through it.

use crate::domain::{
    authz_gate, claim_modes_conflict, projection_anchor_source_is_observed, reduce_lifecycle,
    resolve_constraints, ActionId, AuditEventKind, AuthzDecisionRef, AuthzGateOutcome, AuthzPolicy,
    ConsequenceProfile, ConstraintDecision, ConstraintSnapshot, ExecutionBarrier,
    ExecutionCapability, ExecutionCapabilityError, LeaseRef, LifecycleStage, LifecycleViolation,
    PlanHash, ResourceClaim, Scope, Timestamp, TruthAnchor,
};

/// §7.5 — the lifecycle grammar every consumer reduces against. The same
/// grammar feeds replay, P, Kani and Verus generation, so a single transition
/// table is the authority for "which event is allowed in which stage".
pub trait LifecycleGrammar {
    /// The stage a fresh action starts in (before any event).
    fn initial_stage(&self, profile: ConsequenceProfile) -> LifecycleStage;

    /// Apply an event to the current stage, returning the next stage or a
    /// [`LifecycleViolation`] if the transition is forbidden under the profile.
    ///
    /// # Errors
    /// Returns [`LifecycleViolation::ForbiddenTransition`] for any
    /// `(stage, event, profile)` triple the grammar does not permit.
    fn reduce(
        &self,
        current: LifecycleStage,
        event: AuditEventKind,
        profile: ConsequenceProfile,
    ) -> Result<LifecycleStage, LifecycleViolation>;

    /// Whether a stage is terminal (no further events are accepted).
    fn is_terminal(&self, stage: LifecycleStage) -> bool;
}

/// §7.7 — scope overlap. The MVP is **exact equality**; hierarchical/prefix or
/// structured overlap can be plugged in later behind this same contract without
/// touching the conflict oracle.
pub trait ScopeOverlap {
    /// Whether two scopes overlap (MVP: equality).
    fn overlaps(&self, left: &Scope, right: &Scope) -> bool;
}

/// §7.7 — the lease/claim conflict oracle. `verified_merge` is the already
/// resolved merge decision (see `causlane-contracts` merge semantics); it is the
/// caller's responsibility to pass `true` only when a **verified** merge
/// protocol applies, so the oracle stays fail-closed by default.
///
/// I-006 single authority: both methods below decide a conflict through the one
/// shared mode rule [`claim_modes_conflict`] **and** this oracle's
/// [`ScopeOverlap::overlaps`] for the scope test. [`LeaseTable::grant`] takes a
/// `&impl ConflictOracle` and routes its decision through
/// [`leases_conflict`](ConflictOracle::leases_conflict), so the core lease
/// primitive cannot decide a conflict outside this authority — there is no
/// separate conflict path (nor a duplicated scope-equality) to keep in sync.
///
/// [`LeaseTable::grant`]: crate::LeaseTable::grant
/// [`claim_modes_conflict`]: crate::claim_modes_conflict
pub trait ConflictOracle: ScopeOverlap {
    /// Whether two resource claims conflict on the same resource + overlapping
    /// scope when at least one is exclusive and no verified merge applies.
    fn claims_conflict(
        &self,
        left: &ResourceClaim,
        right: &ResourceClaim,
        verified_merge: bool,
    ) -> bool {
        claim_modes_conflict(
            left.mode,
            right.mode,
            left.resource == right.resource,
            self.overlaps(&left.scope, &right.scope),
            verified_merge,
        )
    }

    /// Whether two granted leases conflict. Mirrors [`claims_conflict`] over the
    /// lease's resource/scope/mode.
    ///
    /// [`claims_conflict`]: ConflictOracle::claims_conflict
    fn leases_conflict(&self, left: &LeaseRef, right: &LeaseRef, verified_merge: bool) -> bool {
        claim_modes_conflict(
            left.mode,
            right.mode,
            left.resource == right.resource,
            self.overlaps(&left.scope, &right.scope),
            verified_merge,
        )
    }
}

/// §7.8 — capability issuer/validator. Executors must spend a derived
/// capability, never raw op permission; the capability structurally binds
/// action/plan/op/barrier/lease, so a forged id alone is insufficient.
pub trait CapabilityIssuer {
    /// Derive a scoped executor capability from a durable execution barrier.
    ///
    /// # Errors
    /// Returns [`ExecutionCapabilityError`] if the barrier does not cover the op
    /// or no barrier lease covers it.
    fn derive_capability(
        &self,
        barrier: &ExecutionBarrier,
        op_index: u32,
    ) -> Result<ExecutionCapability, ExecutionCapabilityError>;

    /// Validate that a capability was structurally derived from the barrier.
    ///
    /// # Errors
    /// Returns [`ExecutionCapabilityError`] on any barrier/action/plan/op/lease
    /// mismatch.
    fn validate_capability(
        &self,
        capability: &ExecutionCapability,
        barrier: &ExecutionBarrier,
    ) -> Result<(), ExecutionCapabilityError>;
}

/// §7.9 — truth anchor / projection resolver. A projection is only authority
/// when anchored to a prior `observed_truth.committed` for the matching
/// action/plan.
pub trait TruthAnchorResolver {
    /// Whether an event kind may serve as a projection's truth-anchor source.
    fn anchor_source_is_valid(&self, kind: AuditEventKind) -> bool;

    /// Whether a resolved anchor binds the same action and plan as its source.
    fn anchor_matches(
        &self,
        anchor: &TruthAnchor,
        source_action: &ActionId,
        source_plan: &PlanHash,
    ) -> bool;
}

/// §7.7 — drain fence acquisition. A fence may be acquired only when no lease is
/// *actively* holding an overlapping scope — the caller passes the active lease
/// set (un-released), and expired leases (`expires_at <= now`) do not block, so
/// the decision uses the active interval, not mere lease existence.
pub trait DrainSemantics: ScopeOverlap {
    /// Whether a drain fence over `fence_scope` can be acquired given the
    /// currently-held leases at time `now`.
    fn can_acquire_fence(
        &self,
        fence_scope: &Scope,
        active_leases: &[LeaseRef],
        now: Timestamp,
    ) -> bool {
        !active_leases.iter().any(|lease| {
            let expired = lease.expires_at.is_some_and(|expiry| expiry.0 <= now.0);
            !expired && self.overlaps(&lease.scope, fence_scope)
        })
    }
}

/// §7.10 — the deny-by-default authorization gate (ADR-0011). The single
/// structural+temporal authority both the live runtime and the replay oracle
/// evaluate against, so the two cannot drift; replay layers keyed-attestation and
/// event-structure checks on top of this decision (`causlane-core` stays
/// crypto-free). Backed by the shared [`classify_authz_decision`] classifier.
///
/// [`classify_authz_decision`]: crate::classify_authz_decision
pub trait AuthzEvaluator {
    /// Evaluate the gate for `required_stages`: each must carry an `Allow` bound to
    /// `action`/`plan`/`predicate_id`, issued under `expected_policy`, and not
    /// expired at `now`; otherwise the first unauthorized stage is denied.
    #[allow(clippy::too_many_arguments)]
    fn evaluate_authz(
        &self,
        required_stages: &[String],
        decisions: &[AuthzDecisionRef],
        action: &ActionId,
        plan: &PlanHash,
        predicate_id: &str,
        expected_policy: AuthzPolicy<'_>,
        now: Timestamp,
    ) -> AuthzGateOutcome;
}

/// §S05 — the constraint plane decision authority. Given an epoch-versioned
/// snapshot and a plan's claims, return whether the plan is allowed, must wait,
/// is denied, or is allowed with restrictions. A provider plugs in domain
/// resource/business state; the kernel default arbiter ([`resolve_constraints`])
/// enforces freeze / token-budget / active-lease-conflict / restrict uniformly.
pub trait ConstraintProvider {
    /// Resolve `claims` against `snapshot`.
    fn resolve(
        &self,
        snapshot: &ConstraintSnapshot,
        claims: &[ResourceClaim],
    ) -> ConstraintDecision;
}

/// The canonical kernel contract authority: a zero-sized value that implements
/// every contract above by delegating to the pure domain primitives. Replay,
/// codegen and runtime route their decisions through this one surface so the
/// formal lanes verify exactly what the runtime enforces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KernelContracts;

impl LifecycleGrammar for KernelContracts {
    fn initial_stage(&self, _profile: ConsequenceProfile) -> LifecycleStage {
        // Every profile's lifecycle starts un-admitted.
        LifecycleStage::New
    }

    fn reduce(
        &self,
        current: LifecycleStage,
        event: AuditEventKind,
        profile: ConsequenceProfile,
    ) -> Result<LifecycleStage, LifecycleViolation> {
        reduce_lifecycle(current, event, profile)
    }

    fn is_terminal(&self, stage: LifecycleStage) -> bool {
        stage == LifecycleStage::Closed
    }
}

impl ScopeOverlap for KernelContracts {
    fn overlaps(&self, left: &Scope, right: &Scope) -> bool {
        // MVP: exact equality. See ScopeOverlap docs for the extension point.
        left == right
    }
}

impl ConflictOracle for KernelContracts {}

impl DrainSemantics for KernelContracts {}

impl CapabilityIssuer for KernelContracts {
    fn derive_capability(
        &self,
        barrier: &ExecutionBarrier,
        op_index: u32,
    ) -> Result<ExecutionCapability, ExecutionCapabilityError> {
        ExecutionCapability::derive_from_barrier(barrier, op_index)
    }

    fn validate_capability(
        &self,
        capability: &ExecutionCapability,
        barrier: &ExecutionBarrier,
    ) -> Result<(), ExecutionCapabilityError> {
        capability.validate_for_barrier(barrier)
    }
}

impl TruthAnchorResolver for KernelContracts {
    fn anchor_source_is_valid(&self, kind: AuditEventKind) -> bool {
        projection_anchor_source_is_observed(kind)
    }

    fn anchor_matches(
        &self,
        anchor: &TruthAnchor,
        source_action: &ActionId,
        source_plan: &PlanHash,
    ) -> bool {
        anchor.action_id == *source_action && anchor.plan_hash == *source_plan
    }
}

impl AuthzEvaluator for KernelContracts {
    fn evaluate_authz(
        &self,
        required_stages: &[String],
        decisions: &[AuthzDecisionRef],
        action: &ActionId,
        plan: &PlanHash,
        predicate_id: &str,
        expected_policy: AuthzPolicy<'_>,
        now: Timestamp,
    ) -> AuthzGateOutcome {
        authz_gate(
            required_stages,
            decisions,
            action,
            plan,
            predicate_id,
            expected_policy,
            now,
        )
    }
}

impl ConstraintProvider for KernelContracts {
    fn resolve(
        &self,
        snapshot: &ConstraintSnapshot,
        claims: &[ResourceClaim],
    ) -> ConstraintDecision {
        resolve_constraints(snapshot, claims, self)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AuthzEvaluator, CapabilityIssuer, ConflictOracle, DrainSemantics, KernelContracts,
        LifecycleGrammar, ScopeOverlap, TruthAnchorResolver,
    };
    use crate::domain::ExecutionCapabilityError;
    use crate::domain::{
        AuditEventId, AuditEventKind, AuthzDecision, AuthzDecisionRef, AuthzDenyReason,
        AuthzGateOutcome, AuthzPolicy, ClaimMode, ConsequenceProfile, ConstraintEpoch,
        ExecutionBarrier, ImpactSetHash, LeaseId, LeaseRef, LifecycleStage, ResourceId, Scope,
        Timestamp,
    };
    use crate::{ActionId, PlanHash, PlanHashError};

    /// Typed error union for the contract tests so `?` composes the two domain
    /// error types without a stringly result. The variants carry the source
    /// error for the test harness's `Debug` report on failure.
    #[derive(Debug)]
    #[allow(dead_code)]
    enum TestError {
        PlanHash(PlanHashError),
        Capability(ExecutionCapabilityError),
    }

    impl From<PlanHashError> for TestError {
        fn from(err: PlanHashError) -> Self {
            TestError::PlanHash(err)
        }
    }

    impl From<ExecutionCapabilityError> for TestError {
        fn from(err: ExecutionCapabilityError) -> Self {
            TestError::Capability(err)
        }
    }

    type TestResult = Result<(), TestError>;

    fn plan_hash() -> Result<PlanHash, PlanHashError> {
        PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    }

    // §7.10: `evaluate_authz` routes the deny-by-default gate (ADR-0011) — a bound,
    // non-expired Allow authorizes the stage; an explicit Deny is refused. The
    // per-reason logic is covered by the `authz_gate` unit tests; this pins the
    // contract delegation.
    #[test]
    fn evaluate_authz_routes_the_gate() -> Result<(), PlanHashError> {
        let action = ActionId("act".to_owned());
        let ph = plan_hash()?;
        let stages = vec!["execution_barrier_logged".to_owned()];
        let policy = AuthzPolicy {
            id: "p",
            version: "1",
            max_age: None,
        };
        let decision = |verdict| AuthzDecisionRef {
            decision_event_id: AuditEventId("d".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: ph.clone(),
            predicate_id: "release.promote".to_owned(),
            actor: "alice".to_owned(),
            stage: "execution_barrier_logged".to_owned(),
            decision: verdict,
            policy_id: "p".to_owned(),
            policy_version: "1".to_owned(),
            issued_at: Timestamp(0),
            expires_at: Some(Timestamp(100)),
            attestation: None,
        };
        let allow = decision(AuthzDecision::Allow);
        assert_eq!(
            KernelContracts.evaluate_authz(
                &stages,
                std::slice::from_ref(&allow),
                &action,
                &ph,
                "release.promote",
                policy,
                Timestamp(10),
            ),
            AuthzGateOutcome::Allowed
        );
        let deny = decision(AuthzDecision::Deny);
        assert!(matches!(
            KernelContracts.evaluate_authz(
                &stages,
                std::slice::from_ref(&deny),
                &action,
                &ph,
                "release.promote",
                policy,
                Timestamp(10),
            ),
            AuthzGateOutcome::Denied {
                reason: AuthzDenyReason::Denied,
                ..
            }
        ));
        Ok(())
    }

    fn lease(
        scope: &str,
        mode: ClaimMode,
        expires_at: Option<u64>,
    ) -> Result<LeaseRef, PlanHashError> {
        Ok(LeaseRef {
            lease_id: LeaseId("l".to_owned()),
            resource: ResourceId("env_write".to_owned()),
            scope: Scope(scope.to_owned()),
            mode,
            amount: 1,
            holder_action_id: ActionId("a".to_owned()),
            holder_plan_hash: plan_hash()?,
            holder_op_index: Some(0),
            epoch: ConstraintEpoch(1),
            expires_at: expires_at.map(Timestamp),
            lease_event_id: AuditEventId("e".to_owned()),
        })
    }

    fn scope(value: &str) -> Scope {
        Scope(value.to_owned())
    }

    // §7.5: Closed is terminal; New is the initial stage for every profile.
    #[test]
    fn lifecycle_closed_is_terminal_and_new_is_initial() {
        let k = KernelContracts;
        assert_eq!(
            k.initial_stage(ConsequenceProfile::RuntimeExecution),
            LifecycleStage::New
        );
        assert!(k.is_terminal(LifecycleStage::Closed));
        assert!(!k.is_terminal(LifecycleStage::Executing));
    }

    // §7.5: RuntimeExecution cannot observe truth without prior execution.
    #[test]
    fn lifecycle_forbids_observe_without_execution() {
        let k = KernelContracts;
        let result = k.reduce(
            LifecycleStage::DispatchLogged,
            AuditEventKind::ObservedTruthCommitted,
            ConsequenceProfile::RuntimeExecution,
        );
        assert!(result.is_err());
    }

    // §7.7: scope overlap is exact-equality MVP.
    #[test]
    fn scope_overlap_is_equality_mvp() {
        let k = KernelContracts;
        assert!(k.overlaps(&scope("env:staging"), &scope("env:staging")));
        assert!(!k.overlaps(&scope("env:staging"), &scope("env:prod")));
    }

    // §7.7: merge default is no; two exclusive leases on the same scope conflict
    // unless a verified merge is passed.
    #[test]
    fn conflict_oracle_is_fail_closed_without_verified_merge() -> TestResult {
        let k = KernelContracts;
        let a = lease("env:staging", ClaimMode::ExclusiveWrite, None)?;
        let b = lease("env:staging", ClaimMode::ExclusiveWrite, None)?;
        assert!(k.leases_conflict(&a, &b, false));
        // A verified merge permits concurrency.
        assert!(!k.leases_conflict(&a, &b, true));
        // Different scopes do not conflict.
        let c = lease("env:prod", ClaimMode::ExclusiveWrite, None)?;
        assert!(!k.leases_conflict(&a, &c, false));
        // Two shared reads never conflict.
        let r1 = lease("env:staging", ClaimMode::SharedRead, None)?;
        let r2 = lease("env:staging", ClaimMode::SharedRead, None)?;
        assert!(!k.leases_conflict(&r1, &r2, false));
        Ok(())
    }

    // §7.7: drain uses the active interval — an expired lease does not block.
    #[test]
    fn drain_fence_ignores_expired_leases() -> TestResult {
        let k = KernelContracts;
        let fence = scope("env:staging");
        let active = lease("env:staging", ClaimMode::ExclusiveWrite, None)?;
        assert!(!k.can_acquire_fence(&fence, std::slice::from_ref(&active), Timestamp(10)));
        let expired = lease("env:staging", ClaimMode::ExclusiveWrite, Some(5))?;
        assert!(k.can_acquire_fence(&fence, std::slice::from_ref(&expired), Timestamp(10)));
        // No overlap with the fence scope never blocks.
        let elsewhere = lease("env:prod", ClaimMode::ExclusiveWrite, None)?;
        assert!(k.can_acquire_fence(&fence, std::slice::from_ref(&elsewhere), Timestamp(10)));
        Ok(())
    }

    // §7.9: only observed-truth is a valid anchor source.
    #[test]
    fn anchor_source_must_be_observed_truth() {
        let k = KernelContracts;
        assert!(k.anchor_source_is_valid(AuditEventKind::ObservedTruthCommitted));
        assert!(!k.anchor_source_is_valid(AuditEventKind::ProjectionEmitted));
    }

    // §7.8: a forged capability with a wrong barrier id fails validation.
    #[test]
    fn capability_validation_requires_structural_binding() -> TestResult {
        let k = KernelContracts;
        let barrier = ExecutionBarrier {
            barrier_id: AuditEventId("barrier".to_owned()),
            action_id: ActionId("a".to_owned()),
            plan_hash: plan_hash()?,
            op_indexes: vec![0],
            impact_set_hash: ImpactSetHash(
                "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                    .to_owned(),
            ),
            witnesses: Vec::new(),
            leases: vec![lease("env:staging", ClaimMode::ExclusiveWrite, None)?],
            authz_decision_refs: Vec::new(),
            constraint_snapshot_id: None,
        };
        let capability = k.derive_capability(&barrier, 0)?;
        assert!(k.validate_capability(&capability, &barrier).is_ok());
        // A capability pointing at a different barrier id is refused.
        let mut forged = capability;
        forged.barrier_event_id = AuditEventId("other".to_owned());
        assert!(k.validate_capability(&forged, &barrier).is_err());
        Ok(())
    }
}
