//! Scope-keyed verified-merge relation for the Alloy generator (I-006 / P0-005),
//! split from `alloy.rs` for the 800-line cap.
//!
//! The replay oracle and this generator resolve the mergeable conflict-domain
//! scopes through the same `causlane_contracts::resolve_mergeable_scopes`, so the
//! Alloy `mergeable` relation cannot diverge from what replay relaxes. Only scopes
//! that actually back a lease in the scenario are kept — a mergeable scope with no
//! lease cannot relax any exclusive-lease conflict.

use std::collections::BTreeSet;

use causlane_contracts::{resolve_mergeable_scopes, CompiledDispatchBundle, TemplateBindings};

use crate::alloy::scope_sig;
use crate::{AlloyScenarioFacts, CodegenError};

/// Resolve the scenario's verified-merge conflict-domain scopes into the
/// `LeaseScope` atoms the `mergeable` relation pins.
pub(crate) fn resolve_mergeable_scope_sigs(
    bundle: &CompiledDispatchBundle,
    scenario: &AlloyScenarioFacts,
) -> Result<Vec<String>, CodegenError> {
    let Some(predicate) = bundle
        .body
        .predicates
        .iter()
        .find(|candidate| candidate.predicate == scenario.predicate_id)
    else {
        return Ok(Vec::new());
    };
    let bindings = TemplateBindings::from_pairs(
        scenario.subject.iter().cloned(),
        scenario.circumstance.iter().cloned(),
    );
    let scopes = resolve_mergeable_scopes(bundle, predicate, &bindings)
        .map_err(|err| CodegenError::Scenario(format!("mergeable scopes: {err:?}")))?;
    let lease_scopes: BTreeSet<&str> = scenario
        .events
        .iter()
        .flat_map(|event| event.leases.iter())
        .map(|lease| lease.scope.as_str())
        .collect();
    Ok(scopes
        .into_iter()
        .filter(|scope| lease_scopes.contains(scope.as_str()))
        .map(|scope| scope_sig(&scope))
        .collect())
}
