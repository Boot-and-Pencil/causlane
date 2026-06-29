# CI Dispatcher

`ci-dispatcher.lan` is the repository's LAN host for heavier CI-adjacent
checks that need the formal, property-test and cargo-fuzz toolchains installed
together. It is not a second source of truth: it runs the checked-in gates
against a git checkout at `origin/main`.

Machine-specific facts, drift notes, local scratch and host inventory belong in
`local/ci-dispatcher.local.md`. That file is intentionally ignored by git. This
document only records the portable contract and commands.

## Preflight

Run the dispatcher preflight from a normal checkout:

```bash
tools/ci-dispatcher-preflight
just ci-dispatcher-preflight
```

The default host is `ci-dispatcher.lan`; the default checkout path is
`/workspace/repo-main`. Override them when needed:

```bash
CI_DISPATCHER_HOST=ci-dispatcher.lan \
CI_DISPATCHER_REPO=/workspace/repo-main \
tools/ci-dispatcher-preflight

tools/ci-dispatcher-preflight --host ci-dispatcher.lan --repo /workspace/repo-main
```

The preflight is deliberately fail-closed:

- the remote checkout must be clean;
- sync is `git fetch origin main` followed by `git merge --ff-only origin/main`;
- the final `HEAD`, `origin/main` and `git ls-remote origin refs/heads/main`
  values must match;
- non-fast-forward or unrelated checkout history is reported, not repaired.

## Gate Surface

The preflight checks the dispatcher has the expected tools on `PATH`, including
Rust stable, `nightly-2025-11-21`, `cargo-fuzz`, `cargo-kani`, Z3, Java, `javac`
and dotnet. Then it runs:

```bash
python3 tools/formal-doctor --json --profile all --lane local_smoke
./tools/cargo-dev test -p causlane-formal --test proptest_smoke --locked
./tools/cargo-dev test -p causlane-core --test proptest_protocol_properties --locked
./tools/cargo-dev test -p causlane-replay --test proptest_parse_boundaries --locked
```

Portable cargo-fuzz equivalents:

```bash
cargo +nightly-2025-11-21 test --manifest-path fuzz/Cargo.toml --no-run --bins --locked
cargo +nightly-2025-11-21 fuzz run requirement_from_tokens -- \
  -runs=1 -artifact_prefix=/tmp/causlane-fuzz-artifacts/
```

The repository-local wrapper form used by the preflight is:

```bash
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 \
  ./tools/cargo +nightly-2025-11-21 test --manifest-path fuzz/Cargo.toml --no-run --bins --locked
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 \
  ./tools/cargo +nightly-2025-11-21 fuzz run requirement_from_tokens -- \
  -runs=1 -artifact_prefix=/tmp/causlane-fuzz-artifacts/
```

The one-run fuzz command is a smoke test for toolchain wiring. It is not the
long-run fuzz budget used for coverage evidence. The preflight checks git
cleanliness again after fuzzing so lockfile or corpus drift cannot be hidden.

## Long Fuzz Runs

Routine protocol fuzz evidence uses a 15-minute budget per protocol target:

```bash
cargo +nightly-2025-11-21 fuzz run replay_trace_json -- \
  -max_total_time=900 -print_final_stats=1
cargo +nightly-2025-11-21 fuzz run replay_scenario_yaml -- \
  -max_total_time=900 -print_final_stats=1
cargo +nightly-2025-11-21 fuzz run registry_yaml_compile -- \
  -max_total_time=900 -print_final_stats=1
```

Repository-local wrapper form:

```bash
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 \
  ./tools/cargo +nightly-2025-11-21 fuzz run replay_trace_json -- \
  -max_total_time=900 -print_final_stats=1
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 \
  ./tools/cargo +nightly-2025-11-21 fuzz run replay_scenario_yaml -- \
  -max_total_time=900 -print_final_stats=1
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 \
  ./tools/cargo +nightly-2025-11-21 fuzz run registry_yaml_compile -- \
  -max_total_time=900 -print_final_stats=1
```

Run those commands from the dispatcher checkout after `tools/ci-dispatcher-preflight`
passes. If a crash/reproducer is produced, commit the curated corpus change and
record the finding in the relevant review or formal-impact document.

## Local Notes

Keep local reality out of git:

```bash
mkdir -p local
$EDITOR local/ci-dispatcher.local.md
```

Use that ignored note for:

- the operational checkout path on this machine;
- installed tool versions and package-manager details;
- known stale or preserved checkouts;
- local artifact directories and logs;
- remediation notes that should not be portable documentation.

Do not put tokens, private keys or service credentials in `local/`. The files
are ignored by git, but they are still plain files and may be included in local
backups.

## Troubleshooting

`remote checkout is not an ancestor of origin/main` means the selected checkout
cannot be fast-forwarded. Use a clean checkout or pass `--repo` pointing at one;
do not repair this by resetting a checkout that may contain local work.

`missing required tool` means the dispatcher image is missing part of the
formal/property/fuzz surface. Install or expose the tool on `PATH`, then rerun
the preflight.

`No such file or directory` for the fuzz artifact prefix means the command was
run manually without creating the artifact directory. The preflight creates its
configured directory automatically.
