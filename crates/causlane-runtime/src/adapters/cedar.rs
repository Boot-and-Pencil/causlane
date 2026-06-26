//! M06.2 — `Cedar` adapter prototype (mapping-only).
//!
//! A faithful schema mapping from Causlane's authz vocabulary — an `ActionCall`
//! (`subject_ref`, `action_id`, `predicate`, `circumstance_ref`) plus the bound
//! plan hash and lifecycle stage — onto `Cedar`'s Principal / Action / Resource /
//! Context (PARC) shape. It proves the schemas line up; it embeds **no**
//! `cedar-policy` engine and makes **no** authorization decision of its own — the
//! deny-by-default gate (`authz_gate`, ADR-0011 / I-009) stays the sole authority.
//!
//! Out of scope: real `cedar-policy` evaluation (a feature-gated follow-up);
//! `Casbin` / `AuthZEN` / `OpenFGA` (M06.3); approvals / step-up / capability /
//! redaction (M06.4–M06.7).

use crate::adapters::engine::{AuthzEngineAdapter, HostAuthzRequest};

/// The `Cedar` entity type of a mapped entity reference — a closed set of schema
/// labels (not a free string), so the mapping is never stringly-dispatched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CedarEntityType {
    /// The principal's entity type (Causlane subject).
    Subject,
    /// The action's entity type (Causlane action verb).
    Action,
    /// The resource's entity type (Causlane predicate — the typed shape acted over).
    Predicate,
}

impl CedarEntityType {
    /// The `Cedar` entity-type token (the left of `Type::"id"`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            CedarEntityType::Subject => "Subject",
            CedarEntityType::Action => "Action",
            CedarEntityType::Predicate => "Predicate",
        }
    }
}

/// A `Cedar` entity reference — a typed `Type::"id"` pair (`Cedar`'s UID shape).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CedarEntityRef {
    /// The entity type (the left of `Type::"id"`).
    pub entity_type: CedarEntityType,
    /// The opaque entity id (the quoted right of `Type::"id"`).
    pub entity_id: String,
}

/// `Cedar` Principal — who/what the action acts on (Causlane `subject_ref`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CedarPrincipal(pub CedarEntityRef);

/// `Cedar` Action — the verb (Causlane `action_id`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CedarAction(pub CedarEntityRef);

/// `Cedar` Resource — the typed shape acted over (Causlane `predicate`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CedarResource(pub CedarEntityRef);

/// `Cedar` Context — the structured attributes needed to interpret the request:
/// the circumstance reference, the bound plan hash, and the lifecycle stage.
/// Typed fields, not an untyped attribute map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CedarContext {
    /// Causlane `circumstance_ref` — the circumstance the action is evaluated in.
    pub circumstance_ref: String,
    /// The plan hash the request is bound to (an I-009 binding coordinate).
    pub plan_hash: String,
    /// The lifecycle stage being authorized (the gate's stage token).
    pub stage: String,
}

/// A complete `Cedar` request tuple (PARC) produced by the mapping.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CedarRequest {
    /// The principal (from the subject).
    pub principal: CedarPrincipal,
    /// The action (from the action verb).
    pub action: CedarAction,
    /// The resource (from the predicate).
    pub resource: CedarResource,
    /// The context (circumstance + plan hash + stage).
    pub context: CedarContext,
}

/// Backwards-compatible alias for the shared [`HostAuthzRequest`] (the M06.2 name,
/// kept so existing call sites and tests are unchanged). M06.3 lifts the request
/// view into the engine-agnostic [`crate::adapters::engine`] contract.
pub type CedarAuthzRequest<'a> = HostAuthzRequest<'a>;

/// Map a Causlane authz request onto its `Cedar` PARC tuple.
///
/// Total (every request yields a [`CedarRequest`]) and faithful (each Causlane
/// coordinate lands in exactly one PARC slot, preserving the
/// subject/verb/shape/context separation): `subject_ref` → Principal, `action_id`
/// → Action, `predicate` → Resource, and `circumstance_ref` + plan hash + stage →
/// Context. It makes no authorization decision.
#[must_use]
pub fn to_cedar(req: &CedarAuthzRequest<'_>) -> CedarRequest {
    CedarRequest {
        principal: CedarPrincipal(CedarEntityRef {
            entity_type: CedarEntityType::Subject,
            entity_id: req.call.subject_ref.clone(),
        }),
        action: CedarAction(CedarEntityRef {
            entity_type: CedarEntityType::Action,
            entity_id: req.call.action_id.0.clone(),
        }),
        resource: CedarResource(CedarEntityRef {
            entity_type: CedarEntityType::Predicate,
            entity_id: req.call.predicate.0.clone(),
        }),
        context: CedarContext {
            circumstance_ref: req.call.circumstance_ref.clone(),
            plan_hash: req.plan.as_str().to_owned(),
            stage: req.stage.to_owned(),
        },
    }
}

/// The `Cedar` adapter — a zero-size mapping unit implementing the shared
/// [`AuthzEngineAdapter`] contract (M06.3).
#[derive(Clone, Copy, Debug, Default)]
pub struct CedarAdapter;

impl AuthzEngineAdapter for CedarAdapter {
    type Request = CedarRequest;

    fn engine(&self) -> &'static str {
        "cedar"
    }

    fn map_request(&self, req: &HostAuthzRequest<'_>) -> CedarRequest {
        to_cedar(req)
    }
}

#[cfg(test)]
mod tests {
    use super::{to_cedar, CedarAuthzRequest, CedarEntityType};
    use causlane_core::{
        ActionCall, ActionId, CorrelationId, PlanHash, PlanHashError, PredicateId,
    };

    const VALID_HASH: &str =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    const OTHER_HASH: &str =
        "sha256:1111111111111111111111111111111111111111111111111111111111111111";

    fn call(action: &str, predicate: &str, subject: &str, circumstance: &str) -> ActionCall {
        ActionCall {
            action_id: ActionId(action.to_owned()),
            predicate: PredicateId(predicate.to_owned()),
            subject_ref: subject.to_owned(),
            circumstance_ref: circumstance.to_owned(),
            correlation_id: CorrelationId("corr".to_owned()),
        }
    }

    #[test]
    fn maps_each_coordinate_to_its_parc_slot() -> Result<(), PlanHashError> {
        let c = call(
            "release.promote_candidate",
            "release.promote",
            "release_candidate:rc_123",
            "requested_by:alice",
        );
        let plan = PlanHash::new(VALID_HASH)?;
        let out = to_cedar(&CedarAuthzRequest {
            call: &c,
            plan: &plan,
            stage: "execution_barrier_logged",
        });
        assert_eq!(out.principal.0.entity_type, CedarEntityType::Subject);
        assert_eq!(out.principal.0.entity_id, "release_candidate:rc_123");
        assert_eq!(out.action.0.entity_type, CedarEntityType::Action);
        assert_eq!(out.action.0.entity_id, "release.promote_candidate");
        assert_eq!(out.resource.0.entity_type, CedarEntityType::Predicate);
        assert_eq!(out.resource.0.entity_id, "release.promote");
        assert_eq!(out.context.circumstance_ref, "requested_by:alice");
        assert_eq!(out.context.plan_hash, VALID_HASH);
        assert_eq!(out.context.stage, "execution_barrier_logged");
        Ok(())
    }

    /// Load-bearing: flipping any single coordinate changes the mapped request, so
    /// no coordinate is dropped or ignored (totality + observability). Non-vacuity:
    /// every one of the six coordinates is exercised.
    #[test]
    fn every_coordinate_is_faithfully_carried() -> Result<(), PlanHashError> {
        let plan1 = PlanHash::new(VALID_HASH)?;
        let plan2 = PlanHash::new(OTHER_HASH)?;
        let base_call = call("a", "p", "s", "c");
        let base = to_cedar(&CedarAuthzRequest {
            call: &base_call,
            plan: &plan1,
            stage: "st",
        });

        let variants = [
            to_cedar(&CedarAuthzRequest {
                call: &call("a2", "p", "s", "c"),
                plan: &plan1,
                stage: "st",
            }),
            to_cedar(&CedarAuthzRequest {
                call: &call("a", "p2", "s", "c"),
                plan: &plan1,
                stage: "st",
            }),
            to_cedar(&CedarAuthzRequest {
                call: &call("a", "p", "s2", "c"),
                plan: &plan1,
                stage: "st",
            }),
            to_cedar(&CedarAuthzRequest {
                call: &call("a", "p", "s", "c2"),
                plan: &plan1,
                stage: "st",
            }),
            to_cedar(&CedarAuthzRequest {
                call: &base_call,
                plan: &plan2,
                stage: "st",
            }),
            to_cedar(&CedarAuthzRequest {
                call: &base_call,
                plan: &plan1,
                stage: "st2",
            }),
        ];
        for variant in &variants {
            assert_ne!(
                &base, variant,
                "a flipped coordinate did not change the Cedar request"
            );
        }
        Ok(())
    }

    /// Faithfulness / no cross-slot aliasing: the same literal in two different
    /// Causlane inputs lands in two distinct, typed PARC slots — never merged.
    #[test]
    fn same_value_in_different_inputs_lands_in_distinct_slots() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(VALID_HASH)?;
        // subject and predicate share the literal "x".
        let c = call("a", "x", "x", "circ");
        let out = to_cedar(&CedarAuthzRequest {
            call: &c,
            plan: &plan,
            stage: "st",
        });
        assert_eq!(out.principal.0.entity_id, "x");
        assert_eq!(out.resource.0.entity_id, "x");
        assert_ne!(out.principal.0.entity_type, out.resource.0.entity_type);
        assert_eq!(out.principal.0.entity_type, CedarEntityType::Subject);
        assert_eq!(out.resource.0.entity_type, CedarEntityType::Predicate);
        Ok(())
    }
}
