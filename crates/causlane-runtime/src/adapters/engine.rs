//! M06.3 — the engine-agnostic authz adapter contract.
//!
//! One Causlane-side request view ([`HostAuthzRequest`]) and one mapping trait
//! ([`AuthzEngineAdapter`]) that every engine sketch implements: `Cedar` (M06.2),
//! `Casbin` / `AuthZEN` / `OpenFGA` (M06.3). An adapter is **mapping-only** — it
//! maps the Causlane authz vocabulary onto an engine's native request shape and
//! makes **no** authorization decision. The deny-by-default gate (`authz_gate`,
//! ADR-0011 / I-009) stays the sole authority; no engine is baked into core or
//! into this runtime crate (no engine dependency exists).

use causlane_core::{ActionCall, PlanHash};

/// The Causlane-side authz request every adapter maps. It borrows an existing
/// `ActionCall` (redefining none of subject / action / predicate / circumstance)
/// plus the plan hash and lifecycle stage the decision is bound to — the single
/// lingua franca between the dispatcher and any engine adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostAuthzRequest<'a> {
    /// The inbound action call (subject, action, predicate, circumstance).
    pub call: &'a ActionCall,
    /// The plan hash the request is bound to (an I-009 binding coordinate).
    pub plan: &'a PlanHash,
    /// The lifecycle stage being authorized (the gate's stage token).
    pub stage: &'a str,
}

/// Maps a Causlane authz request onto one engine's native request shape.
///
/// Implementors are **total** (every [`HostAuthzRequest`] yields a request) and
/// **faithful** (each Causlane coordinate lands in exactly one engine slot). The
/// mapping is pure and **decision-neutral**: the returned value is a request to be
/// evaluated elsewhere — there is no slot to put a verdict in, so an adapter can
/// never become a second authority. Different engines model the same request
/// differently (`PARC` vs `(sub, obj, act)` vs `(user, relation, object)`), so the
/// per-engine slotting is each adapter's documented modeling choice.
pub trait AuthzEngineAdapter {
    /// The engine's native request type.
    type Request;

    /// A stable label identifying the target engine, **for diagnostics/logging
    /// only — never for dispatch** (dispatch is by static type, not this string).
    fn engine(&self) -> &'static str;

    /// Map the Causlane authz request onto this engine's request shape.
    fn map_request(&self, req: &HostAuthzRequest<'_>) -> Self::Request;
}

#[cfg(test)]
mod tests {
    use super::{AuthzEngineAdapter, HostAuthzRequest};
    use crate::adapters::authzen::AuthZenAdapter;
    use crate::adapters::casbin::CasbinAdapter;
    use crate::adapters::cedar::CedarAdapter;
    use crate::adapters::openfga::OpenFgaAdapter;
    use causlane_core::{
        ActionCall, ActionId, CorrelationId, PlanHash, PlanHashError, PredicateId,
    };

    const VALID_HASH: &str =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000";

    fn sample_call() -> ActionCall {
        ActionCall {
            action_id: ActionId("release.promote_candidate".to_owned()),
            predicate: PredicateId("release.promote".to_owned()),
            subject_ref: "release_candidate:rc_123".to_owned(),
            circumstance_ref: "requested_by:alice".to_owned(),
            correlation_id: CorrelationId("corr".to_owned()),
        }
    }

    fn assert_total_and_deterministic<A>(adapter: &A, req: &HostAuthzRequest<'_>)
    where
        A: AuthzEngineAdapter,
        A::Request: PartialEq + core::fmt::Debug,
    {
        assert!(!adapter.engine().is_empty());
        assert_eq!(adapter.map_request(req), adapter.map_request(req));
    }

    /// Contract: every adapter is total (yields a request) and deterministic (the
    /// same input maps to an equal request — the mapping is pure, with no hidden
    /// state), and advertises a non-empty engine label. (Decision-neutrality is
    /// structural — the request types carry no verdict slot — not asserted here.)
    #[test]
    fn every_adapter_is_total_and_deterministic() -> Result<(), PlanHashError> {
        let call = sample_call();
        let plan = PlanHash::new(VALID_HASH)?;
        let req = HostAuthzRequest {
            call: &call,
            plan: &plan,
            stage: "execution_barrier_logged",
        };
        assert_total_and_deterministic(&CedarAdapter, &req);
        assert_total_and_deterministic(&CasbinAdapter, &req);
        assert_total_and_deterministic(&AuthZenAdapter, &req);
        assert_total_and_deterministic(&OpenFgaAdapter, &req);
        Ok(())
    }

    /// Negative control against slot-swapping: the action verb (`action_id`) lands
    /// in each engine's verb slot — `Cedar` action / `Casbin` act / `AuthZEN`
    /// action.name / `OpenFGA` relation. The subject and predicate roles
    /// legitimately differ by engine semantics, so only the verb — whose role is
    /// consistent across all four — is pinned here.
    #[test]
    fn mappings_agree_on_the_verb_slot() -> Result<(), PlanHashError> {
        let call = sample_call();
        let plan = PlanHash::new(VALID_HASH)?;
        let req = HostAuthzRequest {
            call: &call,
            plan: &plan,
            stage: "st",
        };
        let verb = call.action_id.0.clone();
        assert_eq!(CedarAdapter.map_request(&req).action.0.entity_id, verb);
        assert_eq!(CasbinAdapter.map_request(&req).act, verb);
        assert_eq!(AuthZenAdapter.map_request(&req).action.name, verb);
        assert_eq!(OpenFgaAdapter.map_request(&req).relation, verb);
        Ok(())
    }
}
