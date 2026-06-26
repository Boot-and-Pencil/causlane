//! Alloy drain-fence facts + assertion (I-007, structural), split from `alloy.rs`
//! for the 800-line cap. Projects each `DrainFenceAcquired` event's scope into a
//! `DrainFence` binding (reusing the lease's `LeaseScope` atoms) and emits the
//! `GeneratedDrainFenceClear` assertion the gate's drain negative control refutes:
//! a drain fence over a scope that still has an active, overlapping EXCLUSIVE
//! lease. The expiry refinement stays the replay oracle's job — Alloy has no time.

use core::fmt::Write as _;
use std::collections::BTreeSet;

use crate::alloy::{alloy_ident, push_exact_set, scope_sig, AlloyEventKind};
use crate::AlloyScenarioFacts;

struct DrainFenceFact {
    sig: String,
    event: String,
    scope: String,
}

pub(crate) struct DrainFacts {
    drains: Vec<DrainFenceFact>,
}

/// Collect drain-fence facts (I-007). A drain over a scope with no lease cannot
/// conflict, so only drains whose scope is backed by a declared `LeaseScope` atom
/// are emitted — that keeps `dfScope` referencing a declared atom and stays sound
/// (a scope with no active lease trivially satisfies the rule).
pub(crate) fn collect_drain_facts(scenario: &AlloyScenarioFacts) -> DrainFacts {
    let lease_scopes: BTreeSet<String> = scenario
        .events
        .iter()
        .flat_map(|event| event.leases.iter())
        .map(|lease| scope_sig(&lease.scope))
        .collect();
    let mut drains = Vec::new();
    for event in &scenario.events {
        if event.kind != AlloyEventKind::DrainFenceAcquired {
            continue;
        }
        let Some(scope) = &event.scope else {
            continue;
        };
        let scope = scope_sig(scope);
        if !lease_scopes.contains(&scope) {
            continue;
        }
        drains.push(DrainFenceFact {
            sig: format!("Drain_{}", alloy_ident(&event.event_id)),
            event: format!("Event_{}", alloy_ident(&event.event_id)),
            scope,
        });
    }
    DrainFacts { drains }
}

/// The `DrainFence` sig is emitted for EVERY scenario (so the I-007 assertion is
/// present in the positive artifact and `present_obligations` counts it); per-drain
/// atoms are added only when the scenario has drains.
pub(crate) fn push_drain_signatures(text: &mut String, facts: &DrainFacts) {
    text.push_str("\n// Drain-fence facts (I-007 — structural; replay owns expiry).\n");
    text.push_str("sig DrainFence { dfEvent: one Event, dfScope: one LeaseScope }\n");
    for drain in &facts.drains {
        let _written = writeln!(text, "one sig {} extends DrainFence {{}}", drain.sig);
    }
}

/// Pin `DrainFence` to exactly the scenario's drains (`= none` when there are none,
/// so Alloy cannot invent phantom drain atoms that would falsely fail the positive).
pub(crate) fn push_drain_fact_block(text: &mut String, facts: &DrainFacts) {
    text.push_str("fact GeneratedDrainFacts {\n");
    let drain_sigs: Vec<String> = facts.drains.iter().map(|d| d.sig.clone()).collect();
    push_exact_set(text, "DrainFence", &drain_sigs);
    for drain in &facts.drains {
        let _written = writeln!(text, "  {}.dfEvent = {}", drain.sig, drain.event);
        let _written = writeln!(text, "  {}.dfScope = {}", drain.sig, drain.scope);
    }
    text.push_str("}\n\n");
}

/// I-007 (structural): a drain fence over a scope must have no active (granted,
/// not-yet-released) overlapping EXCLUSIVE lease on that scope at the drain event.
/// Mirrors `GeneratedNoExclusiveConflicts`' active-window test.
pub(crate) fn push_drain_assertion(text: &mut String) {
    text.push_str("assert GeneratedDrainFenceClear {\n");
    text.push_str("  all d: DrainFence |\n");
    text.push_str("    no l: Lease |\n");
    text.push_str("      l.scope = d.dfScope and l.mode = ExclusiveLease and\n");
    text.push_str("      (l.leaseEvent = d.dfEvent or\n");
    text.push_str("       (l.leaseEvent in d.dfEvent.hb and\n");
    text.push_str("        not (some l.releaseEvent and l.releaseEvent in d.dfEvent.hb)))\n");
    text.push_str("}\n\n");
}

/// `, exactly N DrainFence` for the bounded `check` scope (N = number of drains).
pub(crate) fn drain_scope_suffix(facts: &DrainFacts) -> String {
    format!(", exactly {} DrainFence", facts.drains.len())
}
