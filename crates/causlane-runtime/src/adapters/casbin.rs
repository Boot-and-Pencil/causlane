//! M06.3 — `Casbin` adapter sketch (mapping-only).
//!
//! Maps the Causlane authz vocabulary onto `Casbin`'s classic `(sub, obj, act)`
//! enforce request plus the binding context attributes. No `casbin` dependency,
//! no `enforce` call, no decision — the deny-by-default gate stays the sole
//! authority. Real `Enforcer::enforce` (and any `dom` tenancy partitioning) is a
//! deferred, feature-gated follow-up.

use crate::adapters::engine::{AuthzEngineAdapter, HostAuthzRequest};

/// The matcher-visible binding context carried alongside the enforce tuple —
/// circumstance, the bound plan hash, and the lifecycle stage. Typed fields, not
/// an untyped attribute map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CasbinContext {
    /// Causlane `circumstance_ref`.
    pub circumstance_ref: String,
    /// The plan hash the request is bound to (an I-009 binding coordinate).
    pub plan_hash: String,
    /// The lifecycle stage being authorized.
    pub stage: String,
}

/// A `Casbin` enforce request: the `(sub, obj, act)` tuple plus binding context.
/// Field names are the `Casbin` slot names so the mapping is self-documenting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CasbinRequest {
    /// `sub` — the subject (Causlane `subject_ref`).
    pub sub: String,
    /// `obj` — the object, the typed shape acted over (Causlane `predicate`).
    pub obj: String,
    /// `act` — the action verb (Causlane `action_id`).
    pub act: String,
    /// The binding context (circumstance + plan hash + stage).
    pub context: CasbinContext,
}

/// Map a Causlane authz request onto its `Casbin` enforce request. Total and
/// faithful: `subject_ref` → `sub`, `predicate` → `obj`, `action_id` → `act`, and
/// circumstance + plan + stage → `context`. It makes no authorization decision.
///
/// Modeling choice (M06.3): `Casbin`'s optional `dom` (tenancy/domain partition)
/// is **not** populated — Causlane's `circumstance_ref` is request context, not a
/// tenant, so it is carried in `context`; real `dom` partitioning is deferred.
#[must_use]
pub fn to_casbin(req: &HostAuthzRequest<'_>) -> CasbinRequest {
    CasbinRequest {
        sub: req.call.subject_ref.clone(),
        obj: req.call.predicate.0.clone(),
        act: req.call.action_id.0.clone(),
        context: CasbinContext {
            circumstance_ref: req.call.circumstance_ref.clone(),
            plan_hash: req.plan.as_str().to_owned(),
            stage: req.stage.to_owned(),
        },
    }
}

/// The `Casbin` adapter — a zero-size mapping unit.
#[derive(Clone, Copy, Debug, Default)]
pub struct CasbinAdapter;

impl AuthzEngineAdapter for CasbinAdapter {
    type Request = CasbinRequest;

    fn engine(&self) -> &'static str {
        "casbin"
    }

    fn map_request(&self, req: &HostAuthzRequest<'_>) -> CasbinRequest {
        to_casbin(req)
    }
}

#[cfg(test)]
mod tests {
    use super::to_casbin;
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

    // Faithful placement: distinct values land in their own slots (which also
    // rules out cross-slot aliasing — an aliased coordinate would show in the
    // wrong slot).
    #[test]
    fn maps_each_coordinate_to_its_slot() -> Result<(), PlanHashError> {
        let c = call("act.verb", "shape", "subj", "circ");
        let plan = PlanHash::new(HASH)?;
        let out = to_casbin(&HostAuthzRequest {
            call: &c,
            plan: &plan,
            stage: "stg",
        });
        assert_eq!(out.sub, "subj");
        assert_eq!(out.obj, "shape");
        assert_eq!(out.act, "act.verb");
        assert_eq!(out.context.circumstance_ref, "circ");
        assert_eq!(out.context.plan_hash, HASH);
        assert_eq!(out.context.stage, "stg");
        Ok(())
    }
}
