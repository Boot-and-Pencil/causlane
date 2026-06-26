//! Pinned tool versions parsed from `.devinfra/tool-versions.json` (P1-005).
//!
//! The Python `tools/formal-doctor` is the version/SHA **verification** authority
//! (it probes the installed version and compares it to the pin). The Rust
//! `causlane formal doctor` is the in-repo convenience: it reports tool presence
//! plus the **pinned** version from this manifest, so the two no longer disagree
//! (the Rust doctor previously reported `version: null` for everything).
//!
//! This module is pure (a typed DTO + a `parse` over text); the file read happens
//! at the `main.rs` boundary.

use serde::Deserialize;

/// The pinned versions the Rust doctor surfaces, keyed by tool family.
#[derive(Debug, Default)]
pub(crate) struct PinnedVersions {
    /// Pin for the Rust toolchain (`rustc`/`cargo`/`rustup`).
    pub rust: Option<String>,
    /// Alloy pin.
    pub alloy: Option<String>,
    /// cargo-kani pin.
    pub kani: Option<String>,
    /// P pin.
    pub p: Option<String>,
    /// Verus pin.
    pub verus: Option<String>,
    /// z3 pin.
    pub z3: Option<String>,
}

impl PinnedVersions {
    /// Parse the pins from the manifest text; malformed/partial text yields
    /// all-`None` so the doctor still reports presence.
    pub(crate) fn parse(text: &str) -> Self {
        let Ok(manifest) = serde_json::from_str::<Manifest>(text) else {
            return Self::default();
        };
        Self {
            rust: manifest.tools.rust.version,
            alloy: manifest.tools.formal_tools.alloy.version,
            kani: manifest.tools.formal_tools.kani.version,
            p: manifest.tools.formal_tools.p.version,
            verus: manifest.tools.formal_tools.verus.version,
            z3: manifest.tools.formal_tools.z3.version,
        }
    }
}

#[derive(Deserialize, Default)]
struct Manifest {
    #[serde(default)]
    tools: Tools,
}

#[derive(Deserialize, Default)]
struct Tools {
    #[serde(default)]
    rust: VersionNode,
    #[serde(default)]
    formal_tools: FormalTools,
}

#[derive(Deserialize, Default)]
struct FormalTools {
    #[serde(default)]
    alloy: VersionNode,
    #[serde(default)]
    kani: VersionNode,
    #[serde(default)]
    p: VersionNode,
    #[serde(default)]
    verus: VersionNode,
    #[serde(default)]
    z3: VersionNode,
}

#[derive(Deserialize, Default)]
struct VersionNode {
    #[serde(default)]
    version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::PinnedVersions;

    const MANIFEST: &str = include_str!("../fixtures/.devinfra/tool-versions.json");

    #[test]
    fn reads_the_repo_manifest_pins() {
        let pins = PinnedVersions::parse(MANIFEST);
        // The repo manifest pins the Rust toolchain and the formal tools.
        assert!(pins.rust.is_some());
        assert!(pins.alloy.is_some());
        assert!(pins.p.is_some());
    }

    #[test]
    fn malformed_manifest_is_all_none() {
        let pins = PinnedVersions::parse("not json");
        assert!(pins.rust.is_none());
        assert!(pins.alloy.is_none());
    }
}
