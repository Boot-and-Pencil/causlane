//! Payload-bound Alloy binding facts + discriminating assertions (P0-FM-004).
//!
//! Projects the capability/barrier/witness payloads carried by the Formal IR
//! (P0-FM-003) into Alloy sigs/relations and emits the discriminating
//! assertions the gate's negative controls must refute:
//! - `GeneratedCapabilityBindsToBarrier` (I-001): a capability derives from a
//!   barrier for the same action+plan covering its op;
//! - `GeneratedApprovalBindingHolds` (I-009): a gate-approval witness binds to
//!   the barrier's action, plan and impact set.

use core::fmt::Write as _;
use std::collections::BTreeSet;

use crate::alloy::{alloy_ident, push_exact_set};
use crate::AlloyScenarioFacts;

/// Capability/barrier op-coverage facts projected from the payload-bound IR.
pub(crate) struct CapabilityFacts {
    ops: Vec<u32>,
    capabilities: Vec<CapabilityFact>,
    coverages: Vec<CoverageFact>,
}

impl CapabilityFacts {
    fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }
}

struct CapabilityFact {
    cap_sig: String,
    cap_event_sig: String,
    barrier_event_sig: String,
    op: u32,
}

struct CoverageFact {
    cov_sig: String,
    barrier_event_sig: String,
    op: u32,
}

/// Approval/witness binding facts projected from barrier witnesses.
pub(crate) struct ApprovalFacts {
    impacts: Vec<String>,
    /// Plan-hash atoms a witness binds to that are NOT the scenario plan (the
    /// scenario plan sig is declared by the core scenario facts). These widen
    /// `PlanHash` so a witness that binds the WRONG plan is a distinct atom and
    /// `GeneratedApprovalBindingHolds` can refute it (I-009 — plan binding).
    extra_plans: Vec<String>,
    bindings: Vec<WitnessBindingFact>,
    barrier_impacts: Vec<BarrierImpactFact>,
}

impl ApprovalFacts {
    fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Plan-hash sigs (beyond the scenario plan) introduced by witness bindings.
    pub(crate) fn extra_plan_sigs(&self) -> &[String] {
        &self.extra_plans
    }
}

#[allow(clippy::struct_field_names)]
struct WitnessBindingFact {
    wb_sig: String,
    barrier_event_sig: String,
    action_sig: String,
    plan_sig: String,
    impact_sig: String,
}

#[allow(clippy::struct_field_names)]
struct BarrierImpactFact {
    bi_sig: String,
    barrier_event_sig: String,
    impact_sig: String,
}

pub(crate) fn collect_capability_facts(scenario: &AlloyScenarioFacts) -> CapabilityFacts {
    let mut ops = BTreeSet::new();
    let mut capabilities = Vec::new();
    let mut coverages = Vec::new();
    for event in &scenario.events {
        let event_sig = format!("Event_{}", alloy_ident(&event.event_id));
        if let Some(barrier) = &event.barrier {
            for op in &barrier.op_indexes {
                ops.insert(*op);
                coverages.push(CoverageFact {
                    cov_sig: format!("BC_{}_{op}", alloy_ident(&event.event_id)),
                    barrier_event_sig: event_sig.clone(),
                    op: *op,
                });
            }
        }
        if let Some(capability) = &event.capability {
            ops.insert(capability.op_index);
            capabilities.push(CapabilityFact {
                cap_sig: format!("Cap_{}", alloy_ident(&event.event_id)),
                cap_event_sig: event_sig.clone(),
                barrier_event_sig: format!("Event_{}", alloy_ident(&capability.barrier_event_id)),
                op: capability.op_index,
            });
        }
    }
    CapabilityFacts {
        ops: ops.into_iter().collect(),
        capabilities,
        coverages,
    }
}

pub(crate) fn collect_approval_facts(scenario: &AlloyScenarioFacts) -> ApprovalFacts {
    let scenario_plan_sig = format!("Plan_{}", alloy_ident(&scenario.plan_hash));
    let mut impacts = BTreeSet::new();
    let mut extra_plans = BTreeSet::new();
    let mut bindings = Vec::new();
    let mut barrier_impacts = Vec::new();
    for event in &scenario.events {
        let Some(barrier) = &event.barrier else {
            continue;
        };
        let barrier_event_sig = format!("Event_{}", alloy_ident(&event.event_id));
        let barrier_impact = alloy_ident(&barrier.impact_set_hash);
        impacts.insert(barrier_impact.clone());
        barrier_impacts.push(BarrierImpactFact {
            bi_sig: format!("BI_{}", alloy_ident(&event.event_id)),
            barrier_event_sig: barrier_event_sig.clone(),
            impact_sig: format!("Impact_{barrier_impact}"),
        });
        for witness in &barrier.witnesses {
            let (Some(bind_action), Some(bind_plan), Some(bind_impact)) = (
                &witness.binds_to_action_id,
                &witness.binds_to_plan_hash,
                &witness.binds_to_impact_set_hash,
            ) else {
                continue;
            };
            let bind_impact_ident = alloy_ident(bind_impact);
            impacts.insert(bind_impact_ident.clone());
            let plan_sig = format!("Plan_{}", alloy_ident(bind_plan));
            if plan_sig != scenario_plan_sig {
                extra_plans.insert(plan_sig.clone());
            }
            bindings.push(WitnessBindingFact {
                wb_sig: format!("WB_{}", alloy_ident(&witness.witness_event_id)),
                barrier_event_sig: barrier_event_sig.clone(),
                action_sig: format!("Action_{}", alloy_ident(bind_action)),
                plan_sig,
                impact_sig: format!("Impact_{bind_impact_ident}"),
            });
        }
    }
    ApprovalFacts {
        impacts: impacts.into_iter().collect(),
        extra_plans: extra_plans.into_iter().collect(),
        bindings,
        barrier_impacts,
    }
}

pub(crate) fn push_capability_signatures(text: &mut String, facts: &CapabilityFacts) {
    if facts.is_empty() {
        return;
    }
    text.push_str("\n// Capability binding facts (I-001 — payload-bound).\n");
    text.push_str("sig OpIndex {}\n");
    for op in &facts.ops {
        let _written = writeln!(text, "one sig Op_{op} extends OpIndex {{}}");
    }
    text.push_str(
        "sig Capability { capEvent: one Event, capBarrier: one Event, capOp: one OpIndex }\n",
    );
    for capability in &facts.capabilities {
        let _written = writeln!(
            text,
            "one sig {} extends Capability {{}}",
            capability.cap_sig
        );
    }
    text.push_str("sig BarrierCoverage { covBarrier: one Event, covOp: one OpIndex }\n");
    for coverage in &facts.coverages {
        let _written = writeln!(
            text,
            "one sig {} extends BarrierCoverage {{}}",
            coverage.cov_sig
        );
    }
}

pub(crate) fn push_approval_signatures(text: &mut String, facts: &ApprovalFacts) {
    if facts.is_empty() {
        return;
    }
    text.push_str("\n// Approval/witness binding facts (I-009 — payload-bound).\n");
    text.push_str("sig ImpactSet {}\n");
    for impact in &facts.impacts {
        let _written = writeln!(text, "one sig Impact_{impact} extends ImpactSet {{}}");
    }
    // Plan-hash atoms a witness binds to that are not the scenario plan, so a
    // wrong-plan binding is a distinct atom the assertion can refute (I-009).
    for plan in &facts.extra_plans {
        let _written = writeln!(text, "one sig {plan} extends PlanHash {{}}");
    }
    text.push_str("sig BarrierImpactBinding { biBarrier: one Event, biImpact: one ImpactSet }\n");
    for barrier_impact in &facts.barrier_impacts {
        let _written = writeln!(
            text,
            "one sig {} extends BarrierImpactBinding {{}}",
            barrier_impact.bi_sig
        );
    }
    text.push_str(
        "sig WitnessBinding { wbBarrier: one Event, wbAction: one Action, wbPlan: one PlanHash, wbImpact: one ImpactSet }\n",
    );
    for binding in &facts.bindings {
        let _written = writeln!(
            text,
            "one sig {} extends WitnessBinding {{}}",
            binding.wb_sig
        );
    }
}

pub(crate) fn push_capability_fact_block(text: &mut String, facts: &CapabilityFacts) {
    if facts.is_empty() {
        return;
    }
    text.push_str("fact GeneratedCapabilityFacts {\n");
    let op_sigs = facts
        .ops
        .iter()
        .map(|op| format!("Op_{op}"))
        .collect::<Vec<_>>();
    push_exact_set(text, "OpIndex", &op_sigs);
    let cap_sigs = facts
        .capabilities
        .iter()
        .map(|capability| capability.cap_sig.clone())
        .collect::<Vec<_>>();
    push_exact_set(text, "Capability", &cap_sigs);
    let cov_sigs = facts
        .coverages
        .iter()
        .map(|coverage| coverage.cov_sig.clone())
        .collect::<Vec<_>>();
    push_exact_set(text, "BarrierCoverage", &cov_sigs);
    for capability in &facts.capabilities {
        let _written = writeln!(
            text,
            "  {}.capEvent = {}",
            capability.cap_sig, capability.cap_event_sig
        );
        let _written = writeln!(
            text,
            "  {}.capBarrier = {}",
            capability.cap_sig, capability.barrier_event_sig
        );
        let _written = writeln!(
            text,
            "  {}.capOp = Op_{}",
            capability.cap_sig, capability.op
        );
    }
    for coverage in &facts.coverages {
        let _written = writeln!(
            text,
            "  {}.covBarrier = {}",
            coverage.cov_sig, coverage.barrier_event_sig
        );
        let _written = writeln!(text, "  {}.covOp = Op_{}", coverage.cov_sig, coverage.op);
    }
    text.push_str("}\n\n");
}

pub(crate) fn push_approval_fact_block(text: &mut String, facts: &ApprovalFacts) {
    if facts.is_empty() {
        return;
    }
    text.push_str("fact GeneratedApprovalFacts {\n");
    let impact_sigs = facts
        .impacts
        .iter()
        .map(|impact| format!("Impact_{impact}"))
        .collect::<Vec<_>>();
    push_exact_set(text, "ImpactSet", &impact_sigs);
    let bi_sigs = facts
        .barrier_impacts
        .iter()
        .map(|barrier_impact| barrier_impact.bi_sig.clone())
        .collect::<Vec<_>>();
    push_exact_set(text, "BarrierImpactBinding", &bi_sigs);
    let wb_sigs = facts
        .bindings
        .iter()
        .map(|binding| binding.wb_sig.clone())
        .collect::<Vec<_>>();
    push_exact_set(text, "WitnessBinding", &wb_sigs);
    for barrier_impact in &facts.barrier_impacts {
        let _written = writeln!(
            text,
            "  {}.biBarrier = {}",
            barrier_impact.bi_sig, barrier_impact.barrier_event_sig
        );
        let _written = writeln!(
            text,
            "  {}.biImpact = {}",
            barrier_impact.bi_sig, barrier_impact.impact_sig
        );
    }
    for binding in &facts.bindings {
        let _written = writeln!(
            text,
            "  {}.wbBarrier = {}",
            binding.wb_sig, binding.barrier_event_sig
        );
        let _written = writeln!(
            text,
            "  {}.wbAction = {}",
            binding.wb_sig, binding.action_sig
        );
        let _written = writeln!(text, "  {}.wbPlan = {}", binding.wb_sig, binding.plan_sig);
        let _written = writeln!(
            text,
            "  {}.wbImpact = {}",
            binding.wb_sig, binding.impact_sig
        );
    }
    text.push_str("}\n\n");
}

/// Emit the discriminating binding assertions.
pub(crate) fn push_binding_assertions(
    text: &mut String,
    capability_facts: &CapabilityFacts,
    approval_facts: &ApprovalFacts,
) {
    if !capability_facts.is_empty() {
        text.push_str("assert GeneratedCapabilityBindsToBarrier {\n");
        text.push_str("  all c: Capability |\n");
        text.push_str("    c.capBarrier.kind = ExecutionBarrierLogged\n");
        text.push_str("    and c.capBarrier.action = c.capEvent.action\n");
        text.push_str("    and c.capBarrier.planHash = c.capEvent.planHash\n");
        text.push_str(
            "    and (some bc: BarrierCoverage | bc.covBarrier = c.capBarrier and bc.covOp = c.capOp)\n",
        );
        text.push_str("}\n\n");
    }
    if !approval_facts.is_empty() {
        text.push_str("assert GeneratedApprovalBindingHolds {\n");
        text.push_str("  all wb: WitnessBinding |\n");
        text.push_str("    wb.wbAction = wb.wbBarrier.action\n");
        text.push_str("    and wb.wbPlan = wb.wbBarrier.planHash\n");
        text.push_str(
            "    and (some bi: BarrierImpactBinding | bi.biBarrier = wb.wbBarrier and bi.biImpact = wb.wbImpact)\n",
        );
        text.push_str("}\n\n");
    }
}

/// Emit `check` commands for the binding assertions, reusing the scope string.
pub(crate) fn push_binding_checks(
    text: &mut String,
    scope: &str,
    capability_facts: &CapabilityFacts,
    approval_facts: &ApprovalFacts,
) {
    if !capability_facts.is_empty() {
        let _written = writeln!(text, "check GeneratedCapabilityBindsToBarrier for {scope}");
    }
    if !approval_facts.is_empty() {
        let _written = writeln!(text, "check GeneratedApprovalBindingHolds for {scope}");
    }
}

/// Fact-grounding facts (P0-004 / Formal IR v2): the `(fact_kind, scope)` a
/// barrier witness CLAIMS and the pair a projection anchor CLAIMS, each paired
/// with the referenced event's own ATTESTED `(fact_kind, scope)`.
/// `GeneratedWitnessFactGrounded` and `GeneratedAnchorFactGrounded` refute a
/// witness/anchor that claims a fact/scope its producer/observed-truth event
/// never recorded — Alloy grounding the same producer attestation the replay
/// oracle checks, not the self-asserted ref.
pub(crate) struct FactGroundingFacts {
    facts: Vec<String>,
    scopes: Vec<String>,
    attestations: Vec<EventAttestation>,
    witness_claims: Vec<FactClaim>,
    anchor_claims: Vec<FactClaim>,
}

impl FactGroundingFacts {
    fn is_empty(&self) -> bool {
        self.witness_claims.is_empty() && self.anchor_claims.is_empty()
    }
}

#[allow(clippy::struct_field_names)]
struct FactClaim {
    claim_sig: String,
    event_sig: String,
    fact_sig: String,
    scope_sig: String,
}

#[allow(clippy::struct_field_names)]
struct EventAttestation {
    ea_sig: String,
    event_sig: String,
    fact_sig: String,
    scope_sig: String,
}

pub(crate) fn collect_fact_grounding_facts(scenario: &AlloyScenarioFacts) -> FactGroundingFacts {
    let mut facts = BTreeSet::new();
    let mut scopes = BTreeSet::new();
    let mut attestations = Vec::new();
    let mut attested_events = BTreeSet::new();
    let mut witness_claims = Vec::new();
    let mut anchor_claims = Vec::new();
    // Record the referenced event's own attested fact (v2 event payload). Absence
    // of an attestation leaves a claim ungrounded — also a refutation.
    let mut record_attestation = |event_id: &str,
                                  event_sig: &str,
                                  facts: &mut BTreeSet<String>,
                                  scopes: &mut BTreeSet<String>| {
        let producer = scenario
            .events
            .iter()
            .find(|candidate| candidate.event_id == event_id);
        if let Some(attested) = producer.and_then(|event| event.fact_kind.as_ref()) {
            let attested_ident = alloy_ident(attested);
            facts.insert(attested_ident.clone());
            let scope_sig = fact_scope_sig(producer.and_then(|event| event.scope.as_ref()));
            scopes.insert(scope_sig.clone());
            if attested_events.insert(event_id.to_owned()) {
                attestations.push(EventAttestation {
                    ea_sig: format!("EA_{}", alloy_ident(event_id)),
                    event_sig: event_sig.to_owned(),
                    fact_sig: format!("Fact_{attested_ident}"),
                    scope_sig,
                });
            }
        }
    };
    for event in &scenario.events {
        // Witness claims: a barrier witness's claimed fact, grounded against the
        // producer (witness) event's attestation.
        if let Some(barrier) = &event.barrier {
            for witness in &barrier.witnesses {
                let Some(claimed) = &witness.fact_kind else {
                    continue;
                };
                facts.insert(alloy_ident(claimed));
                let scope_sig = fact_scope_sig(witness.scope.as_ref());
                scopes.insert(scope_sig.clone());
                let event_sig = format!("Event_{}", alloy_ident(&witness.witness_event_id));
                witness_claims.push(FactClaim {
                    claim_sig: format!("WFC_{}", alloy_ident(&witness.witness_event_id)),
                    event_sig: event_sig.clone(),
                    fact_sig: format!("Fact_{}", alloy_ident(claimed)),
                    scope_sig,
                });
                record_attestation(
                    &witness.witness_event_id,
                    &event_sig,
                    &mut facts,
                    &mut scopes,
                );
            }
        }
        // Anchor claims: a projection anchor's claimed fact, grounded against the
        // anchored observed-truth event's attestation.
        for anchor in &event.anchor_facts {
            let Some(claimed) = &anchor.fact_kind else {
                continue;
            };
            facts.insert(alloy_ident(claimed));
            let scope_sig = fact_scope_sig(anchor.scope.as_ref());
            scopes.insert(scope_sig.clone());
            let event_sig = format!("Event_{}", alloy_ident(&anchor.event_id));
            anchor_claims.push(FactClaim {
                claim_sig: format!(
                    "AFC_{}_{}",
                    alloy_ident(&event.event_id),
                    alloy_ident(&anchor.event_id)
                ),
                event_sig: event_sig.clone(),
                fact_sig: format!("Fact_{}", alloy_ident(claimed)),
                scope_sig,
            });
            record_attestation(&anchor.event_id, &event_sig, &mut facts, &mut scopes);
        }
    }
    FactGroundingFacts {
        facts: facts.into_iter().collect(),
        scopes: scopes.into_iter().collect(),
        attestations,
        witness_claims,
        anchor_claims,
    }
}

fn push_claim_signatures(text: &mut String, sig: &str, relation: &str, claims: &[FactClaim]) {
    if claims.is_empty() {
        return;
    }
    let _written = writeln!(
        text,
        "sig {sig} {{ {relation}Event: one Event, {relation}Fact: one FactKind, {relation}Scope: one FactScope }}"
    );
    for claim in claims {
        let _written = writeln!(text, "one sig {} extends {sig} {{}}", claim.claim_sig);
    }
}

pub(crate) fn push_fact_grounding_signatures(text: &mut String, facts: &FactGroundingFacts) {
    if facts.is_empty() {
        return;
    }
    text.push_str("\n// Fact-grounding facts (P0-004 — producer/observed attestation).\n");
    text.push_str("sig FactKind {}\n");
    for fact in &facts.facts {
        let _written = writeln!(text, "one sig Fact_{fact} extends FactKind {{}}");
    }
    text.push_str("sig FactScope {}\n");
    for scope in &facts.scopes {
        let _written = writeln!(text, "one sig {scope} extends FactScope {{}}");
    }
    text.push_str(
        "sig EventAttestation { eaEvent: one Event, eaFact: one FactKind, eaScope: one FactScope }\n",
    );
    for attestation in &facts.attestations {
        let _written = writeln!(
            text,
            "one sig {} extends EventAttestation {{}}",
            attestation.ea_sig
        );
    }
    push_claim_signatures(text, "WitnessFactClaim", "wfc", &facts.witness_claims);
    push_claim_signatures(text, "AnchorFactClaim", "afc", &facts.anchor_claims);
}

fn push_claim_relations(text: &mut String, sig: &str, relation: &str, claims: &[FactClaim]) {
    if claims.is_empty() {
        return;
    }
    let claim_sigs = claims
        .iter()
        .map(|claim| claim.claim_sig.clone())
        .collect::<Vec<_>>();
    push_exact_set(text, sig, &claim_sigs);
    for claim in claims {
        let _written = writeln!(
            text,
            "  {}.{relation}Event = {}",
            claim.claim_sig, claim.event_sig
        );
        let _written = writeln!(
            text,
            "  {}.{relation}Fact = {}",
            claim.claim_sig, claim.fact_sig
        );
        let _written = writeln!(
            text,
            "  {}.{relation}Scope = {}",
            claim.claim_sig, claim.scope_sig
        );
    }
}

pub(crate) fn push_fact_grounding_fact_block(text: &mut String, facts: &FactGroundingFacts) {
    if facts.is_empty() {
        return;
    }
    text.push_str("fact GeneratedFactGroundingFacts {\n");
    let fact_sigs = facts
        .facts
        .iter()
        .map(|fact| format!("Fact_{fact}"))
        .collect::<Vec<_>>();
    push_exact_set(text, "FactKind", &fact_sigs);
    push_exact_set(text, "FactScope", &facts.scopes);
    let attestation_sigs = facts
        .attestations
        .iter()
        .map(|attestation| attestation.ea_sig.clone())
        .collect::<Vec<_>>();
    push_exact_set(text, "EventAttestation", &attestation_sigs);
    push_claim_relations(text, "WitnessFactClaim", "wfc", &facts.witness_claims);
    push_claim_relations(text, "AnchorFactClaim", "afc", &facts.anchor_claims);
    for attestation in &facts.attestations {
        let _written = writeln!(
            text,
            "  {}.eaEvent = {}",
            attestation.ea_sig, attestation.event_sig
        );
        let _written = writeln!(
            text,
            "  {}.eaFact = {}",
            attestation.ea_sig, attestation.fact_sig
        );
        let _written = writeln!(
            text,
            "  {}.eaScope = {}",
            attestation.ea_sig, attestation.scope_sig
        );
    }
    text.push_str("}\n\n");
}

fn push_grounding_assertion(text: &mut String, assertion: &str, sig: &str, relation: &str) {
    let _written = writeln!(text, "assert {assertion} {{");
    let _written = writeln!(text, "  all c: {sig} |");
    let _written = writeln!(
        text,
        "    some a: EventAttestation | a.eaEvent = c.{relation}Event and a.eaFact = c.{relation}Fact and a.eaScope = c.{relation}Scope"
    );
    text.push_str("}\n\n");
}

/// Emit the fact-grounding assertions: every claimed witness/anchor fact must be
/// backed by the referenced event's own attestation (same event/fact/scope).
pub(crate) fn push_fact_grounding_assertion(text: &mut String, facts: &FactGroundingFacts) {
    if !facts.witness_claims.is_empty() {
        push_grounding_assertion(
            text,
            "GeneratedWitnessFactGrounded",
            "WitnessFactClaim",
            "wfc",
        );
    }
    if !facts.anchor_claims.is_empty() {
        push_grounding_assertion(
            text,
            "GeneratedAnchorFactGrounded",
            "AnchorFactClaim",
            "afc",
        );
    }
}

/// Emit the `check` commands for the fact-grounding assertions.
pub(crate) fn push_fact_grounding_check(
    text: &mut String,
    scope: &str,
    facts: &FactGroundingFacts,
) {
    if !facts.witness_claims.is_empty() {
        let _written = writeln!(text, "check GeneratedWitnessFactGrounded for {scope}");
    }
    if !facts.anchor_claims.is_empty() {
        let _written = writeln!(text, "check GeneratedAnchorFactGrounded for {scope}");
    }
}

/// The `, exactly N Sig` cardinality suffix the fact-grounding sigs add.
#[must_use]
pub(crate) fn fact_grounding_scope_suffix(facts: &FactGroundingFacts) -> String {
    if facts.is_empty() {
        return String::new();
    }
    let mut suffix = format!(
        ", exactly {} FactKind, exactly {} FactScope, exactly {} EventAttestation",
        facts.facts.len(),
        facts.scopes.len(),
        facts.attestations.len(),
    );
    if !facts.witness_claims.is_empty() {
        let _written = write!(
            suffix,
            ", exactly {} WitnessFactClaim",
            facts.witness_claims.len()
        );
    }
    if !facts.anchor_claims.is_empty() {
        let _written = write!(
            suffix,
            ", exactly {} AnchorFactClaim",
            facts.anchor_claims.len()
        );
    }
    suffix
}

#[cfg(test)]
#[path = "alloy_bindings_tests.rs"]
mod alloy_bindings_tests;

fn fact_scope_sig(scope: Option<&String>) -> String {
    match scope {
        Some(scope) => format!("FactScope_{}", alloy_ident(scope)),
        None => "FactScope_absent".to_owned(),
    }
}

/// The `, exactly N Sig` cardinality suffix the binding sigs add to the scope.
#[must_use]
pub(crate) fn binding_scope_suffix(
    capability_facts: &CapabilityFacts,
    approval_facts: &ApprovalFacts,
) -> String {
    let mut suffix = String::new();
    if !capability_facts.is_empty() {
        let _written = write!(
            suffix,
            ", exactly {} Capability, exactly {} OpIndex, exactly {} BarrierCoverage",
            capability_facts.capabilities.len(),
            capability_facts.ops.len(),
            capability_facts.coverages.len(),
        );
    }
    if !approval_facts.is_empty() {
        let _written = write!(
            suffix,
            ", exactly {} ImpactSet, exactly {} BarrierImpactBinding, exactly {} WitnessBinding",
            approval_facts.impacts.len(),
            approval_facts.barrier_impacts.len(),
            approval_facts.bindings.len(),
        );
    }
    suffix
}
