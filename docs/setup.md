# Setup — full software install checklist

This is the consolidated, executable install checklist for the Causlane repo
(spec §9 / §10 / §11.4). It is the human-readable companion to two scripts:

- `tools/bootstrap-full` — idempotent installer, profile-driven.
- `tools/full-doctor` (`just doctor-full`) — fast tool-presence/version gate
  that emits JSON.

Both read pinned versions/URLs/SHAs from `.devinfra/tool-versions.json`, which
is the single source of truth for tool versions. Formal-tool provisioning is
delegated to the existing `tools/formal-install`, and formal-tool diagnosis to
the existing `tools/formal-doctor`.

## TL;DR

```bash
# Provision a profile (idempotent — safe to re-run):
tools/bootstrap-full --profile dev
just bootstrap-full --profile formal

# Verify the install surface (fast, JSON, no heavy build):
just doctor-full                          # == tools/full-doctor --json --profile all
tools/full-doctor --json --profile base
tools/full-doctor --json --profile proof
```

`full-doctor` exits 0 when every **required** tool for the profile is present.
Optional tools that are missing (or merely version-drifted) are reported as
`warn`, never a hard failure — unless you pass `--require-optional`.

## Profiles

| Profile  | Adds on top of previous                                                |
|----------|-----------------------------------------------------------------------|
| `base`   | OS basics + `jq` + `python3` (NO Rust required)                       |
| `dev`    | Rust toolchain + cargo tools + devinfra + Node/npm                    |
| `formal` | Java/`javac` + dotnet + formal tools (Alloy/Z3/P/Kani/Verus)          |
| `ci`     | formal smoke; **no optional agent CLIs**                              |
| `proof`  | Verus + Rust 1.95.0 (for Verus) + Z3 + Lean4/Elan/Lake, strict proof profile |
| `all`    | everything + optional agent CLIs (checked-but-optional)              |

The `ci` profile never requires the optional agent CLIs, and `bootstrap-full`
never installs them in `ci`. Agent CLIs are optional everywhere (policy: they
must not be mandatory for CI/formal gates).

## Rust version policy (reconciliation)

`rust-toolchain.toml` is the **canonical source of truth** and pins:

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy", "rust-src", "llvm-tools-preview"]
```

`.devinfra/tool-versions.json` records `rust.version` (currently `1.96.0`) as
the **observed resolved version** — a diagnostic snapshot for reproducibility,
**not** a second pin. This is a deliberate design (rolling `stable` channel +
recorded observed version), not a drift to be "fixed" by hardcoding a channel
in the JSON.

Consequences:

- `full-doctor` resolves `rustc`/`cargo` with the working directory at the repo
  root so `rust-toolchain.toml` applies; the observed version then matches
  `tool-versions.json`. A version difference is surfaced as a `warn`, not a hard
  failure (the channel, not the JSON, decides the toolchain).
- If you ever want a fully pinned reproducible Rust, change the **channel** in
  `rust-toolchain.toml` (e.g. `channel = "1.96.0"`) — that is the one place that
  decides the toolchain. Do not pin in the JSON.

Verus additionally needs a separate Rust `1.95.0` toolchain
(`.devinfra/tool-versions.json` → `formal_tools.verus.requires_rust_toolchain`):

```bash
rustup toolchain install 1.95.0
rustup component add rust-src --toolchain 1.95.0
rustc +1.95.0 --version
```

`bootstrap-full --profile proof` ensures this toolchain; `full-doctor --profile
proof` requires it.

Lean4 is required for the full `formal-verify-all` gate and is installed
repo-locally through `elan`:

```bash
tools/formal-install lean4
tools/lean4-env lean --version
tools/lean4-env lake --version
```

Lean 4.31.0 reports Lake as a source/build-suffixed version such as
`5.0.0-src+...`; `.devinfra/tool-versions.json` records the expected Lake
family as `5.0.0`. Treat a `5.0.0-src+...` report as matching the pinned Lake
5.0.0 line, not as a toolchain drift.

`bootstrap-full --profile proof` ensures this toolchain; `full-doctor --profile
proof` requires it. `formal/proof-lanes.json` marks Verus and Lean4 as
always-blocking proof lanes, so the executable formal exceptions policy rejects
attempts to skip them.

## Human checklist

### Base

- [ ] `git`, `curl`, `unzip`, `zip`, `tar`, `gzip`, `jq`, `python3` installed.
- [ ] Compiler/build tools: `gcc`/`g++` or `clang`, `make`, `pkg-config`,
      `libssl-dev` (or equivalent).
- [ ] Java 17+ and `javac` installed.
- [ ] dotnet SDK 8 installed.
- [ ] `java -version`, `javac -version`, `dotnet --version`, `jq --version` pass.

Ubuntu/Debian one-liner (see spec §9.2 for the full list):

```bash
sudo apt-get update
sudo apt-get install -y bash ca-certificates curl wget git unzip zip tar gzip \
  xz-utils file build-essential make gcc g++ clang pkg-config libssl-dev \
  coreutils findutils sed grep gawk perl jq python3 python3-venv python3-pip \
  openjdk-17-jdk dotnet-sdk-8.0
```

macOS (see spec §9.3):

```bash
xcode-select --install || true
brew install git curl jq python@3 openjdk@17 dotnet-sdk coreutils gnu-sed grep make
```

### Rust dev

- [ ] `rustup` installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y`).
- [ ] Project toolchain installed (channel from `rust-toolchain.toml`).
- [ ] Components `rustfmt`, `clippy`, `rust-src`, `llvm-tools-preview` present.
- [ ] Cargo tools installed: `just`, `bacon`, `bacon-ls`, `cargo-nextest`,
      `cargo-llvm-cov`, `cargo-deny`, `watchexec` (versions from
      `tool-versions.json`).
- [ ] `just rust-full-check` passes.

### Devinfra

- [ ] `cli-checker` installed (see note below).
- [ ] `process-compose` installed (sha-verified by `bootstrap-full`).
- [ ] `semgrep` installed (pinned version).
- [ ] `just doctor` passes; `just checker-status` shows fresh/pass.

> **cli-checker note (P0 gap):** `tool-versions.json` currently has **empty**
> `archive_url`/`archive_sha256` for `cli_checker`. `bootstrap-full` therefore
> does **not** fabricate a URL or hash; it detects a pre-installed
> `cli-checker` and otherwise emits a "manual install required" remediation.
> Once a reproducible, hashed download source is published, populate those two
> fields and `bootstrap-full` will auto-install with sha256 verification.

### Node/npm (optional)

- [ ] Node/npm installed (nvm/fnm/volta/mise).
- [ ] npm updated to the pinned version (drift is a `warn`, not a failure).
- [ ] Optional agent CLIs installed only in developer profiles, never required
      for CI/formal gates.

### Formal

Provision via the existing installer:

```bash
tools/formal-install alloy
tools/formal-install p
tools/formal-install kani
tools/formal-install verus
tools/formal-install z3      # validates the z3 bundled with the pinned Verus dist
# or: tools/formal-install all
```

- [ ] `.tools/alloy/alloy.jar` downloaded and sha256-verified.
- [ ] `AlloyRunner.java` compiles.
- [ ] `.tools/verus_dist/verus-x86-linux/z3 --version` reports the pinned version.
- [ ] `p --version` passes.
- [ ] `cargo-kani --version` passes (and `cargo-kani setup` run).
- [ ] Verus binary installed; Rust `1.95.0` toolchain installed;
      `verus --version` passes.
- [ ] `python3 tools/formal-doctor --json --profile all` passes (or honestly
      reports missing optional tools).

### Repo gates

These are heavy and are **not** run by `doctor-full` by default. Run them
explicitly:

```bash
just check
just clippy
just test-build
just test
just coverage
just formal-ready
just formal-verify-all
tools/formal-verify-all --profile proof   # in a proof-capable environment
```

`just coverage` enforces workspace line coverage >=85% while still writing the
LCOV summary consumed by devinfra.

`tools/full-doctor --with-gates` will additionally shell out to
`just rust-full-check` and `just formal-verify-all`; this is **off by default**
because `doctor-full` is meant to be a fast presence/version gate.

## Outputs

- `tools/bootstrap-full` writes `target/causlane/bootstrap-full-report.json`
  (`{schema_version, profile, status, actions[]}`).
- `tools/full-doctor --json` emits
  `{schema_version, profile, status, tools[], remediation[]}` to stdout.

## Acceptance (spec §11.4)

- [x] `tools/bootstrap-full --profile all` exists and is idempotent.
- [x] `just doctor-full` exists and emits JSON.
- [x] Doctor checks base/dev/formal/proof tools.
- [x] CI profile does not require optional agent CLIs.
- [x] Proof profile requires Verus + Rust 1.95.0 + Z3.
- [x] Rust version policy documented and reconciled (canonical
      `rust-toolchain.toml` channel + observed version in JSON).
- [ ] `.devinfra/tool-versions.json` has complete URLs/SHA for **all**
      downloadable tools — **open**: `cli_checker.archive_url` /
      `archive_sha256` are still empty (see cli-checker note above).
