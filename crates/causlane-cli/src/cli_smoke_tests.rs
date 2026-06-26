use super::{run, CliError};
use std::path::Path;

fn fixture(name: &str) -> String {
    format!(
        "{}/../../contracts/examples/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn scenario_fixture(name: &str) -> String {
    format!(
        "{}/../../contracts/scenarios/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

fn remove_dir_if_exists(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn assert_run_ok(parts: &[&str]) {
    let result = run(&args(parts));
    assert!(result.is_ok(), "{:?}", result.err().map(|e| e.to_string()));
}

fn graph_text() -> &'static str {
    r"
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
"
}

#[test]
fn validates_example_bundle() {
    let path = fixture("release_promote.registry.yaml");
    let result = run(&args(&["causlane", "bundle", "validate", &path]));
    assert!(result.is_ok(), "{:?}", result.err().map(|e| e.to_string()));
}

#[test]
fn verifies_example_trace() {
    let path = fixture("release_promote.trace.json");
    let result = run(&args(&["causlane", "replay", "verify", &path]));
    assert!(result.is_ok(), "{:?}", result.err().map(|e| e.to_string()));
}

#[test]
fn bundle_less_verify_is_labeled_structural_only() {
    // M3: bundle-less `replay verify` must not print a generic "verified" — it must
    // label structural-only coverage and point at --bundle for full replay.
    let path = fixture("release_promote.trace.json");
    let result = run(&args(&["causlane", "replay", "verify", &path]));
    assert!(
        result.is_ok(),
        "{:?}",
        result.as_ref().err().map(ToString::to_string)
    );
    let text = result.map(|output| output.text).unwrap_or_default();
    assert!(
        text.contains("structural"),
        "must label structural-only coverage: {text}"
    );
    assert!(
        text.contains("--bundle"),
        "must point at --bundle for full replay: {text}"
    );
    assert!(
        !text.contains("verified"),
        "must not print a generic 'verified': {text}"
    );
}

#[test]
fn compiles_generates_and_stale_checks() -> std::io::Result<()> {
    let dir = std::env::temp_dir().join(format!("causlane-cli-{}", std::process::id()));
    remove_dir_if_exists(&dir)?;
    std::fs::create_dir_all(&dir)?;

    let registry = fixture("release_promote.registry.yaml");
    let trace = fixture("release_promote.trace.json");
    let scenario = scenario_fixture("release_promote_success.scenario.yaml");
    let bundle = dir.join("release_promote.bundle.json");
    let emitted_trace = dir.join("release_promote.scenario.trace.json");
    let generated = dir.join("release_promote.generated.als");
    let receipt = dir.join("release_promote.receipt.json");
    let bundle_path = bundle.display().to_string();
    let emitted_trace_path = emitted_trace.display().to_string();
    let generated_path = generated.display().to_string();
    let receipt_path = receipt.display().to_string();

    assert_run_ok(&[
        "causlane",
        "bundle",
        "compile",
        "--registry",
        &registry,
        "--out",
        &bundle_path,
    ]);

    assert_run_ok(&[
        "causlane",
        "replay",
        "verify",
        "--bundle",
        &bundle_path,
        "--trace",
        &trace,
    ]);

    assert_run_ok(&[
        "causlane",
        "scenario",
        "emit-trace",
        "--scenario",
        &scenario,
        "--out",
        &emitted_trace_path,
    ]);

    assert_run_ok(&[
        "causlane",
        "replay",
        "verify",
        "--bundle",
        &bundle_path,
        "--trace",
        &emitted_trace_path,
    ]);

    assert_run_ok(&[
        "causlane",
        "formal",
        "generate",
        "alloy",
        "--bundle",
        &bundle_path,
        "--out",
        &generated_path,
        "--scenario",
        &scenario,
        "--receipt",
        &receipt_path,
    ]);

    assert_run_ok(&[
        "causlane",
        "formal",
        "stale-check",
        "--bundle",
        &bundle_path,
        "--generated",
        &generated_path,
        "--scenario",
        &scenario,
        "--receipt",
        &receipt_path,
    ]);
    let receipt_json = std::fs::read_to_string(&receipt_path)?;
    assert!(receipt_json.contains("\"scenario_hash\": \"sha256:"));
    // P0-006: the Alloy codegen receipt records the concrete per-invariant
    // check obligations the generated facts carry, so coverage is grounded in
    // real assertions. A renamed assertion would drop these (drift guard).
    assert!(
        receipt_json.contains("GeneratedTraceSatisfiesCore"),
        "alloy receipt must record its core trace obligation"
    );
    assert!(
        receipt_json.contains("GeneratedApprovalBindingHolds"),
        "alloy receipt must record its I-009 approval-binding obligation"
    );
    assert!(
        receipt_json.contains("GeneratedWitnessFactGrounded"),
        "alloy receipt must record its I-009 witness fact-grounding obligation"
    );
    assert!(
        receipt_json.contains("GeneratedAnchorFactGrounded"),
        "alloy receipt must record its I-009 anchor fact-grounding obligation"
    );

    remove_dir_if_exists(&dir)
}

#[test]
fn strict_bundle_hash_binds_emitted_trace() -> std::io::Result<()> {
    let dir = std::env::temp_dir().join(format!("causlane-cli-strict-{}", std::process::id()));
    remove_dir_if_exists(&dir)?;
    std::fs::create_dir_all(&dir)?;

    let registry = fixture("release_promote.registry.yaml");
    let scenario = scenario_fixture("release_promote_success.scenario.yaml");
    let bundle = dir.join("release_promote.bundle.json");
    let bound = dir.join("bound.trace.json");
    let unbound = dir.join("unbound.trace.json");
    let bundle_path = bundle.display().to_string();
    let bound_path = bound.display().to_string();
    let unbound_path = unbound.display().to_string();

    assert_run_ok(&[
        "causlane",
        "bundle",
        "compile",
        "--registry",
        &registry,
        "--out",
        &bundle_path,
    ]);

    // Emitting with --bundle stamps the hash, so strict verify accepts it.
    assert_run_ok(&[
        "causlane",
        "scenario",
        "emit-trace",
        "--scenario",
        &scenario,
        "--bundle",
        &bundle_path,
        "--out",
        &bound_path,
    ]);
    assert_run_ok(&[
        "causlane",
        "replay",
        "verify",
        "--bundle",
        &bundle_path,
        "--trace",
        &bound_path,
        "--require-bundle-hash",
    ]);

    // Emitting without --bundle leaves the trace unbound; strict verify rejects it.
    assert_run_ok(&[
        "causlane",
        "scenario",
        "emit-trace",
        "--scenario",
        &scenario,
        "--out",
        &unbound_path,
    ]);
    let strict = run(&args(&[
        "causlane",
        "replay",
        "verify",
        "--bundle",
        &bundle_path,
        "--trace",
        &unbound_path,
        "--require-bundle-hash",
    ]));
    let code = match &strict {
        Err(CliError::Replay(err)) => err.code_token(),
        Err(_) => "other-error",
        Ok(_) => "accepted",
    };
    assert_eq!(
        code, "MissingTraceBundleHash",
        "strict verify must reject an unbound trace"
    );

    remove_dir_if_exists(&dir)
}

#[test]
fn builds_sanitized_support_bundle() -> std::io::Result<()> {
    let dir = std::env::temp_dir().join(format!("causlane-cli-support-{}", std::process::id()));
    remove_dir_if_exists(&dir)?;
    std::fs::create_dir_all(&dir)?;

    let registry = fixture("release_promote.registry.yaml");
    let trace = fixture("release_promote.trace.json");
    let bundle = dir.join("release_promote.bundle.json");
    let graph = dir.join("graph.yaml");
    let support = dir.join("support-bundle.json");
    let bundle_path = bundle.display().to_string();
    let graph_path = graph.display().to_string();
    let support_path = support.display().to_string();

    assert_run_ok(&[
        "causlane",
        "bundle",
        "compile",
        "--registry",
        &registry,
        "--out",
        &bundle_path,
    ]);
    std::fs::write(&graph_path, graph_text())?;
    assert_run_ok(&[
        "causlane",
        "support-bundle",
        "build",
        "--bundle",
        &bundle_path,
        "--trace",
        &trace,
        "--graph",
        &graph_path,
        "--out",
        &support_path,
        "--op",
        "release_promote:0",
    ]);

    let json = std::fs::read_to_string(&support_path)?;
    assert!(json.contains("\"schema_version\": 1"));
    assert!(json.contains("\"redaction\""));
    assert!(json.contains("\"subject_values\": \"[redacted]\""));
    assert!(json.contains("\"command\": \"graph export\""));
    assert!(!json.contains("\"authz_decision\": {"));

    remove_dir_if_exists(&dir)
}

#[test]
fn formal_doctor_json_runs() {
    let result = run(&args(&["causlane", "formal", "doctor", "--json"]));
    let output = result.ok();
    assert!(output.is_some());
    if let Some(out) = output {
        assert!(out.text.contains("\"schema_version\""));
    }
}

#[test]
fn unknown_command_is_usage_error() {
    let result = run(&args(&["causlane"]));
    assert!(matches!(result, Err(CliError::Usage(_))));
}
