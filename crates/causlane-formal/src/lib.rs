//! Formal-toolchain readiness logic — the pure core of `causlane formal doctor`.
//!
//! This crate is pure: it takes already-gathered environment facts ([`EnvFacts`])
//! and a `--require` token list, and produces a [`DoctorReport`]. Actual
//! environment probing (PATH scanning, file checks, reading
//! `.devinfra/tool-versions.json`) is I/O and lives at the binary boundary
//! (`causlane-cli`), keeping this crate free of side effects (ADR-0004).

#![forbid(unsafe_code)]
#![deny(warnings)]

use core::fmt::Write as _;
use serde::Serialize;

/// Gathered facts about one tool (presence + pinned version), supplied by the
/// caller's platform adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolFacts {
    /// Whether the tool was found on the host.
    pub found: bool,
    /// The pinned version, if known.
    pub version: Option<String>,
}

impl ToolFacts {
    /// Construct facts for a tool.
    #[must_use]
    pub fn new(found: bool, version: Option<String>) -> Self {
        Self { found, version }
    }

    /// Convenience: a tool that was not found.
    #[must_use]
    pub fn absent() -> Self {
        Self {
            found: false,
            version: None,
        }
    }
}

/// All gathered environment facts (named fields avoid a stringly-keyed map).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvFacts {
    /// rustc.
    pub rustc: ToolFacts,
    /// cargo.
    pub cargo: ToolFacts,
    /// rustup.
    pub rustup: ToolFacts,
    /// java.
    pub java: ToolFacts,
    /// jq.
    pub jq: ToolFacts,
    /// python3.
    pub python3: ToolFacts,
    /// dotnet.
    pub dotnet: ToolFacts,
    /// Alloy jar.
    pub alloy: ToolFacts,
    /// P CLI.
    pub p: ToolFacts,
    /// cargo-kani.
    pub cargo_kani: ToolFacts,
    /// verus.
    pub verus: ToolFacts,
    /// elan toolchain manager.
    pub elan: ToolFacts,
    /// Lean4 compiler.
    pub lean: ToolFacts,
    /// Lake build tool.
    pub lake: ToolFacts,
    /// z3.
    pub z3: ToolFacts,
}

/// Presence/version of a single tool, with its requirement flag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolStatus {
    /// Whether this tool is required by the active profile.
    pub required: bool,
    /// Whether the tool was found.
    pub found: bool,
    /// The pinned version, if known and the tool is present.
    pub version: Option<String>,
}

impl ToolStatus {
    fn build(required: bool, facts: &ToolFacts) -> Self {
        Self {
            required,
            found: facts.found,
            version: if facts.found {
                facts.version.clone()
            } else {
                None
            },
        }
    }
}

/// Per-tool statuses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolsReport {
    /// The Rust compiler.
    pub rustc: ToolStatus,
    /// Cargo.
    pub cargo: ToolStatus,
    /// rustup toolchain manager.
    pub rustup: ToolStatus,
    /// Java runtime (Alloy needs >= 17).
    pub java: ToolStatus,
    /// jq.
    pub jq: ToolStatus,
    /// python3.
    pub python3: ToolStatus,
    /// dotnet SDK (P needs major 8).
    pub dotnet: ToolStatus,
    /// Alloy distribution jar.
    pub alloy: ToolStatus,
    /// P language CLI.
    pub p: ToolStatus,
    /// Kani (cargo-kani driver).
    #[serde(rename = "cargo-kani")]
    pub cargo_kani: ToolStatus,
    /// Verus binary.
    pub verus: ToolStatus,
    /// elan toolchain manager.
    pub elan: ToolStatus,
    /// Lean4 compiler.
    pub lean: ToolStatus,
    /// Lake build tool.
    pub lake: ToolStatus,
    /// z3 SMT solver.
    pub z3: ToolStatus,
}

impl ToolsReport {
    fn required_found(&self) -> [(bool, bool); 15] {
        [
            (self.rustc.required, self.rustc.found),
            (self.cargo.required, self.cargo.found),
            (self.rustup.required, self.rustup.found),
            (self.java.required, self.java.found),
            (self.jq.required, self.jq.found),
            (self.python3.required, self.python3.found),
            (self.dotnet.required, self.dotnet.found),
            (self.alloy.required, self.alloy.found),
            (self.p.required, self.p.found),
            (self.cargo_kani.required, self.cargo_kani.found),
            (self.verus.required, self.verus.found),
            (self.elan.required, self.elan.found),
            (self.lean.required, self.lean.found),
            (self.lake.required, self.lake.found),
            (self.z3.required, self.z3.found),
        ]
    }
}

/// The full doctor report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorReport {
    /// Schema version of this report shape.
    pub schema_version: u32,
    /// Overall status: `ok`, `missing`, `required_missing`, or
    /// `lane_contract_failed`.
    pub status: String,
    /// Active requirement profile.
    pub profile: String,
    /// Active verification lane.
    pub lane: String,
    /// Per-lane checks that are not tied to a single binary.
    pub lane_checks: Vec<LaneCheck>,
    /// Per-tool statuses.
    pub tools: ToolsReport,
}

/// One lane-contract check, e.g. clean worktree for shared lanes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LaneCheck {
    /// Stable check name.
    pub name: String,
    /// Check status token.
    pub status: String,
    /// Whether a non-ok status fails this doctor invocation.
    pub required: bool,
}

impl LaneCheck {
    /// Construct a lane check.
    #[must_use]
    pub fn new(name: &str, status: &str, required: bool) -> Self {
        Self {
            name: name.to_owned(),
            status: status.to_owned(),
            required,
        }
    }
}

/// Built-in formal doctor profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormalProfile {
    /// No built-in requirements; caller-supplied `--require` only.
    Custom,
    /// Metadata/codegen plus Alloy/P smoke surface. Does not require Verus.
    Base,
    /// Rust/Kani harness surface. Does not require Verus.
    Rust,
    /// Optional proof surface. Explicitly requires Verus and Lean4.
    Proof,
    /// Developer profile requiring every known formal tool.
    All,
}

impl FormalProfile {
    /// Stable profile token.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FormalProfile::Custom => "custom",
            FormalProfile::Base => "base",
            FormalProfile::Rust => "rust",
            FormalProfile::Proof => "proof",
            FormalProfile::All => "all",
        }
    }

    /// Tool requirement tokens for this profile.
    #[must_use]
    pub fn requirement_tokens(self) -> Vec<String> {
        match self {
            FormalProfile::Custom => Vec::new(),
            FormalProfile::Base => ["alloy", "java", "jq", "python3", "p", "dotnet"]
                .iter()
                .map(|token| (*token).to_owned())
                .collect(),
            FormalProfile::Rust => ["rustc", "cargo", "rustup", "cargo-kani"]
                .iter()
                .map(|token| (*token).to_owned())
                .collect(),
            FormalProfile::Proof => ["verus", "z3", "lean4"]
                .iter()
                .map(|token| (*token).to_owned())
                .collect(),
            FormalProfile::All => ["all"].iter().map(|token| (*token).to_owned()).collect(),
        }
    }
}

impl DoctorReport {
    /// Serialize the report to pretty JSON.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_owned())
    }

    /// Render a short human-readable summary.
    #[must_use]
    pub fn to_human(&self) -> String {
        let tool = &self.tools;
        let rows = [
            ("rustc", &tool.rustc),
            ("cargo", &tool.cargo),
            ("rustup", &tool.rustup),
            ("java", &tool.java),
            ("jq", &tool.jq),
            ("python3", &tool.python3),
            ("dotnet", &tool.dotnet),
            ("alloy", &tool.alloy),
            ("p", &tool.p),
            ("cargo-kani", &tool.cargo_kani),
            ("verus", &tool.verus),
            ("elan", &tool.elan),
            ("lean", &tool.lean),
            ("lake", &tool.lake),
            ("z3", &tool.z3),
        ];
        let mut out = format!(
            "formal doctor: {} profile={} lane={}\n",
            self.status, self.profile, self.lane
        );
        for (name, status) in rows {
            let mark = if status.found { "ok     " } else { "MISSING" };
            let req = if status.required { " (required)" } else { "" };
            let version = status.version.as_deref().unwrap_or("-");
            let _written = writeln!(out, "  {mark} {name:<11}{req}  {version}");
        }
        for check in &self.lane_checks {
            let req = if check.required { " (required)" } else { "" };
            let _written = writeln!(out, "  {:<7} lane:{}{}", check.status, check.name, req);
        }
        out
    }

    /// Whether all required tools are present.
    #[must_use]
    pub fn required_satisfied(&self) -> bool {
        !matches!(
            self.status.as_str(),
            "required_missing" | "lane_contract_failed"
        )
    }
}

/// Which tools the active profile requires.
#[allow(clippy::struct_excessive_bools)] // a flag set, not a state machine
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Requirement {
    rustc: bool,
    cargo: bool,
    rustup: bool,
    java: bool,
    jq: bool,
    python3: bool,
    dotnet: bool,
    alloy: bool,
    p: bool,
    cargo_kani: bool,
    verus: bool,
    elan: bool,
    lean: bool,
    lake: bool,
    z3: bool,
}

impl Requirement {
    /// Build a requirement set from `--require` tokens. `all` requires
    /// everything, `rust` the Rust base; otherwise each token names one tool.
    #[must_use]
    pub fn from_tokens(tokens: &[String]) -> Self {
        let mut req = Self::default();
        for raw in tokens {
            let token = raw.trim();
            if token == "all" {
                req = Self::everything();
            } else if token == "rust" {
                req.rustc = true;
                req.cargo = true;
                req.rustup = true;
            } else if token == "rustc" {
                req.rustc = true;
            } else if token == "cargo" {
                req.cargo = true;
            } else if token == "rustup" {
                req.rustup = true;
            } else if token == "java" {
                req.java = true;
            } else if token == "jq" {
                req.jq = true;
            } else if token == "python3" {
                req.python3 = true;
            } else if token == "dotnet" {
                req.dotnet = true;
            } else if token == "alloy" {
                req.alloy = true;
            } else if token == "p" {
                req.p = true;
            } else if token == "kani" || token == "cargo-kani" {
                req.cargo_kani = true;
            } else if token == "verus" {
                req.verus = true;
            } else if token == "lean4" {
                req.elan = true;
                req.lean = true;
                req.lake = true;
            } else if token == "elan" {
                req.elan = true;
            } else if token == "lean" {
                req.lean = true;
            } else if token == "lake" {
                req.lake = true;
            } else if token == "z3" {
                req.z3 = true;
            }
        }
        req
    }

    fn everything() -> Self {
        Self {
            rustc: true,
            cargo: true,
            rustup: true,
            java: true,
            jq: true,
            python3: true,
            dotnet: true,
            alloy: true,
            p: true,
            cargo_kani: true,
            verus: true,
            elan: true,
            lean: true,
            lake: true,
            z3: true,
        }
    }
}

/// Compute a doctor report from `--require` tokens and gathered environment
/// facts. Pure: no I/O.
#[must_use]
pub fn report(require: &[String], env: &EnvFacts) -> DoctorReport {
    report_with_context(
        require,
        env,
        FormalProfile::Custom,
        "local_smoke",
        Vec::new(),
    )
}

/// Compute a doctor report with explicit profile/lane context. Pure: lane
/// checks are gathered by the caller and supplied as data.
#[must_use]
pub fn report_with_context(
    require: &[String],
    env: &EnvFacts,
    profile: FormalProfile,
    lane: &str,
    lane_checks: Vec<LaneCheck>,
) -> DoctorReport {
    let req = Requirement::from_tokens(require);
    let tools = ToolsReport {
        rustc: ToolStatus::build(req.rustc, &env.rustc),
        cargo: ToolStatus::build(req.cargo, &env.cargo),
        rustup: ToolStatus::build(req.rustup, &env.rustup),
        java: ToolStatus::build(req.java, &env.java),
        jq: ToolStatus::build(req.jq, &env.jq),
        python3: ToolStatus::build(req.python3, &env.python3),
        dotnet: ToolStatus::build(req.dotnet, &env.dotnet),
        alloy: ToolStatus::build(req.alloy, &env.alloy),
        p: ToolStatus::build(req.p, &env.p),
        cargo_kani: ToolStatus::build(req.cargo_kani, &env.cargo_kani),
        verus: ToolStatus::build(req.verus, &env.verus),
        elan: ToolStatus::build(req.elan, &env.elan),
        lean: ToolStatus::build(req.lean, &env.lean),
        lake: ToolStatus::build(req.lake, &env.lake),
        z3: ToolStatus::build(req.z3, &env.z3),
    };

    let mut any_required_missing = false;
    let mut any_missing = false;
    for (required, found) in tools.required_found() {
        if !found {
            any_missing = true;
            if required {
                any_required_missing = true;
            }
        }
    }
    let any_lane_failed = lane_checks
        .iter()
        .any(|check| check.required && check.status != "ok");
    let status = if any_required_missing {
        "required_missing"
    } else if any_lane_failed {
        "lane_contract_failed"
    } else if any_missing {
        "missing"
    } else {
        "ok"
    };

    DoctorReport {
        schema_version: 1,
        status: status.to_owned(),
        profile: profile.as_str().to_owned(),
        lane: lane.to_owned(),
        lane_checks,
        tools,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        report, report_with_context, EnvFacts, FormalProfile, LaneCheck, Requirement, ToolFacts,
    };

    fn all_present() -> EnvFacts {
        let present = || ToolFacts::new(true, None);
        EnvFacts {
            rustc: present(),
            cargo: present(),
            rustup: present(),
            java: present(),
            jq: present(),
            python3: present(),
            dotnet: present(),
            alloy: present(),
            p: present(),
            cargo_kani: present(),
            verus: present(),
            elan: present(),
            lean: present(),
            lake: present(),
            z3: present(),
        }
    }

    #[test]
    fn all_present_is_ok() {
        let rep = report(&[], &all_present());
        assert_eq!(rep.schema_version, 1);
        assert_eq!(rep.status, "ok");
        assert!(rep.to_json().contains("\"cargo-kani\""));
    }

    #[test]
    fn optional_missing_is_missing_not_required_missing() {
        let mut env = all_present();
        env.verus = ToolFacts::absent();
        let rep = report(&[], &env);
        assert_eq!(rep.status, "missing");
        assert!(rep.required_satisfied());
    }

    #[test]
    fn required_missing_fails() {
        let mut env = all_present();
        env.verus = ToolFacts::absent();
        let rep = report(&["all".to_owned()], &env);
        assert_eq!(rep.status, "required_missing");
        assert!(!rep.required_satisfied());
    }

    #[test]
    fn require_specific_tokens() {
        let req = Requirement::from_tokens(&["alloy".to_owned(), "z3".to_owned()]);
        let expected = Requirement {
            alloy: true,
            z3: true,
            ..Requirement::default()
        };
        assert_eq!(req, expected);
    }

    #[test]
    fn proof_profile_requires_proof_tools_when_selected() {
        let mut env = all_present();
        env.verus = ToolFacts::absent();
        env.lean = ToolFacts::absent();
        let rep = report_with_context(
            &FormalProfile::Proof.requirement_tokens(),
            &env,
            FormalProfile::Proof,
            "local_smoke",
            Vec::new(),
        );
        assert_eq!(rep.status, "required_missing");
        assert!(rep.tools.verus.required);
        assert!(rep.tools.elan.required);
        assert!(rep.tools.lean.required);
        assert!(rep.tools.lake.required);
    }

    #[test]
    fn lane_contract_failure_is_required_failure() {
        let rep = report_with_context(
            &[],
            &all_present(),
            FormalProfile::Base,
            "fast_ci",
            vec![LaneCheck::new("clean_worktree", "fail", true)],
        );
        assert_eq!(rep.status, "lane_contract_failed");
        assert!(!rep.required_satisfied());
    }
}
