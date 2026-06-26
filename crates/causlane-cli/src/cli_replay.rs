//! `replay verify` / `replay verify --explain` command handlers.
//!
//! `--explain` (M04.4) runs bundle-bound replay and returns a structured
//! [`causlane_replay::ReplayExplain`] — the exact violated invariant, stable
//! error code and causal location — instead of a pass/fail string, so a devtool
//! or CI gate can point at the precise failing step.

use causlane_replay::{EventKindDto, ReplayTrace};

use crate::{read_bundle, read_file, CliError, RunOutput};

pub(crate) fn verify_trace(path: &str) -> Result<String, CliError> {
    let json = read_file(path)?;
    let trace = ReplayTrace::from_json_str(&json)?;
    trace.verify()?;
    // Bundle-less replay runs STRUCTURAL invariants only (I-001..I-003, I-006..I-008):
    // it cannot check the predicate/barrier/witness/capability/authz obligations that
    // require the compiled bundle. Do not print a generic "verified" — label the
    // coverage as structural-only so an execution-bearing trace is not mistaken for a
    // fully replay-verified one (M3).
    let execution_bearing = trace.events.iter().any(|event| {
        matches!(
            event.kind,
            EventKindDto::ExecutionBarrierLogged | EventKindDto::ExecutionStarted
        )
    });
    let caveat = if execution_bearing {
        "; execution-bearing trace — run with --bundle for full replay verification \
         (predicate/barrier/witness/capability/authz NOT checked)"
    } else {
        "; no --bundle supplied — bundle-bound obligations NOT checked"
    };
    Ok(format!(
        "ok: trace for {} passed structural checks ({} events){caveat}",
        trace.action_id,
        trace.events.len()
    ))
}

pub(crate) fn verify_trace_with_bundle(
    bundle_path: &str,
    trace_path: &str,
    require_bundle_hash: bool,
    kernel_secret: Option<&str>,
) -> Result<String, CliError> {
    let bundle = read_bundle(bundle_path)?;
    let json = read_file(trace_path)?;
    let trace = ReplayTrace::from_json_str(&json)?;
    // A kernel secret upgrades verification to attested: every capability must carry
    // a valid keyed attestation (P1-006). Otherwise the structural/strict path runs.
    let mode = if let Some(secret) = kernel_secret {
        trace.verify_with_bundle_attested(&bundle, secret.as_bytes())?;
        "attested"
    } else if require_bundle_hash {
        trace.verify_with_bundle_strict(&bundle)?;
        "strict"
    } else {
        trace.verify_with_bundle(&bundle)?;
        "lenient"
    };
    Ok(format!(
        "ok: trace for {} verified with bundle {} [{mode}] ({} events)",
        trace.action_id,
        bundle.bundle_hash.0,
        trace.events.len()
    ))
}

/// `replay verify --explain [--json]`: bundle-bound replay returning a structured
/// explanation. Process success mirrors trace acceptance, so a rejected trace
/// still prints its diagnosis but signals failure.
pub(crate) fn explain_trace_with_bundle(
    bundle_path: &str,
    trace_path: &str,
    json: bool,
) -> Result<RunOutput, CliError> {
    let bundle = read_bundle(bundle_path)?;
    let trace_json = read_file(trace_path)?;
    let trace = ReplayTrace::from_json_str(&trace_json)?;
    let explain = trace.verify_explain(&bundle);
    let text = if json {
        explain.to_json_pretty()?
    } else {
        explain.to_human()
    };
    Ok(RunOutput {
        text,
        success: explain.accepted,
    })
}
