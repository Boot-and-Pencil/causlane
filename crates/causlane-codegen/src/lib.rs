//! Formal artifact generators for compiled dispatch bundles.
//!
//! Generators in this crate consume `CompiledDispatchBundle` artifacts, never
//! registry YAML, so generated models are tied to the same content-addressed
//! truth that runtime/replay consumes (ADR-0014).

#![forbid(unsafe_code)]
#![deny(warnings)]

mod alloy;
mod alloy_authz;
mod alloy_bindings;
mod alloy_drain;
mod alloy_events;
mod alloy_merge;
mod artifact;
pub mod contract;
mod coverage;
mod error;
mod identifier_check;
mod ir;
mod kani_target;
mod lean4_target;
mod obligations;
mod p_monitors;
mod receipt;
mod stale;
mod targets;
mod verus_target;
pub use alloy::{
    generate_alloy_facts, generate_alloy_facts_with_scenario, AlloyAnchorFact, AlloyEventKind,
    AlloyLeaseFact, AlloyLeaseMode, AlloyScenarioEvent, AlloyScenarioFacts,
};
pub(crate) use artifact::{artifact_header, artifact_header_with_prefix};
pub use artifact::{generated_artifact_hash, FormalTarget, GeneratedArtifact, ReceiptObligation};
pub use contract::{
    CodegenContracts, CoverageReporter, FormalGenerator, FormalIrBuilder, KaniGenerator,
    Lean4Generator, PGenerator, StaleChecker, VerusGenerator,
};
pub use coverage::{
    build_report, default_invariant_matrix, derive_artifact_status, invariant_matrix,
    overall_status, ArtifactFacts, ArtifactReport, ArtifactStatus, CoverageMeta,
    FormalCoverageReport, InvariantCoverage, InvariantStatus, LaneStatus, NegativeControl,
    NegativeControlStatus, ReportStatus, COVERAGE_SCHEMA_VERSION,
};
pub use error::CodegenError;
pub use ir::{
    build_formal_ir, FormalAuthzDecisionPayload, FormalBarrierPayload, FormalCapabilityPayload,
    FormalClaim, FormalEffectTemplate, FormalEvent, FormalIr, FormalLeaseFact, FormalPredicate,
    FormalWitnessPayload, FormalWitnessRequirement,
};
pub use kani_target::generate_kani_harness;
pub use lean4_target::generate_lean4_proof;
pub use receipt::{receipt_to_json, FormalReceipt, ReceiptScope, ToolRunResult};
pub use stale::{stale_check, stale_check_with_expected};
pub use targets::{generate_p_monitor, generate_verus_proof};

/// Version of the generator output schema.
pub const GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use causlane_contracts::{CompiledDispatchBundle, ContractError, RegistryManifest};

    use super::{
        generate_alloy_facts, generate_alloy_facts_with_scenario, stale_check,
        stale_check_with_expected, AlloyEventKind, AlloyLeaseFact, AlloyLeaseMode,
        AlloyScenarioEvent, AlloyScenarioFacts, CodegenError, GENERATOR_VERSION,
    };

    const REGISTRY: &str =
        include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");

    fn demo_bundle() -> Result<CompiledDispatchBundle, ContractError> {
        let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
        CompiledDispatchBundle::compile(&manifest)
    }

    #[test]
    fn generated_alloy_mentions_source_bundle() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let generated = generate_alloy_facts(&bundle, None)?;
        assert!(generated.text.contains("// source_bundle_hash: sha256:"));
        assert!(generated.text.contains("// formal_ir_hash: sha256:"));
        assert!(generated.text.contains("Pred_release_promote_candidate"));
        assert!(generated.artifact_hash.starts_with("sha256:"));
        Ok(())
    }

    #[test]
    fn generated_alloy_can_include_scenario_facts() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let plan_hash = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let scenario = AlloyScenarioFacts {
            scenario_hash:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            action_id: "act".to_owned(),
            plan_hash: plan_hash.to_owned(),
            expected_result: "pass".to_owned(),
            expected_error_code: None,
            formal_obligations: vec!["I-001".to_owned()],
            predicate_id: String::new(),
            subject: Vec::new(),
            circumstance: Vec::new(),
            events: vec![
                AlloyScenarioEvent {
                    event_id: "barrier".to_owned(),
                    kind: AlloyEventKind::ExecutionBarrierLogged,
                    action_id: Some("act".to_owned()),
                    plan_hash: Some(plan_hash.to_owned()),
                    op_index: None,
                    fact_kind: None,
                    scope: None,
                    anchors: Vec::new(),
                    anchor_facts: Vec::new(),
                    leases: Vec::new(),
                    barrier: None,
                    capability: None,
                    authz_decision: None,
                },
                AlloyScenarioEvent {
                    event_id: "execution".to_owned(),
                    kind: AlloyEventKind::ExecutionStarted,
                    action_id: Some("act".to_owned()),
                    plan_hash: Some(plan_hash.to_owned()),
                    op_index: None,
                    fact_kind: None,
                    scope: None,
                    anchors: Vec::new(),
                    anchor_facts: Vec::new(),
                    leases: Vec::new(),
                    barrier: None,
                    capability: None,
                    authz_decision: None,
                },
            ],
        };
        let generated = generate_alloy_facts_with_scenario(&bundle, &scenario)?;
        assert!(generated.text.contains("open core/causlane_core"));
        assert!(generated.text.contains("Event_barrier"));
        assert!(generated.text.contains("GeneratedTraceSatisfiesCore"));
        Ok(())
    }

    fn exclusive_lease(lease_id: &str) -> AlloyLeaseFact {
        AlloyLeaseFact {
            lease_id: lease_id.to_owned(),
            resource: "environment_write".to_owned(),
            scope: "environment:staging".to_owned(),
            mode: AlloyLeaseMode::Exclusive,
            epoch: 0,
        }
    }

    fn lease_event(
        event_id: &str,
        kind: AlloyEventKind,
        lease: AlloyLeaseFact,
    ) -> AlloyScenarioEvent {
        AlloyScenarioEvent {
            event_id: event_id.to_owned(),
            kind,
            action_id: Some("act".to_owned()),
            plan_hash: None,
            op_index: None,
            fact_kind: None,
            scope: None,
            anchors: Vec::new(),
            anchor_facts: Vec::new(),
            leases: vec![lease],
            barrier: None,
            capability: None,
            authz_decision: None,
        }
    }

    fn lease_scenario(events: Vec<AlloyScenarioEvent>) -> AlloyScenarioFacts {
        AlloyScenarioFacts {
            scenario_hash:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            action_id: "act".to_owned(),
            plan_hash: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .to_owned(),
            expected_result: "pass".to_owned(),
            expected_error_code: None,
            formal_obligations: vec!["I-006".to_owned()],
            predicate_id: String::new(),
            subject: Vec::new(),
            circumstance: Vec::new(),
            events,
        }
    }

    // P0-FM-004: `GeneratedNoExclusiveConflicts` must be interval-aware — it
    // mirrors `LeaseTable::grant`, conflicting two same-resource+scope leases
    // only while their active windows `[grant .. release)` overlap, and on at
    // least one exclusive (not both). This guards the emitted facts so a
    // regression back to the mode-only predicate is caught without a SAT run.
    #[test]
    fn alloy_lease_conflict_is_interval_and_merge_aware() -> Result<(), Box<dyn std::error::Error>>
    {
        let bundle = demo_bundle()?;

        // grant -> release -> re-grant of the SAME exclusive resource: the
        // windows do not overlap, so the release event must be wired and the
        // predicate must be present to clear the conflict.
        let sequential = lease_scenario(vec![
            lease_event(
                "evt_lease_a",
                AlloyEventKind::ConstraintLeaseGranted,
                exclusive_lease("lease_a"),
            ),
            lease_event(
                "evt_release_a",
                AlloyEventKind::ConstraintLeaseReleased,
                exclusive_lease("lease_a"),
            ),
            lease_event(
                "evt_lease_b",
                AlloyEventKind::ConstraintLeaseGranted,
                exclusive_lease("lease_b"),
            ),
        ]);
        let seq_text = generate_alloy_facts_with_scenario(&bundle, &sequential)?.text;

        // The predicate is interval + merge aware (oracle-faithful), not mode-only.
        assert!(seq_text.contains("a.mode = ExclusiveLease or b.mode = ExclusiveLease"));
        assert!(seq_text.contains("a.releaseEvent in b.leaseEvent.hb"));
        // P0-005: the merge relaxation is per-scope, not a global `some mergeable or`.
        assert!(seq_text.contains("a.scope not in BundleFacts.mergeable"));
        assert!(seq_text.contains("mergeable: set LeaseScope"));
        // The Lease sig carries the active-window upper bound.
        assert!(seq_text.contains("releaseEvent: lone Event"));
        // lease_a's release window upper bound is wired to the release event.
        assert!(seq_text.contains("Lease_lease_a.releaseEvent = Event_evt_release_a"));

        // grant -> grant of the SAME exclusive resource, never released: the
        // windows overlap forever, so neither lease carries a release bound.
        let overlapping = lease_scenario(vec![
            lease_event(
                "evt_lease_a",
                AlloyEventKind::ConstraintLeaseGranted,
                exclusive_lease("lease_a"),
            ),
            lease_event(
                "evt_lease_b",
                AlloyEventKind::ConstraintLeaseGranted,
                exclusive_lease("lease_b"),
            ),
        ]);
        let overlap_text = generate_alloy_facts_with_scenario(&bundle, &overlapping)?.text;
        assert!(overlap_text.contains("no Lease_lease_a.releaseEvent"));
        assert!(overlap_text.contains("no Lease_lease_b.releaseEvent"));
        Ok(())
    }

    #[test]
    fn stale_check_rejects_manual_edit() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let generated = generate_alloy_facts(&bundle, None)?;
        let edited = generated
            .text
            .replace("CompiledBundleFacts", "TamperedFacts");
        let receipt = format!(
            r#"{{
              "schema_version": 2,
              "receipt_kind": "codegen",
              "artifact_kind": "alloy",
              "target": "alloy",
              "tool": "causlane-codegen",
              "tool_version": "0.0.0",
              "generator_version": "0.0.0",
              "source_bundle_hash": "{}",
              "formal_ir_hash": "{}",
              "scenario_hash": null,
              "core_model_hash": null,
              "generated_artifact_hash": "{}",
              "command": "test",
              "expected_result": "generated",
              "actual_result": "generated",
              "invariant_ids": ["I-001"],
              "scope": {{ "predicates": 1, "scenarios": 0 }},
              "checked_at": "1970-01-01T00:00:00Z"
            }}"#,
            bundle.bundle_hash.0, generated.formal_ir_hash, generated.artifact_hash
        );
        let result = stale_check(&bundle, &edited, Some(&receipt));
        assert!(matches!(result, Err(CodegenError::Stale(_))));
        Ok(())
    }

    #[test]
    fn stale_check_rejects_wrong_scenario_hash() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let generated = generate_alloy_facts(
            &bundle,
            Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        )?;
        let result = stale_check_with_expected(
            &bundle,
            &generated.text,
            None,
            Some("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        );
        assert!(matches!(result, Err(CodegenError::Stale(_))));
        Ok(())
    }

    #[test]
    fn stale_check_rejects_stale_generator_version() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let generated = generate_alloy_facts(&bundle, None)?;
        // Simulate an artifact produced by an older generator: only the header
        // generator_version line is rewritten (no receipt needed).
        let edited = generated
            .text
            .lines()
            .map(|line| {
                if line.starts_with("// generator_version:") {
                    "// generator_version: 0.0.0-old".to_owned()
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let result = stale_check(&bundle, &edited, None);
        assert!(matches!(result, Err(CodegenError::Stale(_))));
        Ok(())
    }

    #[test]
    fn stale_check_rejects_mismatched_invariant_ids() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let generated = generate_alloy_facts(&bundle, None)?;
        // Receipt declares invariant_ids that the header does not carry.
        let receipt = format!(
            r#"{{
              "schema_version": 2,
              "receipt_kind": "codegen",
              "artifact_kind": "alloy",
              "target": "alloy",
              "tool": "causlane-codegen",
              "tool_version": "0.0.0",
              "generator_version": "{}",
              "source_bundle_hash": "{}",
              "formal_ir_hash": "{}",
              "scenario_hash": null,
              "core_model_hash": null,
              "generated_artifact_hash": "{}",
              "command": "test",
              "expected_result": "generated",
              "actual_result": "generated",
              "invariant_ids": ["I-999"],
              "scope": {{ "predicates": 1, "scenarios": 0 }},
              "checked_at": "1970-01-01T00:00:00Z"
            }}"#,
            GENERATOR_VERSION,
            bundle.bundle_hash.0,
            generated.formal_ir_hash,
            generated.artifact_hash
        );
        let result = stale_check(&bundle, &generated.text, Some(&receipt));
        assert!(matches!(result, Err(CodegenError::Stale(_))));
        Ok(())
    }

    #[test]
    fn stale_check_without_a_receipt_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let generated = generate_alloy_facts(&bundle, None)?;
        // H4: a valid, unedited artifact still cannot be proven fresh on editable
        // header fields alone — without a receipt the stale-check fails closed.
        let result = stale_check(&bundle, &generated.text, None);
        assert!(matches!(result, Err(CodegenError::Stale(_))));
        Ok(())
    }

    #[test]
    fn stale_check_without_receipt_rejects_a_body_edit() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let generated = generate_alloy_facts(&bundle, None)?;
        // The H4 hole: a hand-edited body with an intact header previously passed
        // when no receipt was supplied. It must now be rejected.
        let edited = generated
            .text
            .replace("CompiledBundleFacts", "TamperedFacts");
        let result = stale_check(&bundle, &edited, None);
        assert!(matches!(result, Err(CodegenError::Stale(_))));
        Ok(())
    }

    #[test]
    fn stale_check_accepts_a_matching_receipt() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = demo_bundle()?;
        let generated = generate_alloy_facts(&bundle, None)?;
        // Fail-closed must not break the happy path: a receipt that matches the
        // generated artifact is accepted.
        let invariant_ids = serde_json::to_string(&generated.invariant_ids)?;
        let receipt = format!(
            r#"{{
              "schema_version": 2,
              "receipt_kind": "codegen",
              "artifact_kind": "facts",
              "target": "alloy",
              "tool": "causlane-codegen",
              "tool_version": "0.0.0",
              "generator_version": "{generator}",
              "source_bundle_hash": "{bundle_hash}",
              "formal_ir_hash": "{ir_hash}",
              "scenario_hash": null,
              "core_model_hash": null,
              "generated_artifact_hash": "{artifact_hash}",
              "command": "test",
              "expected_result": "generated",
              "actual_result": "generated",
              "invariant_ids": {invariant_ids},
              "scope": {{ "predicates": 1, "scenarios": 0 }},
              "checked_at": "1970-01-01T00:00:00Z"
            }}"#,
            generator = GENERATOR_VERSION,
            bundle_hash = bundle.bundle_hash.0,
            ir_hash = generated.formal_ir_hash,
            artifact_hash = generated.artifact_hash,
        );
        stale_check(&bundle, &generated.text, Some(&receipt))?;
        Ok(())
    }
}
