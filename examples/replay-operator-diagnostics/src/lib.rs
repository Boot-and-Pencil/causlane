#![forbid(unsafe_code)]
#![deny(warnings)]

use std::fmt;

use causlane_contracts::{CompiledDispatchBundle, ContractError, RegistryManifest};
use causlane_replay::{CausalLocation, ReplayError, ReplayExplain, ReplayScenario};

const RELEASE_REGISTRY_YAML: &str =
    include_str!("../../../contracts/examples/release_promote.registry.yaml");
const PROJECTION_REGISTRY_YAML: &str =
    include_str!("../../../contracts/examples/projection_readonly.registry.yaml");
const MULTI_ACTION_REGISTRY_YAML: &str =
    include_str!("../../../contracts/examples/multi_action_reference.registry.yaml");

const RELEASE_SUCCESS_YAML: &str =
    include_str!("../../../contracts/scenarios/release_promote_success.scenario.yaml");
const READ_ONLY_SIDECAR_SUCCESS_YAML: &str =
    include_str!("../../../contracts/scenarios/read_only_sidecar_success.scenario.yaml");
const MULTI_ACTION_REFERENCE_YAML: &str =
    include_str!("../../../contracts/scenarios/multi_action_reference.scenario.yaml");
const OBSERVED_WITHOUT_EXECUTION_YAML: &str =
    include_str!("../../../contracts/scenarios/observed_without_execution_invalid.scenario.yaml");
const PROJECTION_WITHOUT_ANCHOR_YAML: &str =
    include_str!("../../../contracts/scenarios/projection_without_anchor_invalid.scenario.yaml");
const PROJECTION_ANCHOR_WRONG_SCOPE_YAML: &str = include_str!(
    "../../../contracts/scenarios/projection_anchor_wrong_scope_invalid.scenario.yaml"
);
const EVENT_AFTER_CLOSED_YAML: &str =
    include_str!("../../../contracts/scenarios/event_after_closed_invalid.scenario.yaml");

const RELEASE_PLAN_HASH: &str =
    "sha256:a241276dc389e9197710cd415072e850036429799d636873f47ae8a1bc44d47b";
const WRONG_BUNDLE_HASH: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// Summary returned by the replay operator diagnostics example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayOperatorDiagnosticsSummary {
    /// Compiled bundles used by the workflow.
    pub compiled_bundles: usize,
    /// Accepted traces checked through `verify_explain`.
    pub accepted_cases: usize,
    /// Rejected traces checked through `verify_explain`.
    pub rejected_cases: usize,
    /// Rejections that carried structured causal location fields.
    pub located_rejections: usize,
    /// Reports rendered to JSON.
    pub json_reports: usize,
    /// Reports rendered to human-readable text.
    pub human_reports: usize,
    /// Strict bundle-binding negative controls observed.
    pub strict_negative_controls: usize,
}

/// Per-case diagnostic captured by the example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayOperatorDiagnosticReport {
    /// Stable example case label.
    pub case: &'static str,
    /// Whether replay accepted the trace.
    pub accepted: bool,
    /// Violated invariant when the rejection is invariant-bearing.
    pub invariant: Option<String>,
    /// Stable replay error code when rejected.
    pub error_code: Option<String>,
    /// Whether the rejection carried any causal location fields.
    pub has_location: bool,
    /// Number of invariants reported by the explain path.
    pub checked_invariants: usize,
    /// Rendered JSON report length.
    pub json_bytes: usize,
    /// Rendered human report length.
    pub human_bytes: usize,
}

/// Error type for the replay operator diagnostics example.
#[derive(Debug)]
pub enum ReplayOperatorDiagnosticsError {
    /// Registry or bundle compilation failed.
    Contract(ContractError),
    /// Scenario loading, replay verification or explain rendering failed.
    Replay(ReplayError),
    /// A case accepted when the example expected a rejection.
    UnexpectedAccepted(&'static str),
    /// A case rejected when the example expected acceptance.
    UnexpectedRejected(&'static str),
    /// Explain reported the wrong invariant id.
    InvariantMismatch {
        /// Case label.
        case: &'static str,
        /// Expected invariant id.
        expected: Option<&'static str>,
        /// Actual invariant id.
        actual: Option<String>,
    },
    /// Explain reported the wrong stable error code.
    ErrorCodeMismatch {
        /// Case label.
        case: &'static str,
        /// Expected stable error code.
        expected: Option<&'static str>,
        /// Actual stable error code.
        actual: Option<String>,
    },
    /// Explain omitted or populated an unexpected causal location.
    LocationMismatch {
        /// Case label.
        case: &'static str,
        /// Expected location shape.
        expected: &'static str,
        /// Actual location debug rendering.
        actual: String,
    },
    /// The human or JSON report was empty.
    EmptyRenderedReport {
        /// Case label.
        case: &'static str,
        /// Output format.
        format: &'static str,
    },
    /// Explain emitted a non-canonical bundle or trace hash token.
    NonCanonicalHash {
        /// Case label.
        case: &'static str,
        /// Hash field.
        field: &'static str,
        /// Actual hash value.
        actual: String,
    },
    /// Strict bundle binding accepted a trace without an explicit bundle hash.
    StrictBundleHashAccepted,
    /// Strict bundle binding rejected with a different error than expected.
    StrictBundleHashMismatch(String),
}

impl fmt::Display for ReplayOperatorDiagnosticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contract(error) => write!(f, "contract fixture failed: {error}"),
            Self::Replay(error) => write!(f, "replay diagnostics failed: {error}"),
            Self::UnexpectedAccepted(case) => {
                write!(f, "diagnostic case unexpectedly accepted: {case}")
            }
            Self::UnexpectedRejected(case) => {
                write!(f, "diagnostic case unexpectedly rejected: {case}")
            }
            Self::InvariantMismatch {
                case,
                expected,
                actual,
            } => write!(
                f,
                "diagnostic case {case} reported invariant {actual:?}, expected {expected:?}"
            ),
            Self::ErrorCodeMismatch {
                case,
                expected,
                actual,
            } => write!(
                f,
                "diagnostic case {case} reported error code {actual:?}, expected {expected:?}"
            ),
            Self::LocationMismatch {
                case,
                expected,
                actual,
            } => write!(
                f,
                "diagnostic case {case} reported location {actual}, expected {expected}"
            ),
            Self::EmptyRenderedReport { case, format } => {
                write!(f, "diagnostic case {case} rendered empty {format} output")
            }
            Self::NonCanonicalHash {
                case,
                field,
                actual,
            } => write!(
                f,
                "diagnostic case {case} rendered non-canonical {field}: {actual}"
            ),
            Self::StrictBundleHashAccepted => {
                f.write_str("strict bundle binding accepted an unbound trace")
            }
            Self::StrictBundleHashMismatch(actual) => write!(
                f,
                "strict bundle binding rejected with unexpected error: {actual}"
            ),
        }
    }
}

impl std::error::Error for ReplayOperatorDiagnosticsError {}

impl From<ContractError> for ReplayOperatorDiagnosticsError {
    fn from(error: ContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<ReplayError> for ReplayOperatorDiagnosticsError {
    fn from(error: ReplayError) -> Self {
        Self::Replay(error)
    }
}

/// Run the replay operator diagnostics workflow through public APIs.
///
/// # Errors
/// Returns an error if bundle compilation, scenario parsing, report rendering
/// or a deterministic fail-closed replay control diverges from expectation.
#[must_use = "the runnable example result must be checked"]
pub fn run_replay_operator_diagnostics(
) -> Result<ReplayOperatorDiagnosticsSummary, ReplayOperatorDiagnosticsError> {
    let reports = collect_replay_operator_diagnostics()?;
    let accepted_cases = reports.iter().filter(|report| report.accepted).count();
    let rejected_cases = reports.len() - accepted_cases;
    let located_rejections = reports
        .iter()
        .filter(|report| !report.accepted && report.has_location)
        .count();
    let json_reports = reports
        .iter()
        .filter(|report| report.json_bytes > 0)
        .count();
    let human_reports = reports
        .iter()
        .filter(|report| report.human_bytes > 0)
        .count();
    let strict_negative_controls = verify_strict_bundle_hash_negative_control()?;

    Ok(ReplayOperatorDiagnosticsSummary {
        compiled_bundles: 3,
        accepted_cases,
        rejected_cases,
        located_rejections,
        json_reports,
        human_reports,
        strict_negative_controls,
    })
}

/// Collect replay diagnostics across operator-facing success and failure cases.
///
/// # Errors
/// Returns an error if a scenario fails to parse, a bundle fails to compile, or
/// the explain output does not match the documented stable diagnostics.
#[must_use = "the diagnostic reports must be inspected"]
pub fn collect_replay_operator_diagnostics(
) -> Result<Vec<ReplayOperatorDiagnosticReport>, ReplayOperatorDiagnosticsError> {
    let release_bundle = compile_bundle(RELEASE_REGISTRY_YAML)?;
    let projection_bundle = compile_bundle(PROJECTION_REGISTRY_YAML)?;
    let multi_action_bundle = compile_bundle(MULTI_ACTION_REGISTRY_YAML)?;

    let mut reports = Vec::new();
    for case in release_cases() {
        reports.push(explain_case(case, &release_bundle)?);
    }
    for case in projection_cases() {
        reports.push(explain_case(case, &projection_bundle)?);
    }
    for case in multi_action_cases() {
        reports.push(explain_case(case, &multi_action_bundle)?);
    }
    reports.push(verify_bundle_hash_mismatch(&release_bundle)?);
    Ok(reports)
}

/// Verify strict bundle-bound replay refuses an unbound trace.
///
/// # Errors
/// Returns an error if strict replay accepts a trace without `bundle_hash` or
/// rejects for a different reason.
#[must_use = "negative controls must be counted"]
pub fn verify_strict_bundle_hash_negative_control() -> Result<usize, ReplayOperatorDiagnosticsError>
{
    let bundle = compile_bundle(RELEASE_REGISTRY_YAML)?;
    let scenario = ReplayScenario::from_yaml_str(RELEASE_SUCCESS_YAML)?;
    let trace = scenario.to_trace();
    match trace.verify_with_bundle_strict(&bundle) {
        Ok(()) => Err(ReplayOperatorDiagnosticsError::StrictBundleHashAccepted),
        Err(ReplayError::MissingTraceBundleHash { expected })
            if expected == bundle.bundle_hash.0 =>
        {
            Ok(1)
        }
        Err(error) => Err(ReplayOperatorDiagnosticsError::StrictBundleHashMismatch(
            format!("{error:?}"),
        )),
    }
}

fn compile_bundle(yaml: &str) -> Result<CompiledDispatchBundle, ReplayOperatorDiagnosticsError> {
    let manifest = RegistryManifest::from_yaml_str(yaml)?;
    Ok(CompiledDispatchBundle::compile(&manifest)?)
}

fn release_cases() -> [DiagnosticCase; 4] {
    [
        DiagnosticCase {
            label: "release_promote_success",
            scenario_yaml: RELEASE_SUCCESS_YAML,
            accepted: true,
            invariant: None,
            error_code: None,
            location: ExpectedLocation::Empty,
        },
        DiagnosticCase {
            label: "observed_without_execution",
            scenario_yaml: OBSERVED_WITHOUT_EXECUTION_YAML,
            accepted: false,
            invariant: Some("I-002"),
            error_code: Some("ObservedWithoutExecution"),
            location: ExpectedLocation::ActionPlan {
                action_id: "act_promote_123",
                plan_hash: RELEASE_PLAN_HASH,
            },
        },
        DiagnosticCase {
            label: "projection_without_anchor",
            scenario_yaml: PROJECTION_WITHOUT_ANCHOR_YAML,
            accepted: false,
            invariant: Some("I-003"),
            error_code: Some("ProjectionWithoutAnchor"),
            location: ExpectedLocation::Event("evt_projection"),
        },
        DiagnosticCase {
            label: "event_after_closed",
            scenario_yaml: EVENT_AFTER_CLOSED_YAML,
            accepted: false,
            invariant: Some("I-008"),
            error_code: Some("EventAfterClosed"),
            location: ExpectedLocation::ActionEvent {
                action_id: "act_promote_123",
                event_id: "evt_after_closed",
            },
        },
    ]
}

fn projection_cases() -> [DiagnosticCase; 2] {
    [
        DiagnosticCase {
            label: "read_only_sidecar_success",
            scenario_yaml: READ_ONLY_SIDECAR_SUCCESS_YAML,
            accepted: true,
            invariant: None,
            error_code: None,
            location: ExpectedLocation::Empty,
        },
        DiagnosticCase {
            label: "projection_anchor_wrong_scope",
            scenario_yaml: PROJECTION_ANCHOR_WRONG_SCOPE_YAML,
            accepted: false,
            invariant: Some("I-003"),
            error_code: Some("AnchorAttestationMismatch"),
            location: ExpectedLocation::Anchor {
                event_id: "evt_projection",
                anchor_event_id: "evt_observed_promoted",
            },
        },
    ]
}

fn multi_action_cases() -> [DiagnosticCase; 1] {
    [DiagnosticCase {
        label: "multi_action_reference",
        scenario_yaml: MULTI_ACTION_REFERENCE_YAML,
        accepted: true,
        invariant: None,
        error_code: None,
        location: ExpectedLocation::Empty,
    }]
}

fn verify_bundle_hash_mismatch(
    bundle: &CompiledDispatchBundle,
) -> Result<ReplayOperatorDiagnosticReport, ReplayOperatorDiagnosticsError> {
    let scenario = ReplayScenario::from_yaml_str(RELEASE_SUCCESS_YAML)?;
    let mut trace = scenario.to_trace();
    trace.bundle_hash = Some(WRONG_BUNDLE_HASH.to_owned());
    let explain = trace.verify_explain(bundle);
    require_case_explain(
        DiagnosticCase {
            label: "bundle_hash_mismatch",
            scenario_yaml: RELEASE_SUCCESS_YAML,
            accepted: false,
            invariant: None,
            error_code: Some("BundleHashMismatch"),
            location: ExpectedLocation::Empty,
        },
        &explain,
    )
}

fn explain_case(
    case: DiagnosticCase,
    bundle: &CompiledDispatchBundle,
) -> Result<ReplayOperatorDiagnosticReport, ReplayOperatorDiagnosticsError> {
    let scenario = ReplayScenario::from_yaml_str(case.scenario_yaml)?;
    let explain = scenario.to_trace().verify_explain(bundle);
    require_case_explain(case, &explain)
}

fn require_case_explain(
    case: DiagnosticCase,
    explain: &ReplayExplain,
) -> Result<ReplayOperatorDiagnosticReport, ReplayOperatorDiagnosticsError> {
    if case.accepted && !explain.accepted {
        return Err(ReplayOperatorDiagnosticsError::UnexpectedRejected(
            case.label,
        ));
    }
    if !case.accepted && explain.accepted {
        return Err(ReplayOperatorDiagnosticsError::UnexpectedAccepted(
            case.label,
        ));
    }
    if explain.invariant.as_deref() != case.invariant {
        return Err(ReplayOperatorDiagnosticsError::InvariantMismatch {
            case: case.label,
            expected: case.invariant,
            actual: explain.invariant.clone(),
        });
    }
    if explain.error_code.as_deref() != case.error_code {
        return Err(ReplayOperatorDiagnosticsError::ErrorCodeMismatch {
            case: case.label,
            expected: case.error_code,
            actual: explain.error_code.clone(),
        });
    }
    require_location(case.label, &case.location, &explain.causal_location)?;
    render_report(case.label, explain)
}

fn require_location(
    case: &'static str,
    expected: &ExpectedLocation,
    actual: &CausalLocation,
) -> Result<(), ReplayOperatorDiagnosticsError> {
    let matches = match expected {
        ExpectedLocation::Empty => actual.is_empty(),
        ExpectedLocation::ActionPlan {
            action_id,
            plan_hash,
        } => {
            actual.action_id.as_deref() == Some(*action_id)
                && actual.plan_hash.as_deref() == Some(*plan_hash)
        }
        ExpectedLocation::Event(event_id) => actual.event_id.as_deref() == Some(*event_id),
        ExpectedLocation::Anchor {
            event_id,
            anchor_event_id,
        } => {
            actual.event_id.as_deref() == Some(*event_id)
                && actual.anchor_event_id.as_deref() == Some(*anchor_event_id)
        }
        ExpectedLocation::ActionEvent {
            action_id,
            event_id,
        } => {
            actual.action_id.as_deref() == Some(*action_id)
                && actual.event_id.as_deref() == Some(*event_id)
        }
    };
    if matches {
        return Ok(());
    }
    Err(ReplayOperatorDiagnosticsError::LocationMismatch {
        case,
        expected: expected.description(),
        actual: format!("{actual:?}"),
    })
}

fn render_report(
    case: &'static str,
    explain: &ReplayExplain,
) -> Result<ReplayOperatorDiagnosticReport, ReplayOperatorDiagnosticsError> {
    require_hash(case, "bundle_hash", &explain.bundle_hash)?;
    require_hash(case, "trace_hash", &explain.trace_hash)?;

    let json = explain.to_json_pretty()?;
    if json.is_empty() {
        return Err(ReplayOperatorDiagnosticsError::EmptyRenderedReport {
            case,
            format: "json",
        });
    }
    let human = explain.to_human();
    if human.is_empty() {
        return Err(ReplayOperatorDiagnosticsError::EmptyRenderedReport {
            case,
            format: "human",
        });
    }

    Ok(ReplayOperatorDiagnosticReport {
        case,
        accepted: explain.accepted,
        invariant: explain.invariant.clone(),
        error_code: explain.error_code.clone(),
        has_location: !explain.causal_location.is_empty(),
        checked_invariants: explain.checked_invariants.len(),
        json_bytes: json.len(),
        human_bytes: human.len(),
    })
}

fn require_hash(
    case: &'static str,
    field: &'static str,
    value: &str,
) -> Result<(), ReplayOperatorDiagnosticsError> {
    let hash = value.strip_prefix("sha256:");
    if hash.is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())) {
        Ok(())
    } else {
        Err(ReplayOperatorDiagnosticsError::NonCanonicalHash {
            case,
            field,
            actual: value.to_owned(),
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct DiagnosticCase {
    label: &'static str,
    scenario_yaml: &'static str,
    accepted: bool,
    invariant: Option<&'static str>,
    error_code: Option<&'static str>,
    location: ExpectedLocation,
}

#[derive(Clone, Copy, Debug)]
enum ExpectedLocation {
    Empty,
    ActionPlan {
        action_id: &'static str,
        plan_hash: &'static str,
    },
    Event(&'static str),
    Anchor {
        event_id: &'static str,
        anchor_event_id: &'static str,
    },
    ActionEvent {
        action_id: &'static str,
        event_id: &'static str,
    },
}

impl ExpectedLocation {
    fn description(self) -> &'static str {
        match self {
            Self::Empty => "empty causal location",
            Self::ActionPlan { .. } => "action_id and plan_hash causal location",
            Self::Event(_) => "event_id causal location",
            Self::Anchor { .. } => "event_id and anchor_event_id causal location",
            Self::ActionEvent { .. } => "action_id and event_id causal location",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_replay_operator_diagnostics, run_replay_operator_diagnostics,
        verify_strict_bundle_hash_negative_control, ReplayOperatorDiagnosticsError,
    };

    #[test]
    fn replay_operator_diagnostics_summary_counts() -> Result<(), ReplayOperatorDiagnosticsError> {
        let summary = run_replay_operator_diagnostics()?;

        assert_eq!(summary.compiled_bundles, 3);
        assert_eq!(summary.accepted_cases, 3);
        assert_eq!(summary.rejected_cases, 5);
        assert_eq!(summary.located_rejections, 4);
        assert_eq!(summary.json_reports, 8);
        assert_eq!(summary.human_reports, 8);
        assert_eq!(summary.strict_negative_controls, 1);
        Ok(())
    }

    #[test]
    fn diagnostic_cases_are_independently_observable() -> Result<(), ReplayOperatorDiagnosticsError>
    {
        let reports = collect_replay_operator_diagnostics()?;
        let cases = reports
            .iter()
            .map(|report| report.case)
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(reports.len(), 8);
        assert!(cases.contains("release_promote_success"));
        assert!(cases.contains("read_only_sidecar_success"));
        assert!(cases.contains("multi_action_reference"));
        assert!(cases.contains("observed_without_execution"));
        assert!(cases.contains("projection_without_anchor"));
        assert!(cases.contains("projection_anchor_wrong_scope"));
        assert!(cases.contains("event_after_closed"));
        assert!(cases.contains("bundle_hash_mismatch"));
        Ok(())
    }

    #[test]
    fn strict_bundle_hash_negative_control_is_observable(
    ) -> Result<(), ReplayOperatorDiagnosticsError> {
        assert_eq!(verify_strict_bundle_hash_negative_control()?, 1);
        Ok(())
    }
}
