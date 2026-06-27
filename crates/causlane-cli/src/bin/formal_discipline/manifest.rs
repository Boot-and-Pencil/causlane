//! Obligation manifest validation.

use causlane_contracts::{is_known_invariant_id, KNOWN_INVARIANT_RANGE};
use noyalib::compat::serde_yaml::{self, Mapping, Value};
use std::collections::BTreeSet;

struct PlannedPInvariantHook {
    check_id: &'static str,
    invariant_id: &'static str,
    negative_control: &'static str,
}

const PLANNED_P_INVARIANT_HOOKS: &[PlannedPInvariantHook] = &[
    PlannedPInvariantHook {
        check_id: "NoDuplicateHardExecutionForSameIdempotencyKey",
        invariant_id: "I-012",
        negative_control:
            "contracts/scenarios/p_controls/retry_duplicate_execution_control.scenario.yaml",
    },
    PlannedPInvariantHook {
        check_id: "AuthzRevocationBeforeBarrierBlocksExecution",
        invariant_id: "I-014",
        negative_control:
            "contracts/scenarios/p_controls/authz_revoked_old_allow_control.scenario.yaml",
    },
    PlannedPInvariantHook {
        check_id: "NoStaleConstraintEpochAdmission",
        invariant_id: "I-018",
        negative_control:
            "contracts/scenarios/p_controls/constraint_epoch_regression_control.scenario.yaml",
    },
];

pub(crate) fn validate_manifest(path: &str) -> Result<(), Vec<String>> {
    let content = std::fs::read_to_string(path)
        .map_err(|err| vec![format!("manifest {path} cannot be read: {err}")])?;
    validate_manifest_str(path, &content)
}

fn validate_manifest_str(label: &str, content: &str) -> Result<(), Vec<String>> {
    let root: Value = serde_yaml::from_str(content)
        .map_err(|err| vec![format!("manifest {label} YAML parse failed: {err}")])?;
    let mut errors = Vec::new();
    let Some(root_map) = root.as_mapping() else {
        return Err(vec![format!("manifest {label} root must be object")]);
    };

    if number(root_map, "schema_version") != Some(1) {
        errors.push("manifest schema_version must be 1".to_owned());
    }
    if string(root_map, "authority").is_none_or(|value| value.trim().is_empty()) {
        errors.push("manifest authority must be nonempty".to_owned());
    }

    let models = validate_models(sequence(root_map, "models"), &mut errors);
    let protocols = validate_protocols(sequence(root_map, "protocols"), &mut errors);
    for obligation in sequence(root_map, "obligations") {
        validate_obligation(obligation, &models, &protocols, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_models(models: Vec<&Value>, errors: &mut Vec<String>) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for value in models {
        let Some(map) = value.as_mapping() else {
            errors.push("model entry must be object".to_owned());
            continue;
        };
        let id = string(map, "id").unwrap_or("<missing>");
        if !prefixed_three_digits(id, "FM-") {
            errors.push(format!("invalid model id {id}"));
        }
        if string(map, "title").is_none_or(|title| title.trim().is_empty()) {
            errors.push(format!("model {id} title must be nonempty"));
        }
        ids.insert(id.to_owned());
    }
    ids
}

fn validate_protocols(protocols: Vec<&Value>, errors: &mut Vec<String>) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for value in protocols {
        let Some(map) = value.as_mapping() else {
            errors.push("protocol entry must be object".to_owned());
            continue;
        };
        let id = string(map, "id").unwrap_or("<missing>");
        if !prefixed_three_digits(id, "PR-") {
            errors.push(format!("invalid protocol id {id}"));
        }
        if string(map, "title").is_none_or(|title| title.trim().is_empty()) {
            errors.push(format!("protocol {id} title must be nonempty"));
        }
        ids.insert(id.to_owned());
    }
    ids
}

fn validate_obligation(
    value: &Value,
    models: &BTreeSet<String>,
    protocols: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let Some(map) = value.as_mapping() else {
        errors.push("obligation entry must be object".to_owned());
        return;
    };
    let id = string(map, "id").unwrap_or("<missing>");
    let model_id = string(map, "model_id").unwrap_or("<missing>");
    let protocol_id = string(map, "protocol_id").unwrap_or("<missing>");
    let invariant = string(map, "invariant_id").unwrap_or("<missing>");
    if !id.starts_with("OBL-") {
        errors.push(format!("obligation {id} must start with OBL-"));
    }
    if !models.contains(model_id) {
        errors.push(format!(
            "obligation {id} references unknown model {model_id}"
        ));
    }
    if !protocols.contains(protocol_id) {
        errors.push(format!(
            "obligation {id} references unknown protocol {protocol_id}"
        ));
    }
    if !is_known_invariant_id(invariant) {
        errors.push(format!(
            "obligation {id} has invalid invariant {invariant}; expected {KNOWN_INVARIANT_RANGE}"
        ));
    }
    if string(map, "statement").is_none_or(|statement| statement.trim().is_empty()) {
        errors.push(format!("obligation {id} statement is empty"));
    }
    let Some(lanes) = mapping(map, "lanes") else {
        errors.push(format!("obligation {id} missing lanes"));
        return;
    };
    for lane_name in ["replay", "alloy", "p", "kani", "verus", "lean4"] {
        validate_lane(id, lane_name, mapping(lanes, lane_name), errors);
    }
    validate_planned_p_hook_alignment(
        id,
        invariant,
        mapping(lanes, "p"),
        &sequence(map, "negative_controls"),
        errors,
    );
}

fn validate_lane(
    obligation_id: &str,
    lane_name: &str,
    lane: Option<&Mapping>,
    errors: &mut Vec<String>,
) {
    let Some(lane) = lane else {
        errors.push(format!(
            "obligation {obligation_id} missing lane {lane_name}"
        ));
        return;
    };
    match string(lane, "status").unwrap_or("<missing>") {
        "required" | "proof_profile_required" => {
            if !nonempty_check_ids(lane) {
                errors.push(format!(
                    "obligation {obligation_id} lane {lane_name} requires nonempty check_ids"
                ));
            }
        }
        "not_applicable" => {
            if string(lane, "reason").is_none_or(|reason| reason.trim().is_empty()) {
                errors.push(format!(
                    "obligation {obligation_id} lane {lane_name} is not_applicable without reason"
                ));
            }
        }
        "planned" | "non_blocking" | "optional" => {}
        other => errors.push(format!(
            "obligation {obligation_id} lane {lane_name} has unknown status {other}"
        )),
    }
}

fn nonempty_check_ids(map: &Mapping) -> bool {
    sequence(map, "check_ids")
        .iter()
        .any(|value| value.as_str().is_some_and(|id| !id.trim().is_empty()))
}

fn validate_planned_p_hook_alignment(
    obligation_id: &str,
    invariant: &str,
    p_lane: Option<&Mapping>,
    negative_controls: &[&Value],
    errors: &mut Vec<String>,
) {
    for hook in PLANNED_P_INVARIANT_HOOKS {
        let has_check = p_lane.is_some_and(|lane| {
            sequence(lane, "check_ids")
                .iter()
                .any(|value| value.as_str() == Some(hook.check_id))
        });
        let has_control = contains_string(negative_controls, hook.negative_control);

        if has_check && invariant != hook.invariant_id {
            errors.push(format!(
                "obligation {obligation_id} p check {} must be attached to {}, found {invariant}",
                hook.check_id, hook.invariant_id
            ));
        }
        if has_control && invariant != hook.invariant_id {
            errors.push(format!(
                "obligation {obligation_id} negative control {} must be attached to {}, found {invariant}",
                hook.negative_control, hook.invariant_id
            ));
        }
        if has_check && !has_control {
            errors.push(format!(
                "obligation {obligation_id} p check {} requires paired negative control {}",
                hook.check_id, hook.negative_control
            ));
        }
        if has_control && !has_check {
            errors.push(format!(
                "obligation {obligation_id} negative control {} requires paired p check {}",
                hook.negative_control, hook.check_id
            ));
        }
    }
}

fn contains_string(values: &[&Value], needle: &str) -> bool {
    values.iter().any(|value| value.as_str() == Some(needle))
}

fn mapping<'a>(map: &'a Mapping, key: &str) -> Option<&'a Mapping> {
    map.get(key).and_then(Value::as_mapping)
}

fn sequence<'a>(map: &'a Mapping, key: &str) -> Vec<&'a Value> {
    map.get(key)
        .and_then(Value::as_sequence)
        .map_or_else(Vec::new, |values| values.iter().collect())
}

fn string<'a>(map: &'a Mapping, key: &str) -> Option<&'a str> {
    map.get(key).and_then(Value::as_str)
}

fn number(map: &Mapping, key: &str) -> Option<i64> {
    map.get(key).and_then(Value::as_i64)
}

fn prefixed_three_digits(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|tail| tail.len() == 3 && tail.chars().all(|ch| ch.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formal_discipline_current_manifest_is_valid() {
        let path = format!(
            "{}/../../{}",
            env!("CARGO_MANIFEST_DIR"),
            crate::formal_discipline::args::DEFAULT_MANIFEST
        );
        assert!(validate_manifest(&path).is_ok());
    }

    #[test]
    fn formal_discipline_manifest_requires_check_ids() {
        let errors = validate_manifest_str("test", &base_manifest("{status: required}"));
        assert_contains(errors, "check_ids");
    }

    #[test]
    fn formal_discipline_manifest_requires_not_applicable_reason() {
        let errors = validate_manifest_str("test", &base_manifest("{status: not_applicable}"));
        assert_contains(errors, "without reason");
    }

    #[test]
    fn formal_discipline_manifest_accepts_planned_invariant_reservations() {
        let manifest = base_manifest("{status: planned}").replace("I-001", "I-011");

        assert!(validate_manifest_str("test", &manifest).is_ok());
    }

    #[test]
    fn formal_discipline_manifest_rejects_unknown_invariant_ids() {
        let manifest = base_manifest("{status: planned}").replace("I-001", "I-021");
        let errors = validate_manifest_str("test", &manifest);

        assert_contains(errors, "I-001..I-020");
    }

    #[test]
    fn formal_discipline_manifest_rejects_planned_p_hook_misalignment() {
        let manifest = r"
schema_version: 1
authority: test
models:
  - id: FM-001
    title: Model
protocols:
  - id: PR-001
    title: Protocol
obligations:
  - id: OBL-TEST
    model_id: FM-001
    protocol_id: PR-001
    invariant_id: I-011
    statement: test obligation
    lanes:
      replay: {status: planned}
      alloy: {status: planned}
      p: {status: planned, check_ids: [NoDuplicateHardExecutionForSameIdempotencyKey]}
      kani: {status: planned}
      verus: {status: planned}
      lean4: {status: planned}
    negative_controls:
      - contracts/scenarios/p_controls/retry_duplicate_execution_control.scenario.yaml
";
        let errors = validate_manifest_str("test", manifest);

        assert_contains(errors, "must be attached to I-012");
    }

    fn assert_contains(result: Result<(), Vec<String>>, needle: &str) {
        let errors = result.err().unwrap_or_default();
        assert!(
            errors.iter().any(|error| error.contains(needle)),
            "expected a validation error containing {needle:?}",
        );
    }

    fn base_manifest(lane: &str) -> String {
        format!(
            r"
schema_version: 1
authority: test
models:
  - id: FM-001
    title: Model
protocols:
  - id: PR-001
    title: Protocol
obligations:
  - id: OBL-TEST
    model_id: FM-001
    protocol_id: PR-001
    invariant_id: I-001
    statement: test obligation
    lanes:
      replay: {lane}
      alloy: {{status: not_applicable, reason: no model}}
      p: {{status: not_applicable, reason: no model}}
      kani: {{status: not_applicable, reason: no model}}
      verus: {{status: not_applicable, reason: no model}}
      lean4: {{status: not_applicable, reason: no model}}
"
        )
    }
}
