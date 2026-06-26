//! Generated-identifier collision detection (review finding H3/M4).
//!
//! `crate::alloy::alloy_ident` (shared by the Alloy and P generators) maps every
//! non-alphanumeric character to `_`, so it is not injective: distinct domain
//! names such as `evt-1` and `evt_1` produce the same identifier. A collision
//! would silently merge two distinct entities into one sig in the generated
//! model, making any proof over that model unsound. This module fails closed when
//! that would happen, leaving all non-colliding output byte-identical.

use crate::alloy::alloy_ident;
use crate::error::CodegenError;
use crate::ir::FormalIr;

/// Fail closed when two *distinct* raw names of one kind sanitize to the same
/// target identifier. Repeated references to the same raw name are fine.
pub(crate) fn check_injective<'a>(
    kind: &str,
    raws: impl Iterator<Item = &'a str>,
) -> Result<(), CodegenError> {
    // (identifier, raw) pairs, sorted so equal identifiers are adjacent. A pair of
    // neighbours sharing an identifier but differing in raw name is a collision.
    let mut pairs: Vec<(String, &'a str)> = raws.map(|raw| (alloy_ident(raw), raw)).collect();
    pairs.sort();
    for window in pairs.windows(2) {
        if let [(id_a, raw_a), (id_b, raw_b)] = window {
            if id_a == id_b && raw_a != raw_b {
                return Err(CodegenError::Collision(format!(
                    "two {kind} names {raw_a:?} and {raw_b:?} both map to identifier {id_a:?}"
                )));
            }
        }
    }
    Ok(())
}

/// Check every generated-identifier namespace for collisions. Each namespace is a
/// distinct sig prefix (`Pred_` / `Event_` / `Action_` / `Resource_` /
/// `LeaseScope_` / `FactScope_` / `Plan_`), validated independently — lease scopes
/// and fact scopes in particular occupy different prefixes and must not be merged.
pub(crate) fn check_identifier_injectivity(ir: &FormalIr) -> Result<(), CodegenError> {
    check_injective(
        "predicate",
        ir.predicates
            .iter()
            .map(|predicate| predicate.predicate.as_str()),
    )?;

    let mut events: Vec<&str> = Vec::new();
    let mut actions: Vec<&str> = Vec::new();
    let mut resources: Vec<&str> = Vec::new();
    let mut lease_scopes: Vec<&str> = Vec::new();
    let mut fact_scopes: Vec<&str> = Vec::new();
    let mut plans: Vec<&str> = Vec::new();

    for event in &ir.scenario_events {
        events.push(event.event_id.as_str());
        if let Some(action) = &event.action_id {
            actions.push(action.as_str());
        }
        if let Some(plan) = &event.plan_hash {
            plans.push(plan.as_str());
        }
        if let Some(scope) = &event.scope {
            fact_scopes.push(scope.as_str());
        }
        for anchor in &event.anchors {
            events.push(anchor.event_id.as_str());
            if let Some(scope) = &anchor.scope {
                fact_scopes.push(scope.as_str());
            }
        }
        for lease in &event.leases {
            resources.push(lease.resource.as_str());
            lease_scopes.push(lease.scope.as_str());
        }
        if let Some(barrier) = &event.barrier {
            events.push(barrier.barrier_event_id.as_str());
            actions.push(barrier.action_id.as_str());
            plans.push(barrier.plan_hash.as_str());
            for decision in &barrier.authz_decision_event_ids {
                events.push(decision.as_str());
            }
            for witness in &barrier.witnesses {
                events.push(witness.witness_event_id.as_str());
                if let Some(action) = &witness.binds_to_action_id {
                    actions.push(action.as_str());
                }
                if let Some(plan) = &witness.binds_to_plan_hash {
                    plans.push(plan.as_str());
                }
                if let Some(scope) = &witness.scope {
                    fact_scopes.push(scope.as_str());
                }
            }
        }
        if let Some(capability) = &event.capability {
            events.push(capability.barrier_event_id.as_str());
        }
        if let Some(authz) = &event.authz_decision {
            events.push(authz.decision_event_id.as_str());
            actions.push(authz.action_id.as_str());
            plans.push(authz.plan_hash.as_str());
        }
    }

    check_injective("event", events.into_iter())?;
    check_injective("action", actions.into_iter())?;
    check_injective("resource", resources.into_iter())?;
    check_injective("lease scope", lease_scopes.into_iter())?;
    check_injective("fact scope", fact_scopes.into_iter())?;
    check_injective("plan hash", plans.into_iter())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_identifier_injectivity, check_injective};
    use crate::error::CodegenError;
    use crate::ir::{FormalEvent, FormalIr, FormalLeaseFact};

    fn empty_ir() -> FormalIr {
        FormalIr {
            schema_version: 2,
            formal_ir_hash: String::new(),
            generator_version: String::new(),
            canonical_serialization_version: 1,
            source_bundle_hash: String::new(),
            bundle_schema_version: 1,
            bundle_id: String::new(),
            bundle_version: String::new(),
            scenario_hash: None,
            expected_result: None,
            expected_error_code: None,
            predicates: Vec::new(),
            merge_protocols: Vec::new(),
            scenario_events: Vec::new(),
            invariants: Vec::new(),
        }
    }

    fn base_event(event_id: &str) -> FormalEvent {
        FormalEvent {
            event_id: event_id.to_owned(),
            kind: "noop".to_owned(),
            action_id: None,
            plan_hash: None,
            op_index: None,
            fact_kind: None,
            scope: None,
            anchors: Vec::new(),
            leases: Vec::new(),
            barrier: None,
            capability: None,
            authz_decision: None,
        }
    }

    fn lease(resource: &str, scope: &str) -> FormalLeaseFact {
        FormalLeaseFact {
            lease_id: format!("lease_{resource}_{scope}"),
            resource: resource.to_owned(),
            scope: scope.to_owned(),
            mode: "token".to_owned(),
            epoch: 1,
        }
    }

    #[test]
    fn collision_is_detected_for_every_named_kind() {
        // The matrix requires collision coverage for these identifier kinds.
        for kind in ["predicate", "event", "action", "scope", "resource"] {
            let pair = ["a.b", "a_b"];
            assert!(
                matches!(
                    check_injective(kind, pair.into_iter()),
                    Err(CodegenError::Collision(_))
                ),
                "{kind} collision not detected"
            );
        }
    }

    #[test]
    fn repeated_same_name_and_distinct_identifiers_are_allowed() {
        assert!(check_injective("event", ["x", "x", "x"].into_iter()).is_ok());
        assert!(check_injective("event", ["alpha", "beta"].into_iter()).is_ok());
    }

    #[test]
    fn event_id_collision_is_detected_through_the_ir() {
        let mut ir = empty_ir();
        ir.scenario_events = vec![base_event("evt-1"), base_event("evt_1")];
        assert!(matches!(
            check_identifier_injectivity(&ir),
            Err(CodegenError::Collision(_))
        ));
    }

    #[test]
    fn action_id_collision_is_detected_through_the_ir() {
        let mut first = base_event("e1");
        first.action_id = Some("act-x".to_owned());
        let mut second = base_event("e2");
        second.action_id = Some("act_x".to_owned());
        let mut ir = empty_ir();
        ir.scenario_events = vec![first, second];
        assert!(matches!(
            check_identifier_injectivity(&ir),
            Err(CodegenError::Collision(_))
        ));
    }

    #[test]
    fn resource_and_lease_scope_collisions_are_detected_through_the_ir() {
        let mut resource_event = base_event("e1");
        resource_event.leases = vec![lease("r.a", "scope_one"), lease("r_a", "scope_two")];
        let mut ir = empty_ir();
        ir.scenario_events = vec![resource_event];
        assert!(matches!(
            check_identifier_injectivity(&ir),
            Err(CodegenError::Collision(_))
        ));

        let mut scope_event = base_event("e1");
        scope_event.leases = vec![lease("res_one", "env:s"), lease("res_two", "env_s")];
        let mut scope_ir = empty_ir();
        scope_ir.scenario_events = vec![scope_event];
        assert!(matches!(
            check_identifier_injectivity(&scope_ir),
            Err(CodegenError::Collision(_))
        ));
    }

    #[test]
    fn lease_scope_and_fact_scope_are_separate_namespaces() {
        // A lease scope and a fact scope that sanitize to the same string must NOT
        // collide: they are emitted under different sig prefixes.
        let mut event = base_event("e1");
        event.scope = Some("env:s".to_owned()); // fact scope -> FactScope_env_s
        event.leases = vec![lease("res", "env_s")]; // lease scope -> LeaseScope_env_s
        let mut ir = empty_ir();
        ir.scenario_events = vec![event];
        assert!(check_identifier_injectivity(&ir).is_ok());
    }
}
