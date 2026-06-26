use super::{
    collect_fact_grounding_facts, push_fact_grounding_assertion, push_fact_grounding_fact_block,
    push_fact_grounding_signatures,
};
use crate::{
    AlloyEventKind, AlloyScenarioEvent, AlloyScenarioFacts, FormalBarrierPayload,
    FormalWitnessPayload,
};

#[test]
fn fact_grounding_facts_bind_scope() {
    let plan_hash = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let facts = collect_fact_grounding_facts(&AlloyScenarioFacts {
        scenario_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_owned(),
        action_id: "act".to_owned(),
        plan_hash: plan_hash.to_owned(),
        expected_result: "fail".to_owned(),
        expected_error_code: Some("WitnessSelectorMismatch".to_owned()),
        formal_obligations: vec!["I-009".to_owned()],
        predicate_id: String::new(),
        subject: Vec::new(),
        circumstance: Vec::new(),
        events: vec![
            AlloyScenarioEvent {
                event_id: "evt_readiness_ok".to_owned(),
                kind: AlloyEventKind::GateApproved,
                action_id: Some("act".to_owned()),
                plan_hash: Some(plan_hash.to_owned()),
                op_index: None,
                fact_kind: Some("readiness_ok".to_owned()),
                scope: Some("release_candidate:rc_123".to_owned()),
                anchors: Vec::new(),
                anchor_facts: Vec::new(),
                leases: Vec::new(),
                barrier: None,
                capability: None,
                authz_decision: None,
            },
            AlloyScenarioEvent {
                event_id: "evt_barrier".to_owned(),
                kind: AlloyEventKind::ExecutionBarrierLogged,
                action_id: Some("act".to_owned()),
                plan_hash: Some(plan_hash.to_owned()),
                op_index: None,
                fact_kind: None,
                scope: None,
                anchors: Vec::new(),
                anchor_facts: Vec::new(),
                leases: Vec::new(),
                barrier: Some(FormalBarrierPayload {
                    barrier_event_id: "evt_barrier".to_owned(),
                    action_id: "act".to_owned(),
                    plan_hash: plan_hash.to_owned(),
                    op_indexes: vec![0],
                    impact_set_hash:
                        "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                            .to_owned(),
                    witnesses: vec![FormalWitnessPayload {
                        witness_event_id: "evt_readiness_ok".to_owned(),
                        requirement_id: "readiness_before_promotion".to_owned(),
                        kind: "gate_approval".to_owned(),
                        fact_kind: Some("readiness_ok".to_owned()),
                        scope: Some("release_candidate:other".to_owned()),
                        binds_to_action_id: None,
                        binds_to_plan_hash: None,
                        binds_to_impact_set_hash: None,
                    }],
                    lease_ids: Vec::new(),
                    authz_decision_event_ids: Vec::new(),
                }),
                capability: None,
                authz_decision: None,
            },
        ],
    });

    let mut text = String::new();
    push_fact_grounding_signatures(&mut text, &facts);
    push_fact_grounding_fact_block(&mut text, &facts);
    push_fact_grounding_assertion(&mut text, &facts);

    assert!(text.contains("EA_evt_readiness_ok.eaScope = FactScope_release_candidate_rc_123"));
    assert!(text.contains("WFC_evt_readiness_ok.wfcScope = FactScope_release_candidate_other"));
    assert!(text.contains("a.eaScope = c.wfcScope"));
}
