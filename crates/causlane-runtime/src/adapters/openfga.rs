//! M06.3 — `OpenFGA` adapter sketch (mapping-only).
//!
//! Maps the Causlane authz vocabulary onto `OpenFGA` / `Zanzibar`'s
//! `(user, relation, object)` relationship-check shape. No `openfga` dependency,
//! no Check call, no decision — the deny-by-default gate stays the sole authority.
//! A real Check (with contextual tuples and the binding enforcement below) is a
//! deferred, feature-gated follow-up.
//!
//! **Modeling choice (M06.3-OFGA), grounded in `ActionCall`:**
//!   - `user` ← `circumstance_ref` — the actor/requester (e.g. `requested_by:alice`);
//!   - `relation` ← `action_id` — the permission verb ("who can do what");
//!   - `object` ← `predicate` : `subject_ref` — the resource as `Zanzibar` `type:id`,
//!     where `predicate` is the typed shape (the object *type*) and `subject_ref`
//!     is the instance the action acts **on** (the object *id*).
//!
//! This deliberately diverges from the `Cedar`/`AuthZEN` mapping (which place
//! `subject_ref` in the principal slot): in `ReBAC` the `user`/`object` asymmetry
//! is load-bearing, and `subject_ref` ("the subject the action acts on") is the
//! resource *instance*, not the actor. Real `Zanzibar` actor normalization is
//! deferred (`circumstance_ref` is carried verbatim, un-normalized).

use crate::adapters::engine::{AuthzEngineAdapter, HostAuthzRequest};

/// An `OpenFGA` object reference — the `Zanzibar` `type:id` pair: the resource
/// type and the instance id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenFgaObject {
    /// The object type (Causlane `predicate` — the typed shape acted over).
    pub object_type: String,
    /// The object instance id (Causlane `subject_ref` — the thing acted on).
    pub object_id: String,
}

/// The binding sidecar — coordinates with no native `OpenFGA` tuple slot, recorded
/// faithfully so no Causlane coordinate is silently dropped; enforcement (via
/// contextual tuples / post-check) is deferred. Typed fields, not an untyped map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenFgaBinding {
    /// The plan hash the request is bound to (an I-009 binding coordinate).
    pub plan_hash: String,
    /// The lifecycle stage being authorized.
    pub stage: String,
}

/// A complete `OpenFGA` check request: the `(user, relation, object)` tuple plus
/// the binding sidecar. Field names are the `OpenFGA` slot names so the mapping is
/// self-documenting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenFgaCheckRequest {
    /// `user` — the actor/requester (Causlane `circumstance_ref`).
    pub user: String,
    /// `relation` — the permission verb (Causlane `action_id`).
    pub relation: String,
    /// `object` — the resource as `type:id` (Causlane `predicate` : `subject_ref`).
    pub object: OpenFgaObject,
    /// The binding sidecar (plan hash + stage).
    pub binding: OpenFgaBinding,
}

/// Map a Causlane authz request onto its `OpenFGA` check request. Total and
/// faithful per the M06.3-OFGA modeling choice (see module docs): `circumstance_ref`
/// → `user`, `action_id` → `relation`, `predicate` : `subject_ref` → `object`
/// (type : id), and plan + stage → `binding`. It makes no authorization decision.
#[must_use]
pub fn to_openfga(req: &HostAuthzRequest<'_>) -> OpenFgaCheckRequest {
    OpenFgaCheckRequest {
        user: req.call.circumstance_ref.clone(),
        relation: req.call.action_id.0.clone(),
        object: OpenFgaObject {
            object_type: req.call.predicate.0.clone(),
            object_id: req.call.subject_ref.clone(),
        },
        binding: OpenFgaBinding {
            plan_hash: req.plan.as_str().to_owned(),
            stage: req.stage.to_owned(),
        },
    }
}

/// The `OpenFGA` adapter — a zero-size mapping unit.
#[derive(Clone, Copy, Debug, Default)]
pub struct OpenFgaAdapter;

impl AuthzEngineAdapter for OpenFgaAdapter {
    type Request = OpenFgaCheckRequest;

    fn engine(&self) -> &'static str {
        "openfga"
    }

    fn map_request(&self, req: &HostAuthzRequest<'_>) -> OpenFgaCheckRequest {
        to_openfga(req)
    }
}

#[cfg(test)]
mod tests {
    use super::to_openfga;
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

    // The corrected ReBAC mapping: actor->user, verb->relation, and the resource
    // as type:id (predicate is the type, subject_ref the instance) — subject_ref
    // is the object, never the user.
    #[test]
    fn maps_each_coordinate_to_its_slot() -> Result<(), PlanHashError> {
        let c = call("act.verb", "shape", "subj", "circ");
        let plan = PlanHash::new(HASH)?;
        let out = to_openfga(&HostAuthzRequest {
            call: &c,
            plan: &plan,
            stage: "stg",
        });
        assert_eq!(out.user, "circ");
        assert_eq!(out.relation, "act.verb");
        assert_eq!(out.object.object_type, "shape");
        assert_eq!(out.object.object_id, "subj");
        assert_eq!(out.binding.plan_hash, HASH);
        assert_eq!(out.binding.stage, "stg");
        Ok(())
    }
}
