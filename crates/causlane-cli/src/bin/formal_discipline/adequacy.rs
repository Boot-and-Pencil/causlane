//! Strict check-id adequacy validation.

use crate::formal_discipline::args::DiffSource;
use crate::formal_discipline::Findings;
use serde_json::Value as JsonValue;
use serde_yaml::{Mapping, Value as YamlValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

const COVERAGE_REPORT: &str = "target/causlane/formal-coverage-report.json";
const COVERAGE_DOC: &str = "docs/invariants/coverage-matrix.json";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CheckRef {
    lane: String,
    invariant_id: String,
    check_id: String,
}

#[derive(Clone, Debug)]
struct RequiredCheck {
    key: CheckRef,
    obligation_id: String,
    negative_controls: Vec<String>,
}

#[derive(Debug)]
struct ArtifactEvidence {
    artifact_path: String,
    codegen_receipt: String,
    tool_run_receipt: String,
}

#[derive(Debug)]
enum AdequacyError {
    Message(String),
}

impl fmt::Display for AdequacyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => write!(f, "{message}"),
        }
    }
}

pub(crate) fn check(
    manifest_path: &str,
    diff_source: &DiffSource,
    findings: &mut Findings,
) -> String {
    if !Path::new(COVERAGE_REPORT).exists() {
        let message = format!("{COVERAGE_REPORT} is absent; adequacy check skipped");
        if diff_source.is_no_diff() {
            findings.warnings.push(message);
            return "skipped".to_owned();
        }
        findings.errors.push(message);
        return "fail".to_owned();
    }

    let error_start = findings.errors.len();
    match check_inner(manifest_path) {
        Ok(()) => {}
        Err(errors) => findings
            .errors
            .extend(errors.into_iter().map(|error| format!("adequacy: {error}"))),
    }
    if findings.errors.len() == error_start {
        "pass".to_owned()
    } else {
        "fail".to_owned()
    }
}

fn check_inner(manifest_path: &str) -> Result<(), Vec<String>> {
    let manifest = read_yaml(manifest_path).map_err(|error| single_error(&error))?;
    let docs = read_json(COVERAGE_DOC).map_err(|error| single_error(&error))?;
    let report = read_json(COVERAGE_REPORT).map_err(|error| single_error(&error))?;

    let required = required_checks(&manifest)?;
    let required_set: BTreeSet<CheckRef> = required.iter().map(|check| check.key.clone()).collect();
    let docs_counted = counted_from_docs(&docs)?;
    let report_counted = counted_from_report(&report)?;
    let replay_statuses = replay_statuses(&report);
    let artifacts = artifact_evidence(&report);

    let mut errors = Vec::new();
    validate_required_sets(&required, &docs_counted, &report_counted, &mut errors);
    validate_no_unrequired("docs", &docs_counted, &required_set, &mut errors);
    validate_no_unrequired(
        "coverage report",
        &report_counted,
        &required_set,
        &mut errors,
    );
    for required_check in &required {
        if required_check.key.lane == "replay" {
            validate_replay(required_check, &replay_statuses, &mut errors);
        } else {
            validate_artifact(required_check, &artifacts, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_required_sets(
    required: &[RequiredCheck],
    docs_counted: &BTreeSet<CheckRef>,
    report_counted: &BTreeSet<CheckRef>,
    errors: &mut Vec<String>,
) {
    for check in required {
        if !docs_counted.contains(&check.key) {
            errors.push(format!(
                "{} missing from docs coverage",
                format_check(&check.key)
            ));
        }
        if !report_counted.contains(&check.key) {
            errors.push(format!(
                "{} missing from coverage report",
                format_check(&check.key)
            ));
        }
    }
}

fn validate_no_unrequired(
    label: &str,
    counted: &BTreeSet<CheckRef>,
    required: &BTreeSet<CheckRef>,
    errors: &mut Vec<String>,
) {
    for check in counted {
        if !required.contains(check) {
            errors.push(format!(
                "{label} counts non-required check {}",
                format_check(check)
            ));
        }
    }
}

fn validate_replay(
    required: &RequiredCheck,
    replay_statuses: &BTreeMap<String, String>,
    errors: &mut Vec<String>,
) {
    let Some(control_path) = required
        .negative_controls
        .iter()
        .find(|path| scenario_id(path).as_deref() == Some(required.key.check_id.as_str()))
    else {
        errors.push(format!(
            "{} is not listed in obligation {} negative_controls",
            format_check(&required.key),
            required.obligation_id
        ));
        return;
    };
    if !Path::new(control_path).exists() {
        errors.push(format!(
            "{} negative control path is missing: {control_path}",
            format_check(&required.key)
        ));
    }
    if replay_statuses
        .get(&required.key.check_id)
        .map(String::as_str)
        != Some("refuted_by_replay")
    {
        errors.push(format!(
            "{} is not refuted_by_replay in coverage report",
            format_check(&required.key)
        ));
    }
}

fn validate_artifact(
    required: &RequiredCheck,
    artifacts: &BTreeMap<String, ArtifactEvidence>,
    errors: &mut Vec<String>,
) {
    let Some(evidence) = artifacts.get(&required.key.lane) else {
        errors.push(format!(
            "{} has no artifact entry in coverage report",
            format_check(&required.key)
        ));
        return;
    };
    if !file_contains(&evidence.artifact_path, &required.key.check_id) {
        errors.push(format!(
            "{} missing from generated artifact {}",
            format_check(&required.key),
            evidence.artifact_path
        ));
    }
    if !receipt_has_obligation(&evidence.codegen_receipt, &required.key) {
        errors.push(format!(
            "{} missing from codegen receipt {}",
            format_check(&required.key),
            evidence.codegen_receipt
        ));
    }
    if !receipt_has_obligation(&evidence.tool_run_receipt, &required.key) {
        errors.push(format!(
            "{} missing from tool-run receipt {}",
            format_check(&required.key),
            evidence.tool_run_receipt
        ));
    }
}

fn required_checks(value: &YamlValue) -> Result<Vec<RequiredCheck>, Vec<String>> {
    let Some(root) = value.as_mapping() else {
        return Err(vec!["manifest root must be object".to_owned()]);
    };
    let mut checks = Vec::new();
    for obligation in yaml_sequence(root, "obligations") {
        let Some(map) = obligation.as_mapping() else {
            continue;
        };
        let obligation_id = yaml_string(map, "id").unwrap_or("<missing>").to_owned();
        let invariant_id = yaml_string(map, "invariant_id")
            .unwrap_or("<missing>")
            .to_owned();
        let negative_controls = yaml_sequence(map, "negative_controls")
            .into_iter()
            .filter_map(YamlValue::as_str)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let Some(lanes) = yaml_mapping(map, "lanes") else {
            continue;
        };
        for (lane_value, lane_body) in lanes {
            let Some(lane) = lane_value.as_str() else {
                continue;
            };
            let Some(lane_map) = lane_body.as_mapping() else {
                continue;
            };
            let status = yaml_string(lane_map, "status");
            if !matches!(status, Some("required" | "proof_profile_required")) {
                continue;
            }
            for check_id in yaml_sequence(lane_map, "check_ids") {
                let Some(check_id) = check_id.as_str() else {
                    continue;
                };
                if check_id.trim().is_empty() {
                    continue;
                }
                checks.push(RequiredCheck {
                    key: CheckRef {
                        lane: lane.to_owned(),
                        invariant_id: invariant_id.clone(),
                        check_id: check_id.to_owned(),
                    },
                    obligation_id: obligation_id.clone(),
                    negative_controls: negative_controls.clone(),
                });
            }
        }
    }
    Ok(checks)
}

fn counted_from_docs(value: &JsonValue) -> Result<BTreeSet<CheckRef>, Vec<String>> {
    counted_from_entries(value, "invariants", "id")
}

fn counted_from_report(value: &JsonValue) -> Result<BTreeSet<CheckRef>, Vec<String>> {
    counted_from_entries(value, "invariant_coverage", "invariant_id")
}

fn counted_from_entries(
    value: &JsonValue,
    entry_key: &str,
    id_key: &str,
) -> Result<BTreeSet<CheckRef>, Vec<String>> {
    let Some(entries) = value.get(entry_key).and_then(JsonValue::as_array) else {
        return Err(vec![format!("{entry_key} must be array")]);
    };
    let mut counted = BTreeSet::new();
    let mut errors = Vec::new();
    for entry in entries {
        let Some(invariant_id) = entry.get(id_key).and_then(JsonValue::as_str) else {
            errors.push(format!("{entry_key} entry missing {id_key}"));
            continue;
        };
        let Some(check_ids) = entry.get("check_ids").and_then(JsonValue::as_array) else {
            errors.push(format!("{entry_key} {invariant_id} missing check_ids"));
            continue;
        };
        for check in check_ids {
            let Some(check) = check.as_str() else {
                errors.push(format!(
                    "{entry_key} {invariant_id} has non-string check_id"
                ));
                continue;
            };
            let Some((lane, check_id)) = check.split_once(':') else {
                errors.push(format!(
                    "{entry_key} {invariant_id} malformed check_id {check}"
                ));
                continue;
            };
            counted.insert(CheckRef {
                lane: lane.to_owned(),
                invariant_id: invariant_id.to_owned(),
                check_id: check_id.to_owned(),
            });
        }
    }
    if errors.is_empty() {
        Ok(counted)
    } else {
        Err(errors)
    }
}

fn replay_statuses(report: &JsonValue) -> BTreeMap<String, String> {
    let mut statuses = BTreeMap::new();
    if let Some(controls) = report
        .get("negative_controls")
        .and_then(JsonValue::as_array)
    {
        for control in controls {
            if let (Some(scenario), Some(status)) = (
                control.get("scenario").and_then(JsonValue::as_str),
                control.get("status").and_then(JsonValue::as_str),
            ) {
                statuses.insert(scenario.to_owned(), status.to_owned());
            }
        }
    }
    statuses
}

fn artifact_evidence(report: &JsonValue) -> BTreeMap<String, ArtifactEvidence> {
    let mut artifacts = BTreeMap::new();
    if let Some(entries) = report.get("artifacts").and_then(JsonValue::as_array) {
        for entry in entries {
            let target = entry.get("target").and_then(JsonValue::as_str);
            let path = entry.get("path").and_then(JsonValue::as_str);
            let codegen = entry.get("codegen_receipt").and_then(JsonValue::as_str);
            let tool_run = entry.get("tool_run_receipt").and_then(JsonValue::as_str);
            if let (Some(target), Some(path), Some(codegen), Some(tool_run)) =
                (target, path, codegen, tool_run)
            {
                artifacts.insert(
                    target.to_owned(),
                    ArtifactEvidence {
                        artifact_path: path.to_owned(),
                        codegen_receipt: codegen.to_owned(),
                        tool_run_receipt: tool_run.to_owned(),
                    },
                );
            }
        }
    }
    artifacts
}

fn receipt_has_obligation(path: &str, key: &CheckRef) -> bool {
    let Ok(value) = read_json(path) else {
        return false;
    };
    value
        .get("obligations")
        .and_then(JsonValue::as_array)
        .is_some_and(|obligations| {
            obligations.iter().any(|obligation| {
                obligation.get("invariant_id").and_then(JsonValue::as_str)
                    == Some(key.invariant_id.as_str())
                    && obligation.get("check_id").and_then(JsonValue::as_str)
                        == Some(key.check_id.as_str())
            })
        })
}

fn file_contains(path: &str, needle: &str) -> bool {
    std::fs::read_to_string(path).is_ok_and(|content| content.contains(needle))
}

fn scenario_id(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".scenario.yaml"))
        .map(str::to_owned)
}

fn format_check(check: &CheckRef) -> String {
    format!("{}:{}:{}", check.lane, check.invariant_id, check.check_id)
}

fn read_json(path: &str) -> Result<JsonValue, AdequacyError> {
    let content = std::fs::read_to_string(path)
        .map_err(|err| AdequacyError::Message(format!("{path} cannot be read: {err}")))?;
    serde_json::from_str(&content)
        .map_err(|err| AdequacyError::Message(format!("{path} JSON parse failed: {err}")))
}

fn read_yaml(path: &str) -> Result<YamlValue, AdequacyError> {
    let content = std::fs::read_to_string(path)
        .map_err(|err| AdequacyError::Message(format!("{path} cannot be read: {err}")))?;
    serde_yaml::from_str(&content)
        .map_err(|err| AdequacyError::Message(format!("{path} YAML parse failed: {err}")))
}

fn yaml_mapping<'a>(map: &'a Mapping, key: &str) -> Option<&'a Mapping> {
    map.get(YamlValue::String(key.to_owned()))
        .and_then(YamlValue::as_mapping)
}

fn yaml_sequence<'a>(map: &'a Mapping, key: &str) -> Vec<&'a YamlValue> {
    map.get(YamlValue::String(key.to_owned()))
        .and_then(YamlValue::as_sequence)
        .map_or_else(Vec::new, |values| values.iter().collect())
}

fn yaml_string<'a>(map: &'a Mapping, key: &str) -> Option<&'a str> {
    map.get(YamlValue::String(key.to_owned()))
        .and_then(YamlValue::as_str)
}

fn single_error(error: &AdequacyError) -> Vec<String> {
    vec![error.to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formal_discipline_adequacy_collects_required_checks() {
        let manifest = serde_yaml::from_str::<YamlValue>(&manifest_fixture());
        assert!(manifest.is_ok());
        let checks = manifest
            .ok()
            .and_then(|value| required_checks(&value).ok())
            .unwrap_or_default();
        assert!(checks.iter().any(|check| check.key.lane == "replay"
            && check.key.check_id == "projection_without_anchor_invalid"));
        assert!(checks
            .iter()
            .any(|check| check.key.lane == "p" && check.key.check_id == "AnchorFactGrounded"));
        assert!(!checks
            .iter()
            .any(|check| check.key.check_id == "planned_check"));
    }

    #[test]
    fn formal_discipline_adequacy_missing_required_fails() {
        let required = vec![required("p", "I-003", "AnchorFactGrounded")];
        let docs = BTreeSet::new();
        let report = BTreeSet::new();
        let mut errors = Vec::new();
        validate_required_sets(&required, &docs, &report, &mut errors);
        assert!(errors.iter().any(|error| error.contains("docs coverage")));
        assert!(errors.iter().any(|error| error.contains("coverage report")));
    }

    #[test]
    fn formal_discipline_adequacy_unrequired_counted_fails() {
        let counted = BTreeSet::from([check_ref("p", "I-003", "PlannedOnly")]);
        let required = BTreeSet::new();
        let mut errors = Vec::new();
        validate_no_unrequired("docs", &counted, &required, &mut errors);
        assert!(errors
            .iter()
            .any(|error| error.contains("counts non-required check")));
    }

    #[test]
    fn formal_discipline_adequacy_receipt_obligation_matches() {
        let path = temp_file(
            "receipt",
            r#"{"obligations":[{"invariant_id":"I-003","check_id":"AnchorFactGrounded"}]}"#,
        );
        let key = check_ref("p", "I-003", "AnchorFactGrounded");
        assert!(receipt_has_obligation(
            path.to_string_lossy().as_ref(),
            &key
        ));
        let _ignored = std::fs::remove_file(path);
    }

    fn required(lane: &str, invariant_id: &str, check_id: &str) -> RequiredCheck {
        RequiredCheck {
            key: check_ref(lane, invariant_id, check_id),
            obligation_id: "OBL-TEST".to_owned(),
            negative_controls: Vec::new(),
        }
    }

    fn check_ref(lane: &str, invariant_id: &str, check_id: &str) -> CheckRef {
        CheckRef {
            lane: lane.to_owned(),
            invariant_id: invariant_id.to_owned(),
            check_id: check_id.to_owned(),
        }
    }

    fn temp_file(label: &str, content: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "causlane_formal_discipline_adequacy_{label}_{}",
            std::process::id()
        ));
        assert!(std::fs::write(&path, content).is_ok());
        path
    }

    fn manifest_fixture() -> String {
        r"
schema_version: 1
authority: test
obligations:
  - id: OBL-TEST
    invariant_id: I-003
    negative_controls:
      - contracts/scenarios/projection_without_anchor_invalid.scenario.yaml
    lanes:
      replay: {status: required, check_ids: [projection_without_anchor_invalid]}
      p: {status: required, check_ids: [AnchorFactGrounded]}
      lean4: {status: planned, check_ids: [planned_check]}
"
        .to_owned()
    }
}
