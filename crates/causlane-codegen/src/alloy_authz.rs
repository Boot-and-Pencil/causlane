//! Alloy authz-decision facts + assertion (P0-010), split from `alloy_bindings.rs`
//! for the 800-line cap. Projects the recorded authz decisions and the barrier's
//! decision refs from the payload-bound IR into Alloy, and emits the structural
//! `GeneratedAuthzDecisionValid` assertion the gate's authz negative controls
//! refute (a referenced `Deny`, or a barrier with no bound `Allow`).

use core::fmt::Write as _;

use crate::alloy::{alloy_ident, push_exact_set};
use crate::AlloyScenarioFacts;

/// Authz-decision facts (P0-010): each recorded decision's verdict + action/plan
/// binding, and the decisions a barrier references. `GeneratedAuthzDecisionValid`
/// refutes a `RuntimeExecution` barrier that does not reference a bound `Allow`
/// for its action/plan — catching a referenced `Deny` and a missing decision,
/// the same structural authz the replay oracle enforces.
pub(crate) struct AuthzFacts {
    decisions: Vec<AuthzDecisionFact>,
    refs: Vec<BarrierAuthzRef>,
}

impl AuthzFacts {
    fn is_empty(&self) -> bool {
        self.decisions.is_empty() && self.refs.is_empty()
    }
}

#[allow(clippy::struct_field_names)]
struct AuthzDecisionFact {
    ad_sig: String,
    event_sig: String,
    action_sig: String,
    plan_sig: String,
    verdict: &'static str,
}

#[allow(clippy::struct_field_names)]
struct BarrierAuthzRef {
    ref_sig: String,
    barrier_event_sig: String,
    decision_event_sig: String,
}

pub(crate) fn collect_authz_facts(scenario: &AlloyScenarioFacts) -> AuthzFacts {
    let mut decisions = Vec::new();
    let mut refs = Vec::new();
    for event in &scenario.events {
        if let Some(authz) = &event.authz_decision {
            decisions.push(AuthzDecisionFact {
                ad_sig: format!("AD_{}", alloy_ident(&event.event_id)),
                event_sig: format!("Event_{}", alloy_ident(&event.event_id)),
                action_sig: format!("Action_{}", alloy_ident(&authz.action_id)),
                plan_sig: format!("Plan_{}", alloy_ident(&authz.plan_hash)),
                verdict: if authz.decision == "allow" {
                    "AllowVerdict"
                } else {
                    "DenyVerdict"
                },
            });
        }
        if let Some(barrier) = &event.barrier {
            for (index, ref_id) in barrier.authz_decision_event_ids.iter().enumerate() {
                refs.push(BarrierAuthzRef {
                    ref_sig: format!("BAR_{}_{index}", alloy_ident(&event.event_id)),
                    barrier_event_sig: format!("Event_{}", alloy_ident(&event.event_id)),
                    decision_event_sig: format!("Event_{}", alloy_ident(ref_id)),
                });
            }
        }
    }
    AuthzFacts { decisions, refs }
}

pub(crate) fn push_authz_signatures(text: &mut String, facts: &AuthzFacts) {
    if facts.is_empty() {
        return;
    }
    text.push_str("\n// Authz decision facts (P0-010 — payload-bound).\n");
    text.push_str("abstract sig AuthzVerdict {}\n");
    text.push_str("one sig AllowVerdict extends AuthzVerdict {}\n");
    text.push_str("one sig DenyVerdict extends AuthzVerdict {}\n");
    text.push_str(
        "sig AuthzDecision { adEvent: one Event, adAction: one Action, adPlan: one PlanHash, adVerdict: one AuthzVerdict }\n",
    );
    for decision in &facts.decisions {
        let _written = writeln!(
            text,
            "one sig {} extends AuthzDecision {{}}",
            decision.ad_sig
        );
    }
    text.push_str("sig BarrierAuthzRef { barEvent: one Event, decEvent: one Event }\n");
    for reference in &facts.refs {
        let _written = writeln!(
            text,
            "one sig {} extends BarrierAuthzRef {{}}",
            reference.ref_sig
        );
    }
}

pub(crate) fn push_authz_fact_block(text: &mut String, facts: &AuthzFacts) {
    if facts.is_empty() {
        return;
    }
    text.push_str("fact GeneratedAuthzFacts {\n");
    let ad_sigs = facts
        .decisions
        .iter()
        .map(|decision| decision.ad_sig.clone())
        .collect::<Vec<_>>();
    push_exact_set(text, "AuthzDecision", &ad_sigs);
    let ref_sigs = facts
        .refs
        .iter()
        .map(|reference| reference.ref_sig.clone())
        .collect::<Vec<_>>();
    push_exact_set(text, "BarrierAuthzRef", &ref_sigs);
    for decision in &facts.decisions {
        let _written = writeln!(
            text,
            "  {}.adEvent = {}",
            decision.ad_sig, decision.event_sig
        );
        let _written = writeln!(
            text,
            "  {}.adAction = {}",
            decision.ad_sig, decision.action_sig
        );
        let _written = writeln!(text, "  {}.adPlan = {}", decision.ad_sig, decision.plan_sig);
        let _written = writeln!(
            text,
            "  {}.adVerdict = {}",
            decision.ad_sig, decision.verdict
        );
    }
    for reference in &facts.refs {
        let _written = writeln!(
            text,
            "  {}.barEvent = {}",
            reference.ref_sig, reference.barrier_event_sig
        );
        let _written = writeln!(
            text,
            "  {}.decEvent = {}",
            reference.ref_sig, reference.decision_event_sig
        );
    }
    text.push_str("}\n\n");
}

/// Emit the authz-decision assertion: every `RuntimeExecution` barrier must
/// reference an `Allow` decision bound to its action/plan.
pub(crate) fn push_authz_assertion(text: &mut String, facts: &AuthzFacts) {
    if facts.is_empty() {
        return;
    }
    text.push_str("assert GeneratedAuthzDecisionValid {\n");
    text.push_str("  all b: Event | b.kind = ExecutionBarrierLogged implies\n");
    text.push_str("    (some r: BarrierAuthzRef | r.barEvent = b and\n");
    text.push_str(
        "      (some ad: AuthzDecision | ad.adEvent = r.decEvent and ad.adVerdict = AllowVerdict\n",
    );
    text.push_str("        and ad.adAction = b.action and ad.adPlan = b.planHash))\n");
    text.push_str("}\n\n");
}

/// Emit the `check` for the authz-decision assertion.
pub(crate) fn push_authz_check(text: &mut String, scope: &str, facts: &AuthzFacts) {
    if facts.is_empty() {
        return;
    }
    let _written = writeln!(text, "check GeneratedAuthzDecisionValid for {scope}");
}

/// The `, exactly N Sig` cardinality suffix the authz sigs add to the scope.
#[must_use]
pub(crate) fn authz_scope_suffix(facts: &AuthzFacts) -> String {
    if facts.is_empty() {
        return String::new();
    }
    format!(
        ", exactly 2 AuthzVerdict, exactly {} AuthzDecision, exactly {} BarrierAuthzRef",
        facts.decisions.len(),
        facts.refs.len(),
    )
}
