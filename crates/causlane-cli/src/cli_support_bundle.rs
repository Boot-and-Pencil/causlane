//! M07.6 sanitized support bundle command.
//!
//! This module is an adapter over existing projections: replay explanation comes
//! from `ReplayTrace::verify_explain`, graph context comes from M07.2 graph
//! export, and sanitization is driven by the M07.5 redaction class/profile layer.

use causlane_contracts::CompiledDispatchBundle;
use causlane_core::{
    apply_redaction, classify_field, compile_redaction_policy, ClassifiedField, FieldPath,
    FieldVisibility, RedactionClass, RedactionClassPolicy, RedactionPolicy, RedactionSurface,
    SurfaceRedactionProfile,
};
use causlane_formal::{report_with_context, DoctorReport, FormalProfile};
use causlane_replay::{ReplayError, ReplayEvent, ReplayExplain, ReplayTrace};
use serde::Serialize;

use causlane_cli::cli_shared::DEFAULT_FORMAL_LANE;

use crate::cli_graph_export::{export_graph_model, GraphExportDto};
use crate::formal_doctor::formal_lane_checks;
use crate::{
    checked_at_token, gather_env, read_bundle, read_file, write_file, CliError, RunOutput,
};

const SUPPORT_BUNDLE_SCHEMA_VERSION: u32 = 1;
const REDACTED: &str = "[redacted]";

const FIELD_GENERATED_AT: &str = "generated_at";
const FIELD_BUNDLE_HASH: &str = "bundle.bundle_hash";
const FIELD_BUNDLE_ID: &str = "bundle.bundle_id";
const FIELD_BUNDLE_VERSION: &str = "bundle.bundle_version";
const FIELD_BUNDLE_SCHEMA_VERSION: &str = "bundle.bundle_schema_version";
const FIELD_BUNDLE_COUNTS: &str = "bundle.counts";
const FIELD_TRACE_HASH: &str = "trace.trace_hash";
const FIELD_TRACE_ACTION_ID: &str = "trace.action_id";
const FIELD_TRACE_BUNDLE_HASH: &str = "trace.bundle_hash";
const FIELD_TRACE_PREDICATE: &str = "trace.predicate";
const FIELD_TRACE_PLAN_HASH: &str = "trace.plan_hash";
const FIELD_TRACE_COUNTS: &str = "trace.counts";
const FIELD_TRACE_EVENTS: &str = "trace.events";
const FIELD_TRACE_SUBJECT_VALUES: &str = "trace.subject.values";
const FIELD_TRACE_CIRCUMSTANCE_VALUES: &str = "trace.circumstance.values";
const FIELD_TRACE_AUTHZ_PAYLOADS: &str = "trace.events.authz_decision";
const FIELD_TRACE_CAPABILITY_PAYLOADS: &str = "trace.events.execution_capability";
const FIELD_TRACE_CAPABILITY_ATTESTATIONS: &str = "trace.events.execution_capability.attestation";
const FIELD_REPLAY_EXPLAIN: &str = "replay.explain";
const FIELD_GRAPH_EXPORT: &str = "graph.export";
const FIELD_ENVIRONMENT_REPORT: &str = "environment.formal_doctor";

#[derive(Serialize)]
struct SupportBundleDto {
    schema_version: u32,
    generated_at: String,
    bundle: BundleSummaryDto,
    trace: TraceSummaryDto,
    replay: ReplayExplain,
    graph: GraphExportDto,
    environment: DoctorReport,
    redaction: RedactionReportDto,
}

#[derive(Serialize)]
struct BundleSummaryDto {
    bundle_hash: String,
    bundle_schema_version: u32,
    bundle_id: String,
    bundle_version: String,
    predicate_count: usize,
    merge_protocol_count: usize,
}

#[derive(Serialize)]
struct TraceSummaryDto {
    trace_hash: String,
    action_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bundle_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    predicate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan_hash: Option<String>,
    event_count: usize,
    subject_binding_count: usize,
    circumstance_binding_count: usize,
    subject_values: &'static str,
    circumstance_values: &'static str,
    events: Vec<EventSummaryDto>,
}

#[derive(Serialize)]
struct EventSummaryDto {
    position: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_id: Option<String>,
    kind: String,
    action_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan_hash: Option<String>,
    witness_count: usize,
    witness_ref_count: usize,
    anchor_count: usize,
    lease_count: usize,
    flags: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fact_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    occurred_at: Option<u64>,
}

#[derive(Serialize)]
struct RedactionReportDto {
    surface: &'static str,
    revealable_classes: Vec<&'static str>,
    revealed: Vec<String>,
    redacted: Vec<String>,
}

pub(crate) fn build_support_bundle(
    bundle_path: &str,
    trace_path: &str,
    graph_path: &str,
    out: &str,
    focus: Option<&str>,
) -> Result<RunOutput, CliError> {
    let bundle = read_bundle(bundle_path)?;
    let trace_json = read_file(trace_path)?;
    let trace = ReplayTrace::from_json_str(&trace_json)?;
    let graph = export_graph_model(graph_path, focus)?;
    let env = gather_env();
    let environment = report_with_context(
        &FormalProfile::Base.requirement_tokens(),
        &env,
        FormalProfile::Base,
        DEFAULT_FORMAL_LANE,
        formal_lane_checks(DEFAULT_FORMAL_LANE),
    );
    let dto = support_bundle_from_parts(&bundle, &trace, graph, environment, checked_at_token())?;
    let json = serde_json::to_string_pretty(&dto)
        .map_err(|err| CliError::Replay(ReplayError::Decode(err.to_string())))?;
    write_file(out, &json)?;
    Ok(RunOutput {
        text: format!(
            "ok: wrote {out}; bundle_hash {}; trace_events {}",
            bundle.bundle_hash.0,
            trace.events.len()
        ),
        success: true,
    })
}

fn support_bundle_from_parts(
    bundle: &CompiledDispatchBundle,
    trace: &ReplayTrace,
    graph: GraphExportDto,
    environment: DoctorReport,
    generated_at: String,
) -> Result<SupportBundleDto, CliError> {
    let profile = support_bundle_redaction_profile();
    let policy = compile_redaction_policy(&profile);
    let replay = trace.verify_explain(bundle);

    Ok(SupportBundleDto {
        schema_version: SUPPORT_BUNDLE_SCHEMA_VERSION,
        generated_at: reveal_string(&policy, FIELD_GENERATED_AT, generated_at),
        bundle: bundle_summary(bundle, &policy),
        trace: trace_summary(trace, &policy)?,
        replay,
        graph,
        environment,
        redaction: redaction_report(&profile),
    })
}

fn bundle_summary(bundle: &CompiledDispatchBundle, policy: &RedactionPolicy) -> BundleSummaryDto {
    BundleSummaryDto {
        bundle_hash: reveal_string(policy, FIELD_BUNDLE_HASH, bundle.bundle_hash.0.clone()),
        bundle_schema_version: bundle.body.bundle_schema_version,
        bundle_id: reveal_string(policy, FIELD_BUNDLE_ID, bundle.body.bundle_id.clone()),
        bundle_version: reveal_string(
            policy,
            FIELD_BUNDLE_VERSION,
            bundle.body.bundle_version.clone(),
        ),
        predicate_count: bundle.body.predicates.len(),
        merge_protocol_count: bundle.body.merge_protocols.len(),
    }
}

fn trace_summary(
    trace: &ReplayTrace,
    policy: &RedactionPolicy,
) -> Result<TraceSummaryDto, CliError> {
    let trace_hash = causlane_contracts::canonical_json_hash(trace)?;
    Ok(TraceSummaryDto {
        trace_hash: reveal_string(policy, FIELD_TRACE_HASH, trace_hash),
        action_id: reveal_string(policy, FIELD_TRACE_ACTION_ID, trace.action_id.clone()),
        bundle_hash: reveal_optional(policy, FIELD_TRACE_BUNDLE_HASH, trace.bundle_hash.clone()),
        predicate: reveal_optional(policy, FIELD_TRACE_PREDICATE, trace.predicate.clone()),
        plan_hash: reveal_optional(policy, FIELD_TRACE_PLAN_HASH, trace.plan_hash.clone()),
        event_count: trace.events.len(),
        subject_binding_count: trace.subject.len(),
        circumstance_binding_count: trace.circumstance.len(),
        subject_values: REDACTED,
        circumstance_values: REDACTED,
        events: trace
            .events
            .iter()
            .enumerate()
            .map(|(position, event)| event_summary(position, event, policy))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn event_summary(
    position: usize,
    event: &ReplayEvent,
    policy: &RedactionPolicy,
) -> Result<EventSummaryDto, CliError> {
    Ok(EventSummaryDto {
        position,
        event_id: reveal_optional(policy, FIELD_TRACE_EVENTS, event.event_id.clone()),
        kind: event_kind_token(event)?,
        action_id: reveal_string(policy, FIELD_TRACE_EVENTS, event.action_id.clone()),
        plan_hash: reveal_optional(policy, FIELD_TRACE_EVENTS, event.plan_hash.clone()),
        witness_count: event.witnesses.len(),
        witness_ref_count: event.witness_refs.len(),
        anchor_count: event.anchors.len(),
        lease_count: event.leases.len(),
        flags: event_flags(event),
        fact_kind: reveal_optional(policy, FIELD_TRACE_EVENTS, event.fact_kind.clone()),
        scope: reveal_optional(policy, FIELD_TRACE_EVENTS, event.scope.clone()),
        occurred_at: event.occurred_at,
    })
}

fn event_flags(event: &ReplayEvent) -> Vec<&'static str> {
    let mut flags = Vec::new();
    if event.impact_set_hash.is_some() {
        flags.push("impact_set_hash");
    }
    if event.execution_barrier.is_some() {
        flags.push("execution_barrier");
    }
    if event.authz_decision.is_some() {
        flags.push("authz_decision");
    }
    if event.execution_capability.is_some() {
        flags.push("execution_capability");
    }
    if event
        .execution_capability
        .as_ref()
        .is_some_and(|capability| capability.attestation.is_some())
    {
        flags.push("capability_attestation");
    }
    flags
}

fn event_kind_token(event: &ReplayEvent) -> Result<String, CliError> {
    let encoded = serde_json::to_string(&event.kind)
        .map_err(|err| CliError::Replay(ReplayError::Decode(err.to_string())))?;
    serde_json::from_str::<String>(&encoded)
        .map_err(|err| CliError::Replay(ReplayError::Decode(err.to_string())))
}

fn reveal_string(policy: &RedactionPolicy, path: &str, value: String) -> String {
    match classify_field(policy, &field_path(path)) {
        FieldVisibility::Reveal => value,
        FieldVisibility::Redact => REDACTED.to_owned(),
    }
}

fn reveal_optional(policy: &RedactionPolicy, path: &str, value: Option<String>) -> Option<String> {
    value.map(|inner| reveal_string(policy, path, inner))
}

fn redaction_report(profile: &SurfaceRedactionProfile) -> RedactionReportDto {
    let policy = compile_redaction_policy(profile);
    let fields: Vec<FieldPath> = profile
        .fields
        .iter()
        .map(|field| field.path.clone())
        .collect();
    let view = apply_redaction(&policy, &fields);
    RedactionReportDto {
        surface: "support_bundle",
        revealable_classes: vec!["public", "operational"],
        revealed: field_strings(view.revealed),
        redacted: field_strings(view.redacted),
    }
}

fn support_bundle_redaction_profile() -> SurfaceRedactionProfile {
    SurfaceRedactionProfile::new(
        RedactionSurface::SupportBundle,
        RedactionClassPolicy::revealable([RedactionClass::Public, RedactionClass::Operational]),
        vec![
            cf(FIELD_GENERATED_AT, RedactionClass::Operational),
            cf(FIELD_BUNDLE_HASH, RedactionClass::Public),
            cf(FIELD_BUNDLE_ID, RedactionClass::Public),
            cf(FIELD_BUNDLE_VERSION, RedactionClass::Public),
            cf(FIELD_BUNDLE_SCHEMA_VERSION, RedactionClass::Public),
            cf(FIELD_BUNDLE_COUNTS, RedactionClass::Operational),
            cf(FIELD_TRACE_HASH, RedactionClass::Operational),
            cf(FIELD_TRACE_ACTION_ID, RedactionClass::Operational),
            cf(FIELD_TRACE_BUNDLE_HASH, RedactionClass::Operational),
            cf(FIELD_TRACE_PREDICATE, RedactionClass::Operational),
            cf(FIELD_TRACE_PLAN_HASH, RedactionClass::Operational),
            cf(FIELD_TRACE_COUNTS, RedactionClass::Operational),
            cf(FIELD_TRACE_EVENTS, RedactionClass::Operational),
            cf(FIELD_TRACE_SUBJECT_VALUES, RedactionClass::Secret),
            cf(FIELD_TRACE_CIRCUMSTANCE_VALUES, RedactionClass::Secret),
            cf(FIELD_TRACE_AUTHZ_PAYLOADS, RedactionClass::Restricted),
            cf(FIELD_TRACE_CAPABILITY_PAYLOADS, RedactionClass::Restricted),
            cf(FIELD_TRACE_CAPABILITY_ATTESTATIONS, RedactionClass::Secret),
            cf(FIELD_REPLAY_EXPLAIN, RedactionClass::Operational),
            cf(FIELD_GRAPH_EXPORT, RedactionClass::Operational),
            cf(FIELD_ENVIRONMENT_REPORT, RedactionClass::Operational),
        ],
    )
}

fn cf(path: &str, class: RedactionClass) -> ClassifiedField {
    ClassifiedField::new(field_path(path), class)
}

fn field_path(path: &str) -> FieldPath {
    FieldPath(path.to_owned())
}

fn field_strings(fields: std::collections::BTreeSet<FieldPath>) -> Vec<String> {
    fields.into_iter().map(|path| path.0).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        redaction_report, support_bundle_from_parts, support_bundle_redaction_profile,
        FIELD_TRACE_CAPABILITY_ATTESTATIONS, FIELD_TRACE_SUBJECT_VALUES,
    };
    use crate::cli_graph_export::export_graph_model_from_text;
    use crate::formal_doctor::formal_lane_checks;
    use crate::CliError;
    use causlane_cli::cli_shared::DEFAULT_FORMAL_LANE;
    use causlane_contracts::{CompiledDispatchBundle, RegistryManifest};
    use causlane_formal::{report_with_context, EnvFacts, FormalProfile, ToolFacts};
    use causlane_replay::ReplayTrace;

    const REGISTRY: &str =
        include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");
    const TRACE: &str = include_str!("../fixtures/contracts/examples/release_promote.trace.json");
    const GRAPH: &str = r"
produced_facts: []
active_ops: []
lanes:
  - lane_id: main
    capacity: unbounded
ops:
  - action_id: release_promote
    op_index: 0
    lane: main
    requires: []
    writes: [scope:release]
";

    fn bundle() -> Result<CompiledDispatchBundle, CliError> {
        let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
        Ok(CompiledDispatchBundle::compile(&manifest)?)
    }

    fn env_report() -> causlane_formal::DoctorReport {
        let env = EnvFacts {
            rustc: ToolFacts::new(true, Some("test-rust".to_owned())),
            cargo: ToolFacts::new(true, Some("test-rust".to_owned())),
            rustup: ToolFacts::absent(),
            java: ToolFacts::absent(),
            jq: ToolFacts::new(true, None),
            python3: ToolFacts::new(true, None),
            dotnet: ToolFacts::absent(),
            alloy: ToolFacts::absent(),
            p: ToolFacts::absent(),
            cargo_kani: ToolFacts::absent(),
            verus: ToolFacts::absent(),
            elan: ToolFacts::absent(),
            lean: ToolFacts::absent(),
            lake: ToolFacts::absent(),
            z3: ToolFacts::absent(),
        };
        report_with_context(
            &FormalProfile::Base.requirement_tokens(),
            &env,
            FormalProfile::Base,
            DEFAULT_FORMAL_LANE,
            formal_lane_checks(DEFAULT_FORMAL_LANE),
        )
    }

    #[test]
    fn support_bundle_omits_raw_sensitive_trace_payloads() -> Result<(), CliError> {
        let mut trace = ReplayTrace::from_json_str(TRACE)?;
        trace.subject.push(causlane_replay::ScenarioBinding {
            key: "email".to_owned(),
            value: "alice@example.test".to_owned(),
        });
        trace.circumstance.push(causlane_replay::ScenarioBinding {
            key: "token".to_owned(),
            value: "secret-token".to_owned(),
        });
        let bundle = bundle()?;
        let graph = export_graph_model_from_text("test", GRAPH, None)?;
        let dto =
            support_bundle_from_parts(&bundle, &trace, graph, env_report(), "unix:1".to_owned())?;
        let json = serde_json::to_string(&dto)
            .map_err(|err| CliError::Usage(format!("json error: {err}")))?;

        assert!(json.contains("\"subject_values\":\"[redacted]\""));
        assert!(!json.contains("alice@example.test"));
        assert!(!json.contains("secret-token"));
        assert!(!json.contains("authz_decision\":{\""));
        Ok(())
    }

    #[test]
    fn support_bundle_redaction_reports_secret_fields_as_redacted() {
        let report = redaction_report(&support_bundle_redaction_profile());
        assert!(report
            .redacted
            .iter()
            .any(|path| path == FIELD_TRACE_SUBJECT_VALUES));
        assert!(report
            .redacted
            .iter()
            .any(|path| path == FIELD_TRACE_CAPABILITY_ATTESTATIONS));
        assert!(report.revealed.iter().any(|path| path == "graph.export"));
    }
}
