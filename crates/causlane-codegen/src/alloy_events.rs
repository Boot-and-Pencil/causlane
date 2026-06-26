//! Per-event Alloy field emission + multi-action sig derivation (P1-002), split
//! from `alloy.rs` for the 800-line cap.
//!
//! A scenario may span more than one action (e.g. a readiness action whose
//! observed truth a separate promotion action witnesses). Each event names its own
//! `Action`/`PlanHash` atom, derived from the event's `action_id`/`plan_hash` (with
//! the scenario's primary as the default), so the model does not collapse distinct
//! actions onto one atom. A single-action scenario degenerates to one of each,
//! identical to before.

use core::fmt::Write as _;
use std::collections::BTreeMap;

use crate::alloy::{alloy_ident, event_kind_sig, set_expr, validate_hash, AlloyScenarioFacts};
use crate::CodegenError;

/// The distinct `Action_*` sigs in a scenario: the primary plus every event's
/// `action_id`, in stable first-seen order.
pub(crate) fn distinct_action_sigs(scenario: &AlloyScenarioFacts) -> Vec<String> {
    let mut sigs = vec![format!("Action_{}", alloy_ident(&scenario.action_id))];
    for event in &scenario.events {
        if let Some(action_id) = &event.action_id {
            let sig = format!("Action_{}", alloy_ident(action_id));
            if !sigs.contains(&sig) {
                sigs.push(sig);
            }
        }
    }
    sigs
}

/// The distinct `Plan_*` sigs in a scenario: the primary plan plus every event's
/// `plan_hash`, in stable first-seen order.
///
/// # Errors
/// Returns [`CodegenError::Scenario`] if an event plan hash is malformed.
pub(crate) fn distinct_plan_sigs(
    scenario: &AlloyScenarioFacts,
) -> Result<Vec<String>, CodegenError> {
    let mut sigs = vec![format!("Plan_{}", alloy_ident(&scenario.plan_hash))];
    for event in &scenario.events {
        if let Some(plan_hash) = &event.plan_hash {
            validate_hash("event.plan_hash", plan_hash)?;
            let sig = format!("Plan_{}", alloy_ident(plan_hash));
            if !sigs.contains(&sig) {
                sigs.push(sig);
            }
        }
    }
    Ok(sigs)
}

/// Emit per-event `action`/`kind`/`planHash`/`anchors`/`hb` fields. Each event is
/// bound to ITS action/plan atom (multi-action aware).
pub(crate) fn push_event_fields(
    text: &mut String,
    scenario: &AlloyScenarioFacts,
    event_sigs: &[String],
) -> Result<(), CodegenError> {
    let event_by_id = scenario
        .events
        .iter()
        .zip(event_sigs.iter())
        .map(|(event, sig)| (event.event_id.clone(), sig.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut prior = Vec::new();
    for (event, sig) in scenario.events.iter().zip(event_sigs.iter()) {
        let kind = event_kind_sig(event.kind);
        let action_id = event.action_id.as_deref().unwrap_or(&scenario.action_id);
        let action_sig = format!("Action_{}", alloy_ident(action_id));
        let _written = writeln!(text, "  {sig}.action = {action_sig}");
        let _written = writeln!(text, "  {sig}.kind = {kind}");
        match &event.plan_hash {
            Some(plan_hash) => {
                validate_hash("event.plan_hash", plan_hash)?;
                let plan_sig = format!("Plan_{}", alloy_ident(plan_hash));
                let _written = writeln!(text, "  {sig}.planHash = {plan_sig}");
            }
            None => {
                let _written = writeln!(text, "  no {sig}.planHash");
            }
        }
        let anchors = event
            .anchors
            .iter()
            .map(|anchor| {
                event_by_id.get(anchor).cloned().ok_or_else(|| {
                    CodegenError::Scenario(format!("unknown anchor event id {anchor}"))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let _written = writeln!(text, "  {sig}.anchors = {}", set_expr(&anchors));
        let _written = writeln!(text, "  {sig}.hb = {}", set_expr(&prior));
        prior.push(sig.clone());
    }
    Ok(())
}
