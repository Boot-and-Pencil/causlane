# Full toolchain addendum for formal lifecycle work

The existing `tools/bootstrap-full`, `tools/full-doctor`, `tools/formal-install` and `.devinfra/tool-versions.json` already cover the main toolchain: base OS tools, Rust, cargo tools, devinfra, Node/npm, Java, dotnet, Alloy, P, Kani, Verus and the z3 bundled with the pinned Verus distribution.

This addendum records the extra toolchain needed for the operational Lean4
proof lane.

## Current required families

```text
base: bash git curl unzip zip tar gzip jq python3 sha256sum
rust: rustup rustc cargo rustfmt clippy rust-src llvm-tools-preview
cargo tools: just bacon bacon-ls cargo-nextest cargo-llvm-cov cargo-deny watchexec
node/npm: node npm
devinfra: cli-checker process-compose semgrep
formal: java javac dotnet alloy p cargo-kani verus z3
proof extras: elan lean lake
```

## Add Lean4 family

`.devinfra/tool-versions.json` includes:

```json
{
  "tools": {
    "formal_tools": {
      "lean4": {
        "enabled": true,
        "source": "github-release+elan",
        "elan_version": "4.2.3",
        "lean_version": "4.31.0",
        "lake_version": "5.0.0",
        "toolchain": "leanprover/lean4:v4.31.0",
        "profile_required": ["proof", "all"]
      }
    }
  }
}
```

Required binaries:

```text
elan
lean
lake
```

Profiles:

```text
base/dev/rust: Lean not required
formal/ci: Lean checked if present, not required
proof/all: Lean required
```

Doctor acceptance:

```bash
tools/full-doctor --json --profile proof
```

must report Lean4 status and fail if missing.

## Installation policy

1. Use pinned versions.
2. Never fabricate a download URL/hash if source metadata is missing.
3. Prefer `elan` for Lean installation and `lake` for project build.
4. Record installed version in doctor report.
5. Count Lean4 coverage only through generated artifacts, tool-run receipts,
   stale-check and the derived coverage report.
