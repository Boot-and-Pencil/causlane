use causlane_contracts::CompiledDispatchBundle;

use crate::{error::CodegenError, generated_artifact_hash, FormalReceipt, GENERATOR_VERSION};

/// Check a generated artifact against its source bundle and optional receipt.
///
/// # Errors
/// Returns [`CodegenError::Stale`] when hashes do not match, or
/// [`CodegenError::Receipt`] for malformed receipt JSON.
pub fn stale_check(
    bundle: &CompiledDispatchBundle,
    generated_text: &str,
    receipt_json: Option<&str>,
) -> Result<(), CodegenError> {
    stale_check_with_expected(bundle, generated_text, receipt_json, None)
}

/// Check a generated artifact against its source bundle and expected scenario.
///
/// # Errors
/// Returns [`CodegenError::Stale`] when hashes do not match, or
/// [`CodegenError::Receipt`] for malformed receipt JSON.
pub fn stale_check_with_expected(
    bundle: &CompiledDispatchBundle,
    generated_text: &str,
    receipt_json: Option<&str>,
    expected_scenario_hash: Option<&str>,
) -> Result<(), CodegenError> {
    let header_bundle_hash = header_value(generated_text, "source_bundle_hash")?;
    if header_bundle_hash != bundle.bundle_hash.0 {
        return Err(CodegenError::Stale(format!(
            "header source_bundle_hash {} does not match bundle {}",
            header_bundle_hash, bundle.bundle_hash.0
        )));
    }
    let header_scenario_hash = header_scenario_hash(generated_text)?;
    if let Some(expected) = expected_scenario_hash {
        if header_scenario_hash.as_deref() != Some(expected) {
            return Err(CodegenError::Stale(format!(
                "header scenario_hash {header_scenario_hash:?} does not match expected {expected}"
            )));
        }
    }
    let header_formal_ir_hash = optional_header_value(generated_text, "formal_ir_hash")?;
    let header_target = optional_header_value(generated_text, "target")?;
    // Trigger on a generator-version change even without a receipt: an artifact
    // produced by an older generator is stale (FM-005 trigger `generator_version`).
    let header_generator_version = optional_header_value(generated_text, "generator_version")?;
    if let Some(generator) = &header_generator_version {
        if generator != GENERATOR_VERSION {
            return Err(CodegenError::Stale(format!(
                "header generator_version {generator} does not match current {GENERATOR_VERSION}"
            )));
        }
    }
    let header_invariant_ids = optional_header_value(generated_text, "invariant_ids")?;
    // H4: editable header fields cannot prove a generated artifact is fresh — a
    // hand-edited body with an intact header would otherwise pass. Require a
    // receipt; its `generated_artifact_hash` is compared against the artifact
    // below (and the header is part of the hashed text), so any body/header edit
    // is caught. Fail closed when no receipt is supplied.
    if receipt_json.is_none() {
        return Err(CodegenError::Stale(
            "a codegen receipt is required to prove the generated artifact is fresh; \
             editable header fields alone are not sufficient"
                .to_owned(),
        ));
    }
    let actual_artifact_hash = generated_artifact_hash(generated_text);
    if let Some(json) = receipt_json {
        let receipt: FormalReceipt =
            serde_json::from_str(json).map_err(|err| CodegenError::Receipt(err.to_string()))?;
        if receipt.schema_version != 2 {
            return Err(CodegenError::Receipt(format!(
                "receipt schema_version must be 2, got {}",
                receipt.schema_version
            )));
        }
        if receipt.source_bundle_hash != bundle.bundle_hash.0 {
            return Err(CodegenError::Stale(format!(
                "receipt source_bundle_hash {} does not match bundle {}",
                receipt.source_bundle_hash, bundle.bundle_hash.0
            )));
        }
        if receipt.scenario_hash != header_scenario_hash {
            return Err(CodegenError::Stale(format!(
                "receipt scenario_hash {:?} does not match header {:?}",
                receipt.scenario_hash, header_scenario_hash
            )));
        }
        if let Some(header) = header_formal_ir_hash {
            if receipt.formal_ir_hash.as_deref() != Some(header.as_str()) {
                return Err(CodegenError::Stale(format!(
                    "receipt formal_ir_hash {:?} does not match header {header}",
                    receipt.formal_ir_hash
                )));
            }
        }
        if let Some(header) = header_target {
            if receipt.target.as_deref() != Some(header.as_str()) {
                return Err(CodegenError::Stale(format!(
                    "receipt target {:?} does not match header {header}",
                    receipt.target
                )));
            }
        }
        if let Some(header) = &header_generator_version {
            if &receipt.generator_version != header {
                return Err(CodegenError::Stale(format!(
                    "receipt generator_version {} does not match header {header}",
                    receipt.generator_version
                )));
            }
        }
        if let Some(header) = &header_invariant_ids {
            let header_ids = parse_header_invariant_ids(header);
            if receipt.invariant_ids != header_ids {
                return Err(CodegenError::Stale(format!(
                    "receipt invariant_ids {:?} do not match header {header_ids:?}",
                    receipt.invariant_ids
                )));
            }
        }
        if receipt.generated_artifact_hash != actual_artifact_hash {
            return Err(CodegenError::Stale(format!(
                "receipt generated_artifact_hash {} does not match actual {}",
                receipt.generated_artifact_hash, actual_artifact_hash
            )));
        }
    }
    Ok(())
}

fn header_value(text: &str, key: &str) -> Result<String, CodegenError> {
    let slash_prefix = format!("// {key}: ");
    let lean_prefix = format!("-- {key}: ");
    text.lines()
        .find_map(|line| {
            line.strip_prefix(&slash_prefix)
                .or_else(|| line.strip_prefix(&lean_prefix))
                .map(ToOwned::to_owned)
        })
        .ok_or_else(|| CodegenError::Header(format!("missing {key} header")))
}

fn optional_header_value(text: &str, key: &str) -> Result<Option<String>, CodegenError> {
    match header_value(text, key) {
        Ok(value) => Ok(Some(value)),
        Err(CodegenError::Header(_msg)) => Ok(None),
        Err(err) => Err(err),
    }
}

fn header_scenario_hash(text: &str) -> Result<Option<String>, CodegenError> {
    let value = header_value(text, "scenario_hash")?;
    if value == "null" {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// Parse the `invariant_ids` header value back into the receipt's list form.
/// The header is `none` (empty) or a comma-joined list (`I-001,I-002`).
fn parse_header_invariant_ids(value: &str) -> Vec<String> {
    if value == "none" {
        return Vec::new();
    }
    value
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
