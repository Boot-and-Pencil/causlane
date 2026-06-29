#![forbid(unsafe_code)]
#![deny(warnings)]

use std::fmt;

use causlane_contracts::{CompiledDispatchBundle, ContractError, RegistryManifest};
use causlane_replay::{CausalLocation, ReplayError, ReplayExplain, ReplayScenario};

const REGISTRY_YAML: &str =
    include_str!("../../../contracts/examples/release_promote.registry.yaml");
const SUCCESS_SCENARIO_YAML: &str =
    include_str!("../../../contracts/scenarios/release_promote_success.scenario.yaml");
const EXECUTION_WITHOUT_BARRIER_YAML: &str =
    include_str!("../../../contracts/scenarios/execution_without_barrier_invalid.scenario.yaml");
const MISSING_WITNESS_YAML: &str =
    include_str!("../../../contracts/scenarios/missing_witness_invalid.scenario.yaml");
const CONFLICTING_LEASES_YAML: &str =
    include_str!("../../../contracts/scenarios/conflicting_leases_invalid.scenario.yaml");
const WRONG_BUNDLE_HASH: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// Summary returned by the replay diagnostics example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayDiagnosticsSummary {
    /// Accepted traces checked through `verify_explain`.
    pub accepted_cases: usize,
    /// Rejected traces checked through `verify_explain`.
    pub rejected_cases: usize,
    /// Rejections that carried a structured causal location.
    pub located_rejections: usize,
    /// Reports rendered to JSON.
    pub json_reports: usize,
    /// Reports rendered to human-readable text.
    pub human_reports: usize,
}

/// Per-case diagnostic captured by the example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayDiagnosticReport {
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

/// Error type for the replay diagnostics example.
#[derive(Debug)]
pub enum ReplayDiagnosticsError {
    /// Registry or bundle compilation failed.
    Contract(ContractError),
    /// Scenario loading or explain rendering failed.
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
}

impl fmt::Display for ReplayDiagnosticsError {
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
        }
    }
}

impl std::error::Error for ReplayDiagnosticsError {}

impl From<ContractError> for ReplayDiagnosticsError {
    fn from(error: ContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<ReplayError> for ReplayDiagnosticsError {
    fn from(error: ReplayError) -> Self {
        Self::Replay(error)
    }
}

/// Run the replay diagnostics example.
///
/// # Errors
/// Returns an error if bundle compilation, scenario parsing or any diagnostic
/// assertion diverges from the expected public explain behavior.
#[must_use = "the runnable example result must be checked"]
pub fn run_replay_diagnostics() -> Result<ReplayDiagnosticsSummary, ReplayDiagnosticsError> {
    let reports = collect_replay_diagnostics()?;
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

    Ok(ReplayDiagnosticsSummary {
        accepted_cases,
        rejected_cases,
        located_rejections,
        json_reports,
        human_reports,
    })
}

/// Collect per-case replay diagnostics through public APIs.
///
/// # Errors
/// Returns an error if a scenario fails to parse, report rendering fails or the
/// explain output does not match the documented stable diagnostics.
#[must_use = "the diagnostic reports must be inspected"]
pub fn collect_replay_diagnostics() -> Result<Vec<ReplayDiagnosticReport>, ReplayDiagnosticsError> {
    let bundle = compile_bundle()?;
    let mut reports = Vec::new();
    for case in scenario_cases() {
        let scenario = ReplayScenario::from_yaml_str(case.scenario_yaml)?;
        let explain = scenario.to_trace().verify_explain(&bundle);
        reports.push(require_case_explain(case, &explain)?);
    }
    reports.push(verify_bundle_hash_mismatch(&bundle)?);
    Ok(reports)
}

fn compile_bundle() -> Result<CompiledDispatchBundle, ReplayDiagnosticsError> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY_YAML)?;
    Ok(CompiledDispatchBundle::compile(&manifest)?)
}

fn scenario_cases() -> [DiagnosticCase; 4] {
    [
        DiagnosticCase {
            label: "release_promote_success",
            scenario_yaml: SUCCESS_SCENARIO_YAML,
            accepted: true,
            invariant: None,
            error_code: None,
            location: ExpectedLocation::Empty,
        },
        DiagnosticCase {
            label: "execution_without_barrier",
            scenario_yaml: EXECUTION_WITHOUT_BARRIER_YAML,
            accepted: false,
            invariant: Some("I-001"),
            error_code: Some("ExecutionWithoutBarrier"),
            location: ExpectedLocation::Action("act_promote_123"),
        },
        DiagnosticCase {
            label: "missing_witness",
            scenario_yaml: MISSING_WITNESS_YAML,
            accepted: false,
            invariant: Some("I-009"),
            error_code: Some("RequiredWitnessMissing"),
            location: ExpectedLocation::Requirement("readiness_before_promotion"),
        },
        DiagnosticCase {
            label: "conflicting_leases",
            scenario_yaml: CONFLICTING_LEASES_YAML,
            accepted: false,
            invariant: Some("I-006"),
            error_code: Some("ConflictingLeases"),
            location: ExpectedLocation::Scope("environment:staging"),
        },
    ]
}

fn verify_bundle_hash_mismatch(
    bundle: &CompiledDispatchBundle,
) -> Result<ReplayDiagnosticReport, ReplayDiagnosticsError> {
    let scenario = ReplayScenario::from_yaml_str(SUCCESS_SCENARIO_YAML)?;
    let mut trace = scenario.to_trace();
    trace.bundle_hash = Some(WRONG_BUNDLE_HASH.to_owned());
    let explain = trace.verify_explain(bundle);
    require_case_explain(
        DiagnosticCase {
            label: "bundle_hash_mismatch",
            scenario_yaml: SUCCESS_SCENARIO_YAML,
            accepted: false,
            invariant: None,
            error_code: Some("BundleHashMismatch"),
            location: ExpectedLocation::Empty,
        },
        &explain,
    )
}

fn require_case_explain(
    case: DiagnosticCase,
    explain: &ReplayExplain,
) -> Result<ReplayDiagnosticReport, ReplayDiagnosticsError> {
    if case.accepted && !explain.accepted {
        return Err(ReplayDiagnosticsError::UnexpectedRejected(case.label));
    }
    if !case.accepted && explain.accepted {
        return Err(ReplayDiagnosticsError::UnexpectedAccepted(case.label));
    }
    if explain.invariant.as_deref() != case.invariant {
        return Err(ReplayDiagnosticsError::InvariantMismatch {
            case: case.label,
            expected: case.invariant,
            actual: explain.invariant.clone(),
        });
    }
    if explain.error_code.as_deref() != case.error_code {
        return Err(ReplayDiagnosticsError::ErrorCodeMismatch {
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
) -> Result<(), ReplayDiagnosticsError> {
    let matches = match expected {
        ExpectedLocation::Empty => actual.is_empty(),
        ExpectedLocation::Action(action_id) => actual.action_id.as_deref() == Some(*action_id),
        ExpectedLocation::Requirement(requirement_id) => {
            actual.requirement_id.as_deref() == Some(*requirement_id)
        }
        ExpectedLocation::Scope(scope) => actual.scope.as_deref() == Some(*scope),
    };
    if matches {
        return Ok(());
    }
    Err(ReplayDiagnosticsError::LocationMismatch {
        case,
        expected: expected.description(),
        actual: format!("{actual:?}"),
    })
}

fn render_report(
    case: &'static str,
    explain: &ReplayExplain,
) -> Result<ReplayDiagnosticReport, ReplayDiagnosticsError> {
    let json = explain.to_json_pretty()?;
    if json.is_empty() {
        return Err(ReplayDiagnosticsError::EmptyRenderedReport {
            case,
            format: "json",
        });
    }
    let human = explain.to_human();
    if human.is_empty() {
        return Err(ReplayDiagnosticsError::EmptyRenderedReport {
            case,
            format: "human",
        });
    }

    Ok(ReplayDiagnosticReport {
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
    Action(&'static str),
    Requirement(&'static str),
    Scope(&'static str),
}

impl ExpectedLocation {
    fn description(self) -> &'static str {
        match self {
            Self::Empty => "empty causal location",
            Self::Action(_) => "action_id causal location",
            Self::Requirement(_) => "requirement_id causal location",
            Self::Scope(_) => "scope causal location",
        }
    }
}
