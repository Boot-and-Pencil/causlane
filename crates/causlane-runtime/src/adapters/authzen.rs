//! M06.3 — `AuthZEN` adapter sketch (mapping-only).
//!
//! Maps the Causlane authz vocabulary onto the `OpenID` `AuthZEN` Authorization API
//! `{ subject, action, resource, context }` (`PARC`-like) request shape — the
//! near-1:1 mirror of the `Cedar` mapping. No PDP call, no transport, no decision
//! — the deny-by-default gate stays the sole authority. A real `AuthZEN` PDP
//! round-trip (with its JSON wire types) is a deferred, feature-gated follow-up.

use crate::adapters::engine::{AuthzEngineAdapter, HostAuthzRequest};

/// The `AuthZEN` entity kind — a closed set of `type` labels (not a free string),
/// so the mapping is never stringly-dispatched. This closed set is a deliberate
/// sketch simplification of the full `AuthZEN` type space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthZenEntityKind {
    /// The subject's type (Causlane subject).
    Subject,
    /// The resource's type (Causlane predicate — the typed shape acted over).
    Predicate,
}

impl AuthZenEntityKind {
    /// The `AuthZEN` `type` token.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            AuthZenEntityKind::Subject => "Subject",
            AuthZenEntityKind::Predicate => "Predicate",
        }
    }
}

/// An `AuthZEN` entity — a typed `{ type, id }` pair (subject or resource).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthZenEntity {
    /// The entity type label.
    pub kind: AuthZenEntityKind,
    /// The opaque entity id.
    pub id: String,
}

/// The `AuthZEN` action — a named verb (Causlane `action_id`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthZenAction {
    /// The action name.
    pub name: String,
}

/// The `AuthZEN` context — circumstance, bound plan hash, and lifecycle stage.
/// Typed fields, not an untyped attribute map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthZenContext {
    /// Causlane `circumstance_ref`.
    pub circumstance_ref: String,
    /// The plan hash the request is bound to (an I-009 binding coordinate).
    pub plan_hash: String,
    /// The lifecycle stage being authorized.
    pub stage: String,
}

/// A complete `AuthZEN` request: subject, action, resource, context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthZenRequest {
    /// The subject (from `subject_ref`).
    pub subject: AuthZenEntity,
    /// The action (from `action_id`).
    pub action: AuthZenAction,
    /// The resource (from `predicate`).
    pub resource: AuthZenEntity,
    /// The context (circumstance + plan hash + stage).
    pub context: AuthZenContext,
}

/// Map a Causlane authz request onto its `AuthZEN` request shape. Total and
/// faithful: `subject_ref` → `subject`, `action_id` → `action`, `predicate` →
/// `resource`, and `circumstance_ref` + plan + stage → `context`. It makes no
/// authorization decision.
#[must_use]
pub fn to_authzen(req: &HostAuthzRequest<'_>) -> AuthZenRequest {
    AuthZenRequest {
        subject: AuthZenEntity {
            kind: AuthZenEntityKind::Subject,
            id: req.call.subject_ref.clone(),
        },
        action: AuthZenAction {
            name: req.call.action_id.0.clone(),
        },
        resource: AuthZenEntity {
            kind: AuthZenEntityKind::Predicate,
            id: req.call.predicate.0.clone(),
        },
        context: AuthZenContext {
            circumstance_ref: req.call.circumstance_ref.clone(),
            plan_hash: req.plan.as_str().to_owned(),
            stage: req.stage.to_owned(),
        },
    }
}

/// The `AuthZEN` adapter — a zero-size mapping unit.
#[derive(Clone, Copy, Debug, Default)]
pub struct AuthZenAdapter;

impl AuthzEngineAdapter for AuthZenAdapter {
    type Request = AuthZenRequest;

    fn engine(&self) -> &'static str {
        "authzen"
    }

    fn map_request(&self, req: &HostAuthzRequest<'_>) -> AuthZenRequest {
        to_authzen(req)
    }
}

#[cfg(test)]
mod tests {
    use super::{to_authzen, AuthZenEntityKind};
    use crate::adapters::engine::HostAuthzRequest;
    use causlane_core::{
        ActionCall, ActionId, CorrelationId, PlanHash, PlanHashError, PredicateId,
    };

    const HASH: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

    fn call(action: &str, predicate: &str, subject: &str, circumstance: &str) -> ActionCall {
        ActionCall {
            action_id: ActionId(action.to_owned()),
            predicate: PredicateId(predicate.to_owned()),
            subject_ref: subject.to_owned(),
            circumstance_ref: circumstance.to_owned(),
            correlation_id: CorrelationId("corr".to_owned()),
        }
    }

    // Faithful placement into the PARC slots, with distinct entity kinds (a value
    // shared between subject and resource lands in distinct typed slots).
    #[test]
    fn maps_each_coordinate_to_its_slot() -> Result<(), PlanHashError> {
        let c = call("act.verb", "shape", "subj", "circ");
        let plan = PlanHash::new(HASH)?;
        let out = to_authzen(&HostAuthzRequest {
            call: &c,
            plan: &plan,
            stage: "stg",
        });
        assert_eq!(out.subject.kind, AuthZenEntityKind::Subject);
        assert_eq!(out.subject.id, "subj");
        assert_eq!(out.action.name, "act.verb");
        assert_eq!(out.resource.kind, AuthZenEntityKind::Predicate);
        assert_eq!(out.resource.id, "shape");
        assert_eq!(out.context.circumstance_ref, "circ");
        assert_eq!(out.context.plan_hash, HASH);
        assert_eq!(out.context.stage, "stg");
        Ok(())
    }
}
