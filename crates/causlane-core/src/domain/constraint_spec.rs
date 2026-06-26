//! Constraint specification, snapshot and the default arbiter (M05.3).
//!
//! Split from `constraint.rs` (which holds the claim/lease/decision primitives)
//! for the 800-line file cap. A [`ConstraintSpec`] is a typed constraint a
//! provider imposes; a [`ConstraintSnapshot`] is the epoch-versioned state a
//! decision is taken against. [`resolve_constraints`] is the first function that
//! actually PRODUCES a [`ConstraintDecision`] (`Allow` / `Wait` / `Deny` /
//! `AllowWithRestrictions`) from a snapshot + a plan's claims, reusing the S03
//! conflict/claim primitives and adding token-budget arbitration. The kernel
//! `ConstraintProvider` contract (in `contract.rs`) delegates here.

use super::{
    claim_modes_conflict, ClaimMode, ConstraintBlocker, ConstraintDecision, ConstraintEpoch,
    ConstraintId, ConstraintViolation, LeaseRef, ResourceClaim, ResourceId, Scope,
};
use crate::contract::ScopeOverlap;

/// What a constraint imposes on claims whose scope it covers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstraintKind {
    /// No exclusive-write claim may be admitted on the scope (a hard deny).
    Freeze {
        /// The frozen scope.
        scope: Scope,
    },
    /// A finite token budget for a resource within a scope (capacity/quota): a
    /// claim larger than `limit` is denied; one that would push the active total
    /// over `limit` waits.
    TokenBudget {
        /// The resource the budget applies to.
        resource: ResourceId,
        /// The scope the budget applies to.
        scope: Scope,
        /// The maximum total token amount.
        limit: u64,
    },
    /// Claims on the scope are allowed but carry the noted restriction.
    Restrict {
        /// The restricted scope.
        scope: Scope,
        /// A human-readable restriction recorded on the decision.
        note: String,
    },
}

/// A typed constraint a provider imposes on the constraint plane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintSpec {
    /// The constraint's id (named in any blocker/violation it raises).
    pub constraint_id: ConstraintId,
    /// What the constraint imposes.
    pub kind: ConstraintKind,
}

/// An epoch-versioned snapshot of the constraint plane that a decision is taken
/// against. Leases granted by an `Allow` decision inherit this `epoch` (a lease
/// is only valid within its epoch — ADR-0005/0013).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintSnapshot {
    /// The snapshot's id (matches `ExecutionBarrier.constraint_snapshot_id`).
    pub snapshot_id: ConstraintId,
    /// The constraint-plane epoch this snapshot was taken at.
    pub epoch: ConstraintEpoch,
    /// The constraints active at this epoch.
    pub constraints: Vec<ConstraintSpec>,
    /// The leases active at this epoch.
    pub active_leases: Vec<LeaseRef>,
}

/// Total active token amount held for `resource` within scopes overlapping `scope`.
fn active_token_amount(
    snapshot: &ConstraintSnapshot,
    resource: &ResourceId,
    scope: &Scope,
    scopes: &impl ScopeOverlap,
) -> u64 {
    snapshot
        .active_leases
        .iter()
        .filter(|lease| {
            lease.mode == ClaimMode::Token
                && lease.resource == *resource
                && scopes.overlaps(&lease.scope, scope)
        })
        .map(|lease| lease.amount)
        .fold(0_u64, u64::saturating_add)
}

/// Total token amount already claimed earlier in the same batch for `resource`
/// within scopes overlapping `scope`. Folded into the held amount so that several
/// claims on one resource in a single resolve cannot collectively over-allocate
/// the budget (each later claim sees the earlier claims as already committed).
fn batch_token_amount(
    prior: &[ResourceClaim],
    resource: &ResourceId,
    scope: &Scope,
    scopes: &impl ScopeOverlap,
) -> u64 {
    prior
        .iter()
        .filter(|claim| {
            claim.mode == ClaimMode::Token
                && claim.resource == *resource
                && scopes.overlaps(&claim.scope, scope)
        })
        .map(|claim| claim.amount)
        .fold(0_u64, u64::saturating_add)
}

/// Resolve a plan's `claims` against `snapshot`, producing a [`ConstraintDecision`].
///
/// Precedence is fail-closed: any hard violation (`Freeze`, or a token claim that
/// can never fit) yields `Deny`; else any temporary blocker (an active exclusive
/// lease conflict, or a token claim that would exceed the budget now) yields
/// `Wait`; else any `Restrict` constraint yields `AllowWithRestrictions`; else
/// `Allow` with the claims to acquire as leases.
#[must_use]
pub fn resolve_constraints(
    snapshot: &ConstraintSnapshot,
    claims: &[ResourceClaim],
    scopes: &impl ScopeOverlap,
) -> ConstraintDecision {
    let mut violations = Vec::new();
    let mut blockers = Vec::new();
    let mut restrictions = Vec::new();

    for (index, claim) in claims.iter().enumerate() {
        for spec in &snapshot.constraints {
            match &spec.kind {
                ConstraintKind::Freeze { scope } => {
                    if claim.mode == ClaimMode::ExclusiveWrite
                        && scopes.overlaps(scope, &claim.scope)
                    {
                        violations.push(ConstraintViolation {
                            constraint_id: spec.constraint_id.clone(),
                            reason: format!("scope {} is frozen", claim.scope.0),
                        });
                    }
                }
                ConstraintKind::TokenBudget {
                    resource,
                    scope,
                    limit,
                } => {
                    if claim.mode == ClaimMode::Token
                        && *resource == claim.resource
                        && scopes.overlaps(scope, &claim.scope)
                    {
                        if claim.amount > *limit {
                            violations.push(ConstraintViolation {
                                constraint_id: spec.constraint_id.clone(),
                                reason: format!(
                                    "claim {} exceeds token budget {}",
                                    claim.amount, limit
                                ),
                            });
                        } else {
                            let held = active_token_amount(
                                snapshot,
                                &claim.resource,
                                &claim.scope,
                                scopes,
                            )
                            .saturating_add(batch_token_amount(
                                claims.get(..index).unwrap_or(&[]),
                                &claim.resource,
                                &claim.scope,
                                scopes,
                            ));
                            if held.saturating_add(claim.amount) > *limit {
                                blockers.push(ConstraintBlocker {
                                    constraint_id: spec.constraint_id.clone(),
                                    reason: format!(
                                        "token budget {limit} would be exceeded (held {held}, claim {})",
                                        claim.amount
                                    ),
                                });
                            }
                        }
                    }
                }
                ConstraintKind::Restrict { scope, note } => {
                    if scopes.overlaps(scope, &claim.scope) {
                        restrictions.push(note.clone());
                    }
                }
            }
        }

        if claim.mode == ClaimMode::ExclusiveWrite {
            for lease in &snapshot.active_leases {
                if claim_modes_conflict(
                    lease.mode,
                    claim.mode,
                    lease.resource == claim.resource,
                    scopes.overlaps(&lease.scope, &claim.scope),
                    false,
                ) {
                    blockers.push(ConstraintBlocker {
                        constraint_id: ConstraintId(format!("lease:{}", lease.lease_id.0)),
                        reason: format!("active lease conflicts on scope {}", claim.scope.0),
                    });
                }
            }
        }
    }

    if !violations.is_empty() {
        ConstraintDecision::Deny { violations }
    } else if !blockers.is_empty() {
        ConstraintDecision::Wait { blockers }
    } else if !restrictions.is_empty() {
        ConstraintDecision::AllowWithRestrictions { restrictions }
    } else {
        ConstraintDecision::Allow {
            required_leases: claims.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_constraints, ConstraintKind, ConstraintSnapshot, ConstraintSpec};
    use crate::domain::{
        ClaimMode, ConstraintDecision, ConstraintEpoch, ConstraintId, LeaseId, LeaseRef, PlanHash,
        ResourceClaim, ResourceId, Scope,
    };
    use crate::{ActionId, AuditEventId, KernelContracts, PlanHashError};

    type TestResult = Result<(), PlanHashError>;

    fn plan() -> Result<PlanHash, PlanHashError> {
        PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    }

    fn claim(resource: &str, scope: &str, mode: ClaimMode, amount: u64) -> ResourceClaim {
        ResourceClaim {
            resource: ResourceId(resource.to_owned()),
            scope: Scope(scope.to_owned()),
            mode,
            amount,
        }
    }

    fn lease(
        id: &str,
        resource: &str,
        scope: &str,
        mode: ClaimMode,
        amount: u64,
        plan: &PlanHash,
    ) -> LeaseRef {
        LeaseRef {
            lease_id: LeaseId(id.to_owned()),
            resource: ResourceId(resource.to_owned()),
            scope: Scope(scope.to_owned()),
            mode,
            amount,
            holder_action_id: ActionId("act".to_owned()),
            holder_plan_hash: plan.clone(),
            holder_op_index: Some(0),
            epoch: ConstraintEpoch(1),
            expires_at: None,
            lease_event_id: AuditEventId(format!("evt_{id}")),
        }
    }

    fn snapshot(
        constraints: Vec<ConstraintSpec>,
        active_leases: Vec<LeaseRef>,
    ) -> ConstraintSnapshot {
        ConstraintSnapshot {
            snapshot_id: ConstraintId("snap".to_owned()),
            epoch: ConstraintEpoch(1),
            constraints,
            active_leases,
        }
    }

    fn spec(id: &str, kind: ConstraintKind) -> ConstraintSpec {
        ConstraintSpec {
            constraint_id: ConstraintId(id.to_owned()),
            kind,
        }
    }

    #[test]
    fn unconstrained_claims_are_allowed_with_their_leases() {
        let snap = snapshot(Vec::new(), Vec::new());
        let claims = vec![claim(
            "env",
            "environment:staging",
            ClaimMode::ExclusiveWrite,
            1,
        )];
        assert_eq!(
            resolve_constraints(&snap, &claims, &KernelContracts),
            ConstraintDecision::Allow {
                required_leases: claims.clone()
            }
        );
    }

    #[test]
    fn freeze_denies_an_exclusive_write() {
        let snap = snapshot(
            vec![spec(
                "freeze-staging",
                ConstraintKind::Freeze {
                    scope: Scope("environment:staging".to_owned()),
                },
            )],
            Vec::new(),
        );
        let claims = vec![claim(
            "env",
            "environment:staging",
            ClaimMode::ExclusiveWrite,
            1,
        )];
        assert!(matches!(
            resolve_constraints(&snap, &claims, &KernelContracts),
            ConstraintDecision::Deny { .. }
        ));
    }

    #[test]
    fn token_budget_denies_oversized_and_waits_when_full() -> TestResult {
        let plan = plan()?;
        let budget = spec(
            "tokens",
            ConstraintKind::TokenBudget {
                resource: ResourceId("slots".to_owned()),
                scope: Scope("pool:a".to_owned()),
                limit: 3,
            },
        );

        // A single claim larger than the budget can never fit -> Deny.
        let over = vec![claim("slots", "pool:a", ClaimMode::Token, 4)];
        assert!(matches!(
            resolve_constraints(
                &snapshot(vec![budget.clone()], Vec::new()),
                &over,
                &KernelContracts
            ),
            ConstraintDecision::Deny { .. }
        ));

        // Held 2 + claim 2 > limit 3 -> Wait.
        let held = vec![lease("l1", "slots", "pool:a", ClaimMode::Token, 2, &plan)];
        let fits_later = vec![claim("slots", "pool:a", ClaimMode::Token, 2)];
        assert!(matches!(
            resolve_constraints(
                &snapshot(vec![budget.clone()], held),
                &fits_later,
                &KernelContracts
            ),
            ConstraintDecision::Wait { .. }
        ));

        // Held 1 + claim 2 == limit 3 -> Allow.
        let room = vec![lease("l1", "slots", "pool:a", ClaimMode::Token, 1, &plan)];
        let fits = vec![claim("slots", "pool:a", ClaimMode::Token, 2)];
        assert!(matches!(
            resolve_constraints(&snapshot(vec![budget], room), &fits, &KernelContracts),
            ConstraintDecision::Allow { .. }
        ));
        Ok(())
    }

    #[test]
    fn same_resource_token_claims_in_a_batch_cannot_over_allocate() {
        let budget = spec(
            "tokens",
            ConstraintKind::TokenBudget {
                resource: ResourceId("slots".to_owned()),
                scope: Scope("pool:a".to_owned()),
                limit: 10,
            },
        );

        // Two claims that each fit alone but together exceed the budget must Wait,
        // not Allow (regression: the batch was previously admitted whole).
        let over = vec![
            claim("slots", "pool:a", ClaimMode::Token, 6),
            claim("slots", "pool:a", ClaimMode::Token, 6),
        ];
        assert!(matches!(
            resolve_constraints(
                &snapshot(vec![budget.clone()], Vec::new()),
                &over,
                &KernelContracts
            ),
            ConstraintDecision::Wait { .. }
        ));

        // Two claims summing exactly to the budget still fit -> Allow.
        let exact = vec![
            claim("slots", "pool:a", ClaimMode::Token, 5),
            claim("slots", "pool:a", ClaimMode::Token, 5),
        ];
        assert!(matches!(
            resolve_constraints(
                &snapshot(vec![budget.clone()], Vec::new()),
                &exact,
                &KernelContracts
            ),
            ConstraintDecision::Allow { .. }
        ));

        // Claims on different resources are budgeted independently -> Allow.
        let ports = spec(
            "ports",
            ConstraintKind::TokenBudget {
                resource: ResourceId("ports".to_owned()),
                scope: Scope("pool:a".to_owned()),
                limit: 10,
            },
        );
        let split = vec![
            claim("slots", "pool:a", ClaimMode::Token, 6),
            claim("ports", "pool:a", ClaimMode::Token, 6),
        ];
        assert!(matches!(
            resolve_constraints(
                &snapshot(vec![budget, ports], Vec::new()),
                &split,
                &KernelContracts
            ),
            ConstraintDecision::Allow { .. }
        ));
    }

    #[test]
    fn active_exclusive_lease_makes_a_conflicting_claim_wait() -> TestResult {
        let plan = plan()?;
        let snap = snapshot(
            Vec::new(),
            vec![lease(
                "l1",
                "env",
                "environment:staging",
                ClaimMode::ExclusiveWrite,
                1,
                &plan,
            )],
        );
        let claims = vec![claim(
            "env",
            "environment:staging",
            ClaimMode::ExclusiveWrite,
            1,
        )];
        assert!(matches!(
            resolve_constraints(&snap, &claims, &KernelContracts),
            ConstraintDecision::Wait { .. }
        ));
        Ok(())
    }

    #[test]
    fn restrict_allows_with_the_recorded_restriction() {
        let snap = snapshot(
            vec![spec(
                "rate",
                ConstraintKind::Restrict {
                    scope: Scope("environment:staging".to_owned()),
                    note: "rate-limited".to_owned(),
                },
            )],
            Vec::new(),
        );
        let claims = vec![claim(
            "env",
            "environment:staging",
            ClaimMode::SharedRead,
            1,
        )];
        assert_eq!(
            resolve_constraints(&snap, &claims, &KernelContracts),
            ConstraintDecision::AllowWithRestrictions {
                restrictions: vec!["rate-limited".to_owned()]
            }
        );
    }

    /// Fail-closed precedence: Deny beats Wait, Wait beats Restrict, Restrict
    /// beats Allow — across the four decision kinds.
    #[test]
    fn decision_precedence_is_fail_closed() -> TestResult {
        let plan = plan()?;
        // Deny (freeze) outranks Wait (token-full): two claims, one each.
        let deny_over_wait = snapshot(
            vec![
                spec(
                    "freeze",
                    ConstraintKind::Freeze {
                        scope: Scope("a".to_owned()),
                    },
                ),
                spec(
                    "budget",
                    ConstraintKind::TokenBudget {
                        resource: ResourceId("slots".to_owned()),
                        scope: Scope("b".to_owned()),
                        limit: 1,
                    },
                ),
            ],
            Vec::new(),
        );
        let mixed = vec![
            claim("env", "a", ClaimMode::ExclusiveWrite, 1),
            claim("slots", "b", ClaimMode::Token, 5),
        ];
        assert!(matches!(
            resolve_constraints(&deny_over_wait, &mixed, &KernelContracts),
            ConstraintDecision::Deny { .. }
        ));

        // Wait (active conflict) outranks Restrict (same scope restriction).
        let wait_over_restrict = snapshot(
            vec![spec(
                "rate",
                ConstraintKind::Restrict {
                    scope: Scope("a".to_owned()),
                    note: "noted".to_owned(),
                },
            )],
            vec![lease("l1", "env", "a", ClaimMode::ExclusiveWrite, 1, &plan)],
        );
        let conflicting = vec![claim("env", "a", ClaimMode::ExclusiveWrite, 1)];
        assert!(matches!(
            resolve_constraints(&wait_over_restrict, &conflicting, &KernelContracts),
            ConstraintDecision::Wait { .. }
        ));
        Ok(())
    }

    /// Exhaustiveness guard: a new `ConstraintKind` breaks this until handled.
    #[test]
    fn constraint_kind_is_exhaustive() {
        fn covered(kind: &ConstraintKind) -> bool {
            match kind {
                ConstraintKind::Freeze { .. }
                | ConstraintKind::TokenBudget { .. }
                | ConstraintKind::Restrict { .. } => true,
            }
        }
        assert!(covered(&ConstraintKind::Freeze {
            scope: Scope("s".to_owned())
        }));
    }
}
