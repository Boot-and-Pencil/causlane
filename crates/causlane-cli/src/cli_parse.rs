//! Pure CLI argument parsing (split from `main.rs` for the 800-line cap).
//!
//! This module never touches the filesystem or environment; it only turns the
//! raw `argv` strings into a typed [`Command`]. All file/env I/O stays in the
//! `main.rs` boundary.

use causlane_formal::FormalProfile;

use causlane_cli::cli_shared::{
    flag_present, flag_value, DEFAULT_FORMAL_ARTIFACT_DIR, DEFAULT_FORMAL_LANE,
    DEFAULT_FORMAL_RECEIPT_DIR,
};
use causlane_cli::formal_artifacts::single_target_from_kind;

use crate::formal_doctor::formal_profile_from_arg;
use crate::Command;

const COMMAND_GRAPH: &str = "graph";
const GRAPH_EXPORT: &str = "export";

pub(crate) fn parse(args: &[String]) -> Option<Command> {
    let command = args.get(1)?;
    if command == "bundle" {
        return parse_bundle(args);
    }
    if command == "replay" {
        return parse_replay(args);
    }
    if command == "explain" {
        return parse_explain(args);
    }
    if command == "why-blocked" {
        return parse_why_blocked(args);
    }
    if command == "why-not-parallel" {
        return parse_why_not_parallel(args);
    }
    if command == COMMAND_GRAPH {
        return parse_graph(args);
    }
    if command == "support-bundle" {
        return parse_support_bundle(args);
    }
    if command == "scenario" {
        return parse_scenario(args);
    }
    if command == "formal" {
        return parse_formal(args);
    }
    if command == "contract" {
        return parse_contract(args);
    }
    None
}

fn parse_contract(args: &[String]) -> Option<Command> {
    let sub = args.get(2)?;
    if sub != "test" {
        return None;
    }
    let manifest = flag_value(args, "--manifest")?;
    let json = flag_present(args, "--json");
    Some(Command::ContractTest { manifest, json })
}

fn parse_bundle(args: &[String]) -> Option<Command> {
    let sub = args.get(2)?;
    if sub == "validate" {
        let path = args.get(3)?;
        return Some(Command::BundleValidate(path.clone()));
    }
    if sub == "compile" {
        let registry = flag_value(args, "--registry")?;
        let out = flag_value(args, "--out")?;
        return Some(Command::BundleCompile { registry, out });
    }
    None
}

fn parse_replay(args: &[String]) -> Option<Command> {
    let sub = args.get(2)?;
    if sub != "verify" {
        return None;
    }
    if args.get(3).is_some_and(|arg| !arg.starts_with("--")) {
        return Some(Command::ReplayVerifyStructural(args.get(3)?.clone()));
    }
    let bundle = flag_value(args, "--bundle")?;
    let trace = flag_value(args, "--trace")?;
    let require_bundle_hash = flag_present(args, "--require-bundle-hash");
    let kernel_secret = flag_value(args, "--kernel-secret");
    let explain = flag_present(args, "--explain");
    let json = flag_present(args, "--json");
    Some(Command::ReplayVerifyWithBundle {
        bundle,
        trace,
        require_bundle_hash,
        kernel_secret,
        explain,
        json,
    })
}

fn parse_explain(args: &[String]) -> Option<Command> {
    let sub = args.get(2)?;
    if sub != "replay" {
        return None;
    }
    let bundle = flag_value(args, "--bundle")?;
    let trace = flag_value(args, "--trace")?;
    let json = flag_present(args, "--json");
    Some(Command::ExplainReplay {
        bundle,
        trace,
        json,
    })
}

fn parse_why_blocked(args: &[String]) -> Option<Command> {
    let graph = flag_value(args, "--graph")?;
    let op = flag_value(args, "--op")?;
    let json = flag_present(args, "--json");
    Some(Command::WhyBlocked { graph, op, json })
}

fn parse_why_not_parallel(args: &[String]) -> Option<Command> {
    let graph = flag_value(args, "--graph")?;
    let op = flag_value(args, "--op")?;
    let with = flag_value(args, "--with")?;
    let json = flag_present(args, "--json");
    Some(Command::WhyNotParallel {
        graph,
        op,
        with,
        json,
    })
}

fn parse_graph(args: &[String]) -> Option<Command> {
    let sub = args.get(2)?;
    if sub != GRAPH_EXPORT {
        return None;
    }
    let graph = flag_value(args, "--graph")?;
    let format = flag_value(args, "--format")?;
    let op = flag_value(args, "--op");
    let out = flag_value(args, "--out");
    Some(Command::GraphExport {
        graph,
        format,
        op,
        out,
    })
}

fn parse_support_bundle(args: &[String]) -> Option<Command> {
    let sub = args.get(2)?;
    if sub != "build" {
        return None;
    }
    let bundle = flag_value(args, "--bundle")?;
    let trace = flag_value(args, "--trace")?;
    let graph = flag_value(args, "--graph")?;
    let out = flag_value(args, "--out")?;
    let op = flag_value(args, "--op");
    Some(Command::SupportBundleBuild {
        bundle,
        trace,
        graph,
        out,
        op,
    })
}

fn parse_scenario(args: &[String]) -> Option<Command> {
    let sub = args.get(2)?;
    if sub == "emit-trace" {
        let scenario = flag_value(args, "--scenario")?;
        let out = flag_value(args, "--out")?;
        let bundle = flag_value(args, "--bundle");
        let kernel_secret = flag_value(args, "--kernel-secret");
        return Some(Command::ScenarioEmitTrace {
            scenario,
            out,
            bundle,
            kernel_secret,
        });
    }
    if sub == "compile" {
        let scenario = flag_value(args, "--scenario")?;
        let bundle = flag_value(args, "--bundle")?;
        let out_dir = flag_value(args, "--out-dir")?;
        let kernel_secret = flag_value(args, "--kernel-secret");
        return Some(Command::ScenarioCompile {
            scenario,
            bundle,
            out_dir,
            kernel_secret,
        });
    }
    if sub == "validate" {
        return Some(Command::ScenarioValidate(args.get(3)?.clone()));
    }
    None
}

#[allow(clippy::too_many_lines)]
fn parse_formal(args: &[String]) -> Option<Command> {
    let sub = args.get(2)?;
    if sub == "doctor" {
        let mut json = false;
        let mut require: Vec<String> = Vec::new();
        let mut profile = FormalProfile::Custom;
        let mut lane = DEFAULT_FORMAL_LANE.to_owned();
        let mut index = 3;
        while let Some(flag) = args.get(index) {
            if flag == "--json" {
                json = true;
                index += 1;
            } else if flag == "--require" {
                if let Some(value) = args.get(index + 1) {
                    for token in value.split(',') {
                        require.push(token.to_owned());
                    }
                }
                index += 2;
            } else if flag == "--profile" {
                if let Some(value) = args.get(index + 1) {
                    profile = formal_profile_from_arg(value);
                    require.extend(profile.requirement_tokens());
                }
                index += 2;
            } else if flag == "--lane" {
                if let Some(value) = args.get(index + 1) {
                    lane.clone_from(value);
                }
                index += 2;
            } else {
                index += 1;
            }
        }
        return Some(Command::FormalDoctor {
            json,
            require,
            profile,
            lane,
        });
    }
    if sub == "generate" && args.get(3).is_some_and(|kind| kind == "alloy") {
        let bundle = flag_value(args, "--bundle")?;
        let out = flag_value(args, "--out")?;
        let scenario = flag_value(args, "--scenario");
        let receipt = flag_value(args, "--receipt");
        return Some(Command::FormalGenerateAlloy {
            bundle,
            out,
            scenario,
            receipt,
        });
    }
    if sub == "generate" && args.get(3).is_some_and(|kind| kind == "all") {
        let bundle = flag_value(args, "--bundle")?;
        let scenario = flag_value(args, "--scenario");
        let artifact_dir = flag_value(args, "--artifact-dir")
            .unwrap_or_else(|| DEFAULT_FORMAL_ARTIFACT_DIR.to_owned());
        let receipt_dir = flag_value(args, "--receipt-dir")
            .unwrap_or_else(|| DEFAULT_FORMAL_RECEIPT_DIR.to_owned());
        return Some(Command::FormalGenerateAll {
            bundle,
            scenario,
            artifact_dir,
            receipt_dir,
        });
    }
    if sub == "generate" {
        if let Some(target) = single_target_from_kind(args.get(3)?) {
            let bundle = flag_value(args, "--bundle")?;
            let out = flag_value(args, "--out")?;
            let scenario = flag_value(args, "--scenario");
            let receipt = flag_value(args, "--receipt");
            return Some(Command::FormalGenerate {
                target,
                bundle,
                scenario,
                out,
                receipt,
            });
        }
    }
    if sub == "stale-check" {
        let bundle = flag_value(args, "--bundle")?;
        let generated = flag_value(args, "--generated")?;
        let scenario = flag_value(args, "--scenario");
        let receipt = flag_value(args, "--receipt");
        return Some(Command::FormalStaleCheck {
            bundle,
            generated,
            scenario,
            receipt,
        });
    }
    if sub == "stale-check-all" {
        let bundle = flag_value(args, "--bundle")?;
        let scenario = flag_value(args, "--scenario");
        let artifact_dir = flag_value(args, "--artifact-dir")
            .unwrap_or_else(|| DEFAULT_FORMAL_ARTIFACT_DIR.to_owned());
        let receipt_dir = flag_value(args, "--receipt-dir")
            .unwrap_or_else(|| DEFAULT_FORMAL_RECEIPT_DIR.to_owned());
        return Some(Command::FormalStaleCheckAll {
            bundle,
            scenario,
            artifact_dir,
            receipt_dir,
        });
    }
    if sub == "ir" && args.get(3).is_some_and(|kind| kind == "emit") {
        let bundle = flag_value(args, "--bundle")?;
        let scenario = flag_value(args, "--scenario");
        let out = flag_value(args, "--out")?;
        return Some(Command::FormalIrEmit {
            bundle,
            scenario,
            out,
        });
    }
    None
}

pub(crate) fn usage() -> String {
    [
        "causlane CLI",
        "usage:",
        "  causlane bundle validate <registry.yaml>",
        "  causlane bundle compile --registry <registry.yaml> --out <bundle.json>",
        "  causlane replay verify <trace.json>   (structural-only; pass --bundle for full replay verification)",
        "  causlane replay verify --bundle <bundle.json> --trace <trace.json> [--require-bundle-hash] [--kernel-secret <secret>] [--explain] [--json]",
        "  causlane explain replay --bundle <bundle.json> --trace <trace.json> [--json]",
        "  causlane why-blocked --graph <graph.yaml|json> --op <action_id>:<op_index> [--json]",
        "  causlane why-not-parallel --graph <graph.yaml|json> --op <action_id>:<op_index> --with <action_id>:<op_index> [--json]",
        "  causlane graph export --graph <graph.yaml|json> --format <json|mermaid|dot> [--op <action_id>:<op_index>] [--out <path>]",
        "  causlane support-bundle build --bundle <bundle.json> --trace <trace.json> --graph <graph.yaml|json> --out <support-bundle.json> [--op <action_id>:<op_index>]",
        "  causlane scenario emit-trace --scenario <scenario.yaml> --out <trace.json> [--bundle <bundle.json>] [--kernel-secret <secret>]",
        "  causlane scenario compile --scenario <scenario.yaml> --bundle <bundle.json> --out-dir <dir> [--kernel-secret <secret>]",
        "  causlane contract test --manifest <contract-tests.yaml> [--json]",
        "  causlane scenario validate <scenario.yaml>",
        "  causlane formal doctor [--json] [--profile custom|base|rust|proof|all] [--lane <profile-lane>] [--require a,b,...]",
        "  causlane formal generate <alloy|p|kani|verus|lean4|all> --bundle <bundle.json> --out <artifact> [--scenario <scenario.yaml>] [--receipt <receipt.json>]",
        "  causlane formal stale-check --bundle <bundle.json> --generated <facts.als> [--scenario <scenario.yaml>] [--receipt <receipt.json>]",
        "  causlane formal ir emit --bundle <bundle.json> [--scenario <scenario.yaml>] --out <formal_ir.json>",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::Command;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    #[test]
    fn parses_explain_replay_alias() {
        let parsed = parse(&args(&[
            "causlane",
            "explain",
            "replay",
            "--bundle",
            "bundle.json",
            "--trace",
            "trace.json",
            "--json",
        ]));
        assert!(matches!(
            parsed,
            Some(Command::ExplainReplay {
                bundle,
                trace,
                json: true,
            }) if bundle == "bundle.json" && trace == "trace.json"
        ));
    }

    #[test]
    fn parses_why_commands() {
        let blocked = parse(&args(&[
            "causlane",
            "why-blocked",
            "--graph",
            "graph.yaml",
            "--op",
            "act:0",
            "--json",
        ]));
        assert!(matches!(
            blocked,
            Some(Command::WhyBlocked {
                graph,
                op,
                json: true,
            }) if graph == "graph.yaml" && op == "act:0"
        ));

        let pair = parse(&args(&[
            "causlane",
            "why-not-parallel",
            "--graph",
            "graph.yaml",
            "--op",
            "act:0",
            "--with",
            "other:0",
        ]));
        assert!(matches!(
            pair,
            Some(Command::WhyNotParallel {
                graph,
                op,
                with,
                json: false,
            }) if graph == "graph.yaml" && op == "act:0" && with == "other:0"
        ));
    }

    #[test]
    fn parses_graph_export_command() {
        let parsed = parse(&args(&[
            "causlane",
            "graph",
            "export",
            "--graph",
            "graph.yaml",
            "--format",
            "mermaid",
            "--op",
            "act:0",
            "--out",
            "graph.mmd",
        ]));
        assert!(matches!(
            parsed,
            Some(Command::GraphExport {
                graph,
                format,
                op: Some(op),
                out: Some(out),
            }) if graph == "graph.yaml"
                && format == "mermaid"
                && op == "act:0"
                && out == "graph.mmd"
        ));
    }

    #[test]
    fn parses_support_bundle_build_command() {
        let parsed = parse(&args(&[
            "causlane",
            "support-bundle",
            "build",
            "--bundle",
            "bundle.json",
            "--trace",
            "trace.json",
            "--graph",
            "graph.yaml",
            "--out",
            "support.json",
            "--op",
            "act:0",
        ]));
        assert!(matches!(
            parsed,
            Some(Command::SupportBundleBuild {
                bundle,
                trace,
                graph,
                out,
                op: Some(op),
            }) if bundle == "bundle.json"
                && trace == "trace.json"
                && graph == "graph.yaml"
                && out == "support.json"
                && op == "act:0"
        ));
    }
}
