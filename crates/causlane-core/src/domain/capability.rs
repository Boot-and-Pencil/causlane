//! Execution-capability derivation and validation (ADR-0013).
//!
//! Split from `constraint.rs` for the 800-line file cap. A capability is the
//! scoped token an executor spends instead of a raw lease array; it is only
//! valid when it was derived from a durable barrier for the same
//! action/plan/op, names only barrier leases, and carries the **canonical**
//! capability id for that barrier+op (so a forged id cannot be spent).

use super::{CapabilityId, ExecutionBarrier, ExecutionCapability, LeaseId, Timestamp};

/// Pure predicate for the binding portion of capability validation. The caller
/// supplies equality/coverage facts after resolving concrete ids and op sets.
///
/// The five boolean facts mirror the formal capability-binding truth table
/// (barrier/action/plan/op/lease), so they are intentionally explicit rather
/// than packed into a struct.
#[must_use]
#[allow(clippy::fn_params_excessive_bools)]
pub fn capability_binding_matches(
    barrier_event_matches: bool,
    action_matches: bool,
    plan_matches: bool,
    op_index_covered: bool,
    leases_present: bool,
) -> bool {
    barrier_event_matches && action_matches && plan_matches && op_index_covered && leases_present
}

/// The canonical execution-capability id for a barrier + op (ADR-0013): a
/// capability is only valid if its id is exactly this. Single source of truth
/// for [`ExecutionCapability::derive_from_barrier`] and `validate_for_barrier`.
#[must_use]
pub fn canonical_capability_id(barrier: &ExecutionBarrier, op_index: u32) -> CapabilityId {
    CapabilityId(format!("cap:{}:{op_index}", barrier.barrier_id.0))
}

/// Execution-capability derivation/validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionCapabilityError {
    /// The requested op index is not covered by the barrier.
    OpNotCovered {
        /// Requested op index.
        op_index: u32,
    },
    /// No lease in the barrier covers the requested op.
    LeaseCoverageMissing {
        /// Requested op index.
        op_index: u32,
    },
    /// The capability does not point at the checked barrier.
    BarrierMismatch {
        /// Capability id.
        capability_id: CapabilityId,
    },
    /// The capability action/plan/op binding does not match the barrier.
    BindingMismatch {
        /// Capability id.
        capability_id: CapabilityId,
    },
    /// The capability names a lease absent from the barrier.
    LeaseNotInBarrier {
        /// Missing lease id.
        lease_id: LeaseId,
    },
    /// The capability id is not the canonical id derived from the barrier+op, so
    /// a forged/relabelled capability id was supplied even though the structural
    /// binding matched.
    CapabilityIdMismatch {
        /// Canonical id expected for this barrier+op.
        expected: CapabilityId,
        /// Id actually carried by the capability.
        actual: CapabilityId,
    },
}

/// Why a worker must refuse to spend a presented capability at execution time
/// (M06.6). Distinct from [`ExecutionCapabilityError`], which covers
/// derivation/structural validation: this is the *spend-time* admission verdict the
/// executor runs, adding op-targeting and temporal liveness on top of
/// [`ExecutionCapability::validate_for_barrier`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilitySpendRefusal {
    /// The capability failed structural/binding validation against the barrier
    /// (carries the underlying cause). Deny-wins, highest precedence.
    NotBoundToBarrier {
        /// The structural validation cause.
        cause: ExecutionCapabilityError,
    },
    /// The capability is valid for the barrier but is scoped to a different op than
    /// the one the worker intends to run (no op-substitution).
    OpMismatch {
        /// Op index the capability authorizes.
        capability_op: u32,
        /// Op index the worker intends to execute.
        requested_op: u32,
    },
    /// The capability's lease-derived expiry is at or before the spend instant — the
    /// capability is dead even if the barrier was authorized.
    Expired {
        /// The capability's expiry.
        expires_at: Timestamp,
        /// The spend instant.
        now: Timestamp,
    },
}

/// The execution a worker intends to perform, presented for capability admission
/// (M06.6). A named borrow struct (not positional args) keeps the spend call site
/// self-documenting.
#[derive(Clone, Copy, Debug)]
pub struct CapabilitySpendRequest<'a> {
    /// The durable barrier the capability claims to derive from.
    pub barrier: &'a ExecutionBarrier,
    /// The op index the worker intends to execute.
    pub requested_op: u32,
    /// The spend instant for expiry evaluation.
    pub now: Timestamp,
}

impl ExecutionCapability {
    /// Derive a scoped executor capability from a durable execution barrier.
    ///
    /// # Errors
    /// Returns [`ExecutionCapabilityError`] if the barrier does not cover the
    /// requested op or no barrier lease covers that op.
    #[must_use = "capability derivation errors must be handled"]
    pub fn derive_from_barrier(
        barrier: &ExecutionBarrier,
        op_index: u32,
    ) -> Result<Self, ExecutionCapabilityError> {
        if !barrier.op_indexes.contains(&op_index) {
            return Err(ExecutionCapabilityError::OpNotCovered { op_index });
        }
        let lease_ids = barrier
            .leases
            .iter()
            .filter(|lease| lease.holder_op_index.is_none_or(|index| index == op_index))
            .map(|lease| lease.lease_id.clone())
            .collect::<Vec<_>>();
        if lease_ids.is_empty() {
            return Err(ExecutionCapabilityError::LeaseCoverageMissing { op_index });
        }
        let expires_at = barrier
            .leases
            .iter()
            .filter_map(|lease| lease.expires_at)
            .min();
        Ok(Self {
            capability_id: canonical_capability_id(barrier, op_index),
            action_id: barrier.action_id.clone(),
            plan_hash: barrier.plan_hash.clone(),
            op_index,
            barrier_event_id: barrier.barrier_id.clone(),
            lease_ids,
            expires_at,
            // The minting kernel attaches the keyed attestation separately (it
            // holds the secret); derivation itself is key-agnostic.
            attestation: None,
        })
    }

    /// Canonical bytes a kernel attests over (HMAC) when minting this capability.
    /// Each field is length-prefixed so the concatenation is injective — no field
    /// can be shifted into an adjacent one. Excludes the attestation itself.
    #[must_use]
    pub fn attestation_message(&self) -> Vec<u8> {
        use core::fmt::Write as _;
        let lease_ids = self
            .lease_ids
            .iter()
            .map(|lease| lease.0.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let expires = self
            .expires_at
            .map_or_else(|| "none".to_owned(), |timestamp| timestamp.0.to_string());
        let op_index = self.op_index.to_string();
        let mut msg = String::from("causlane-capability-attestation-v1");
        for field in [
            self.capability_id.0.as_str(),
            self.action_id.0.as_str(),
            self.plan_hash.as_str(),
            op_index.as_str(),
            self.barrier_event_id.0.as_str(),
            lease_ids.as_str(),
            expires.as_str(),
        ] {
            let _written = write!(msg, "\u{1f}{}:{field}", field.len());
        }
        msg.into_bytes()
    }

    /// Validate that this capability was derived from the supplied barrier.
    ///
    /// # Errors
    /// Returns [`ExecutionCapabilityError`] on any barrier/action/plan/op/lease
    /// or canonical-id mismatch.
    #[must_use = "capability validation errors must be handled"]
    pub fn validate_for_barrier(
        &self,
        barrier: &ExecutionBarrier,
    ) -> Result<(), ExecutionCapabilityError> {
        if self.barrier_event_id != barrier.barrier_id {
            return Err(ExecutionCapabilityError::BarrierMismatch {
                capability_id: self.capability_id.clone(),
            });
        }
        if !capability_binding_matches(
            true,
            self.action_id == barrier.action_id,
            self.plan_hash == barrier.plan_hash,
            barrier.op_indexes.contains(&self.op_index),
            !self.lease_ids.is_empty(),
        ) {
            return Err(ExecutionCapabilityError::BindingMismatch {
                capability_id: self.capability_id.clone(),
            });
        }
        for lease_id in &self.lease_ids {
            let covered = barrier
                .leases
                .iter()
                .any(|lease| lease.lease_id == *lease_id);
            if !covered {
                return Err(ExecutionCapabilityError::LeaseNotInBarrier {
                    lease_id: lease_id.clone(),
                });
            }
        }
        // Canonical id: even when the structural binding matches, the capability
        // id itself must be the one derived from this barrier+op, so a forged or
        // relabelled id cannot be spent.
        let expected_id = canonical_capability_id(barrier, self.op_index);
        if self.capability_id != expected_id {
            return Err(ExecutionCapabilityError::CapabilityIdMismatch {
                expected: expected_id,
                actual: self.capability_id.clone(),
            });
        }
        Ok(())
    }

    /// Fail-closed admission a worker runs before spending this capability (M06.6):
    /// the executor must execute **only** with a capability that is structurally
    /// bound to the barrier, scoped to the exact op it is about to run, and live at
    /// `req.now`.
    ///
    /// Precedence (deny-wins, first failure returned):
    /// 1. [`CapabilitySpendRefusal::NotBoundToBarrier`] — [`Self::validate_for_barrier`]
    ///    must pass.
    /// 2. [`CapabilitySpendRefusal::OpMismatch`] — `op_index` must equal
    ///    `req.requested_op`.
    /// 3. [`CapabilitySpendRefusal::Expired`] — any `expires_at <= req.now` refuses.
    ///
    /// The temporal check enforces the **lease-derived** expiry (the minimum of the
    /// barrier's lease expiries, set by [`Self::derive_from_barrier`]) that the
    /// authorization gate does not look at — authz judges the *decision's* freshness,
    /// this judges the *capability's*. `Ok(())` authorizes exactly this spend.
    ///
    /// # Errors
    /// Returns the first [`CapabilitySpendRefusal`] per the precedence above.
    #[must_use = "a capability spend verdict must be enforced"]
    pub fn spend_admits(
        &self,
        req: CapabilitySpendRequest<'_>,
    ) -> Result<(), CapabilitySpendRefusal> {
        self.validate_for_barrier(req.barrier)
            .map_err(|cause| CapabilitySpendRefusal::NotBoundToBarrier { cause })?;
        if self.op_index != req.requested_op {
            return Err(CapabilitySpendRefusal::OpMismatch {
                capability_op: self.op_index,
                requested_op: req.requested_op,
            });
        }
        if let Some(expires_at) = self.expires_at {
            if expires_at <= req.now {
                return Err(CapabilitySpendRefusal::Expired {
                    expires_at,
                    now: req.now,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_capability_id, CapabilitySpendRefusal, CapabilitySpendRequest,
        ExecutionCapabilityError,
    };
    use crate::domain::{
        ActionId, AuditEventId, CapabilityId, ClaimMode, ConstraintEpoch, ExecutionBarrier,
        ExecutionCapability, ImpactSetHash, LeaseId, LeaseRef, ResourceId, Scope, Timestamp,
    };
    use crate::{PlanHash, PlanHashError};

    /// Typed error union so `?` composes the two error types in the test.
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

    fn barrier() -> Result<ExecutionBarrier, PlanHashError> {
        let plan = PlanHash::new(
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
        )?;
        Ok(ExecutionBarrier {
            barrier_id: AuditEventId("evt_barrier".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: plan.clone(),
            op_indexes: vec![0],
            impact_set_hash: ImpactSetHash(
                "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                    .to_owned(),
            ),
            witnesses: Vec::new(),
            leases: vec![LeaseRef {
                lease_id: LeaseId("l".to_owned()),
                resource: ResourceId("r".to_owned()),
                scope: Scope("s".to_owned()),
                mode: ClaimMode::ExclusiveWrite,
                amount: 1,
                holder_action_id: ActionId("act".to_owned()),
                holder_plan_hash: plan,
                holder_op_index: Some(0),
                epoch: ConstraintEpoch(0),
                expires_at: None,
                lease_event_id: AuditEventId("evt_leases".to_owned()),
            }],
            authz_decision_refs: Vec::new(),
            constraint_snapshot_id: None,
        })
    }

    // A derived capability validates; a capability whose id is forged (but whose
    // structural binding still matches) is rejected with CapabilityIdMismatch.
    #[test]
    fn forged_capability_id_is_rejected() -> Result<(), TestError> {
        let barrier = barrier()?;
        let capability = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        assert!(capability.validate_for_barrier(&barrier).is_ok());

        let mut forged = capability;
        forged.capability_id = CapabilityId("cap:forged:0".to_owned());
        assert!(matches!(
            forged.validate_for_barrier(&barrier),
            Err(ExecutionCapabilityError::CapabilityIdMismatch { .. })
        ));
        Ok(())
    }

    /// The same `barrier()` fixture but with the single lease's `holder_op_index`
    /// overridden, so a derivation for op 0 finds no covering lease.
    fn barrier_lease_holder_op(
        holder_op_index: Option<u32>,
    ) -> Result<ExecutionBarrier, PlanHashError> {
        let mut barrier = barrier()?;
        if let Some(lease) = barrier.leases.first_mut() {
            lease.holder_op_index = holder_op_index;
        }
        Ok(barrier)
    }

    // The canonical capability id is exactly `cap:{barrier_id}:{op}` and is
    // distinct per op, so a capability minted for one op cannot be replayed as
    // another op's (I-001 fail-closed binding).
    #[test]
    fn canonical_capability_id_is_barrier_and_op_scoped() -> Result<(), TestError> {
        let barrier = barrier()?;
        assert_eq!(
            canonical_capability_id(&barrier, 0),
            CapabilityId("cap:evt_barrier:0".to_owned())
        );
        assert_ne!(
            canonical_capability_id(&barrier, 0),
            canonical_capability_id(&barrier, 1)
        );
        Ok(())
    }

    // derive_from_barrier fail-closed paths (I-001): an op the barrier does not
    // cover, and an op no barrier lease covers, both refuse to mint a capability.
    #[test]
    fn derive_rejects_uncovered_op_and_uncovered_lease() -> Result<(), TestError> {
        // op 1 is not in barrier.op_indexes (= [0]).
        assert!(matches!(
            ExecutionCapability::derive_from_barrier(&barrier()?, 1),
            Err(ExecutionCapabilityError::OpNotCovered { op_index: 1 })
        ));
        // op 0 is covered, but the only lease is held for op 1, so nothing covers op 0.
        assert!(matches!(
            ExecutionCapability::derive_from_barrier(&barrier_lease_holder_op(Some(1))?, 0),
            Err(ExecutionCapabilityError::LeaseCoverageMissing { op_index: 0 })
        ));
        Ok(())
    }

    // validate_for_barrier rejects a capability that points at a different barrier
    // event (I-009 exact binding), before any structural check.
    #[test]
    fn validate_rejects_barrier_mismatch() -> Result<(), TestError> {
        let barrier = barrier()?;
        let mut capability = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        capability.barrier_event_id = AuditEventId("evt_other_barrier".to_owned());
        assert!(matches!(
            capability.validate_for_barrier(&barrier),
            Err(ExecutionCapabilityError::BarrierMismatch { .. })
        ));
        Ok(())
    }

    // validate_for_barrier rejects every structural binding defect with
    // BindingMismatch (I-009): wrong action, wrong plan, an op the barrier does
    // not cover, and a capability carrying no leases.
    #[test]
    fn validate_rejects_each_binding_mismatch() -> Result<(), TestError> {
        let barrier = barrier()?;
        let other_plan = PlanHash::new(
            "sha256:3333333333333333333333333333333333333333333333333333333333333333",
        )?;

        let mut wrong_action = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        wrong_action.action_id = ActionId("other".to_owned());

        let mut wrong_plan = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        wrong_plan.plan_hash = other_plan;

        let mut wrong_op = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        wrong_op.op_index = 7;

        let mut no_leases = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        no_leases.lease_ids = Vec::new();

        for capability in [wrong_action, wrong_plan, wrong_op, no_leases] {
            assert!(matches!(
                capability.validate_for_barrier(&barrier),
                Err(ExecutionCapabilityError::BindingMismatch { .. })
            ));
        }
        Ok(())
    }

    // validate_for_barrier rejects a capability naming a lease the barrier never
    // held, even when the action/plan/op binding is otherwise exact (I-009).
    #[test]
    fn validate_rejects_lease_not_in_barrier() -> Result<(), TestError> {
        let barrier = barrier()?;
        let mut capability = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        capability.lease_ids = vec![LeaseId("ghost".to_owned())];
        assert!(matches!(
            capability.validate_for_barrier(&barrier),
            Err(ExecutionCapabilityError::LeaseNotInBarrier { .. })
        ));
        Ok(())
    }

    // The attestation message is deterministic and depends on every field, so a
    // minted attestation cannot be replayed across a capability that differs in
    // any of action/op/lease/expiry.
    #[test]
    fn attestation_message_is_deterministic_and_field_sensitive() -> Result<(), TestError> {
        let barrier = barrier()?;
        let base = ExecutionCapability::derive_from_barrier(&barrier, 0)?.attestation_message();
        assert_eq!(
            base,
            ExecutionCapability::derive_from_barrier(&barrier, 0)?.attestation_message()
        );

        let mut action = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        action.action_id = ActionId("other".to_owned());

        let mut op = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        op.op_index = 9;

        let mut leases = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        leases.lease_ids = vec![LeaseId("other".to_owned())];

        let mut expires = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        expires.expires_at = Some(Timestamp(123));

        for capability in [action, op, leases, expires] {
            assert_ne!(base, capability.attestation_message());
        }
        Ok(())
    }

    // The length-prefixed encoding is injective at field boundaries: two
    // capabilities whose raw `capability_id ++ action_id` would concatenate to the
    // same bytes ("ab"+"cd" == "abc"+"d") still produce distinct messages.
    #[test]
    fn attestation_message_is_boundary_injective() -> Result<(), TestError> {
        let barrier = barrier()?;
        let mut left = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        left.capability_id = CapabilityId("ab".to_owned());
        left.action_id = ActionId("cd".to_owned());

        let mut right = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        right.capability_id = CapabilityId("abc".to_owned());
        right.action_id = ActionId("d".to_owned());

        assert_ne!(left.attestation_message(), right.attestation_message());
        Ok(())
    }

    // --- M06.6: spend-time capability admission (`spend_admits`) ---

    /// The `barrier()` fixture with the single lease's `expires_at` overridden, so a
    /// derived capability carries that lease-derived expiry (`min` over leases).
    fn barrier_lease_expiring(
        expires_at: Option<Timestamp>,
    ) -> Result<ExecutionBarrier, PlanHashError> {
        let mut barrier = barrier()?;
        if let Some(lease) = barrier.leases.first_mut() {
            lease.expires_at = expires_at;
        }
        Ok(barrier)
    }

    fn spend_req(
        barrier: &ExecutionBarrier,
        requested_op: u32,
        now: u64,
    ) -> CapabilitySpendRequest<'_> {
        CapabilitySpendRequest {
            barrier,
            requested_op,
            now: Timestamp(now),
        }
    }

    // A freshly derived capability spent against its own barrier, for its own op,
    // before any expiry, is admitted.
    #[test]
    fn spend_admits_ok_means_bound_op_exact_and_live() -> Result<(), TestError> {
        let barrier = barrier()?; // lease expires_at None -> capability never expires
        let capability = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        assert_eq!(
            capability.spend_admits(spend_req(&barrier, 0, 1_000)),
            Ok(())
        );
        Ok(())
    }

    // A capability validated against a different barrier refuses, carrying the
    // structural cause (NotBoundToBarrier wraps validate_for_barrier's verdict).
    #[test]
    fn spend_refuses_wrong_barrier() -> Result<(), TestError> {
        let mut other = barrier()?;
        other.barrier_id = AuditEventId("evt_other".to_owned());
        let barrier = barrier()?;
        let capability = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        assert!(matches!(
            capability.spend_admits(spend_req(&other, 0, 1)),
            Err(CapabilitySpendRefusal::NotBoundToBarrier {
                cause: ExecutionCapabilityError::BarrierMismatch { .. }
            })
        ));
        Ok(())
    }

    // A relabelled (forged) capability id refuses with the canonical-id cause.
    #[test]
    fn spend_refuses_forged_id() -> Result<(), TestError> {
        let barrier = barrier()?;
        let mut capability = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        capability.capability_id = CapabilityId("cap:forged:0".to_owned());
        assert!(matches!(
            capability.spend_admits(spend_req(&barrier, 0, 1)),
            Err(CapabilitySpendRefusal::NotBoundToBarrier {
                cause: ExecutionCapabilityError::CapabilityIdMismatch { .. }
            })
        ));
        Ok(())
    }

    // A capability valid for op 0 cannot be spent for op 1 (no op-substitution).
    #[test]
    fn spend_refuses_op_substitution() -> Result<(), TestError> {
        let barrier = barrier()?;
        let capability = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        assert_eq!(
            capability.spend_admits(spend_req(&barrier, 1, 1)),
            Err(CapabilitySpendRefusal::OpMismatch {
                capability_op: 0,
                requested_op: 1,
            })
        );
        Ok(())
    }

    // Expiry uses `<=` (parity with the authz gate): `expires_at == now` refuses,
    // `now < expires_at` is live.
    #[test]
    fn spend_refuses_at_expiry_boundary() -> Result<(), TestError> {
        let barrier = barrier_lease_expiring(Some(Timestamp(5)))?;
        let capability = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        assert_eq!(capability.expires_at, Some(Timestamp(5)));
        assert_eq!(
            capability.spend_admits(spend_req(&barrier, 0, 5)),
            Err(CapabilitySpendRefusal::Expired {
                expires_at: Timestamp(5),
                now: Timestamp(5),
            })
        );
        assert_eq!(capability.spend_admits(spend_req(&barrier, 0, 4)), Ok(()));
        Ok(())
    }

    // Precedence: structural binding dominates op-mismatch dominates expiry.
    #[test]
    fn spend_precedence_structural_beats_op_beats_expiry() -> Result<(), TestError> {
        let barrier = barrier_lease_expiring(Some(Timestamp(5)))?;
        // Wrong action (structural) + wrong op + expired -> NotBoundToBarrier.
        let mut wrong_action = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        wrong_action.action_id = ActionId("other".to_owned());
        assert!(matches!(
            wrong_action.spend_admits(spend_req(&barrier, 1, 100)),
            Err(CapabilitySpendRefusal::NotBoundToBarrier { .. })
        ));
        // Structurally bound but wrong op + expired -> OpMismatch (before expiry).
        let capability = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        assert_eq!(
            capability.spend_admits(spend_req(&barrier, 1, 100)),
            Err(CapabilitySpendRefusal::OpMismatch {
                capability_op: 0,
                requested_op: 1,
            })
        );
        Ok(())
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SpendCategory {
        Admit,
        NotBound,
        OpMismatch,
        Expired,
    }

    fn categorize(verdict: &Result<(), CapabilitySpendRefusal>) -> SpendCategory {
        match verdict {
            Ok(()) => SpendCategory::Admit,
            Err(CapabilitySpendRefusal::NotBoundToBarrier { .. }) => SpendCategory::NotBound,
            Err(CapabilitySpendRefusal::OpMismatch { .. }) => SpendCategory::OpMismatch,
            Err(CapabilitySpendRefusal::Expired { .. }) => SpendCategory::Expired,
        }
    }

    /// Independent oracle: a single structural conjunction (not
    /// `validate_for_barrier`'s early-return ladder), then op-exactness, then expiry —
    /// the M06.6 three-way precedence, blind to which structural sub-check failed.
    fn spend_oracle(
        capability: &ExecutionCapability,
        req: &CapabilitySpendRequest<'_>,
    ) -> SpendCategory {
        let bound = capability.barrier_event_id == req.barrier.barrier_id
            && capability.action_id == req.barrier.action_id
            && capability.plan_hash == req.barrier.plan_hash
            && req.barrier.op_indexes.contains(&capability.op_index)
            && !capability.lease_ids.is_empty()
            && capability
                .lease_ids
                .iter()
                .all(|id| req.barrier.leases.iter().any(|lease| lease.lease_id == *id))
            && capability.capability_id
                == canonical_capability_id(req.barrier, capability.op_index);
        if !bound {
            return SpendCategory::NotBound;
        }
        if capability.op_index != req.requested_op {
            return SpendCategory::OpMismatch;
        }
        if capability
            .expires_at
            .is_some_and(|expiry| expiry <= req.now)
        {
            return SpendCategory::Expired;
        }
        SpendCategory::Admit
    }

    /// Load-bearing property (M06.6): over caps {valid, none-expiry, forged-id,
    /// wrong-action} × barrier {right, other} × `requested_op` {match, sub} × now
    /// {before, at, after expiry}, `spend_admits` matches the independent oracle in
    /// every cell, with non-vacuity over all four categories.
    #[test]
    fn spend_admits_grid_matches_oracle() -> Result<(), TestError> {
        let expiring = barrier_lease_expiring(Some(Timestamp(50)))?;
        let barrier_noexp = barrier()?;
        let mut other = barrier()?;
        other.barrier_id = AuditEventId("evt_other".to_owned());

        let valid = ExecutionCapability::derive_from_barrier(&expiring, 0)?;
        let valid_noexp = ExecutionCapability::derive_from_barrier(&barrier_noexp, 0)?;
        let mut forged = ExecutionCapability::derive_from_barrier(&expiring, 0)?;
        forged.capability_id = CapabilityId("cap:forged:0".to_owned());
        let mut wrong_action = ExecutionCapability::derive_from_barrier(&expiring, 0)?;
        wrong_action.action_id = ActionId("other".to_owned());

        let caps = [valid, valid_noexp, forged, wrong_action];
        let barriers = [&expiring, &other];
        let mut seen = Vec::new();
        for capability in &caps {
            for target in barriers {
                for requested_op in [0u32, 1] {
                    for now in [49u64, 50, 51] {
                        let req = spend_req(target, requested_op, now);
                        let verdict = capability.spend_admits(req);
                        assert_eq!(categorize(&verdict), spend_oracle(capability, &req));
                        seen.push(categorize(&verdict));
                    }
                }
            }
        }
        for want in [
            SpendCategory::Admit,
            SpendCategory::NotBound,
            SpendCategory::OpMismatch,
            SpendCategory::Expired,
        ] {
            assert!(seen.contains(&want), "category never produced");
        }
        Ok(())
    }
}
