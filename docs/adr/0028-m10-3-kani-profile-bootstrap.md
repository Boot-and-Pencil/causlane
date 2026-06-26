# ADR-0028: M10.3 Kani Profile Bootstrap

Date: 2026-06-24

## Status

Accepted

## Context

The Kani lane already runs real generated harnesses under `cargo-kani`, but the
runner shape lived partly as shell literals in `tools/formal-verify-all`. M10.3
needs a foundation for lane-specific bounds and future CI/nightly split without
duplicating Kani invocation policy across scripts and docs.

## Decision

Introduce `formal/kani/profile.json` as the machine-readable source for:

- the generated fixture stem;
- the temporary Kani package name;
- the Kani output format;
- default unwind bounds per formal lane.

`tools/formal-verify-all` reads this profile with `jq` and fails closed for an
unknown lane or missing field. `tools/schema-validate-all` validates the profile
against `contracts/schema/formal_kani_profile.schema.json`.

Add `tools/formal-verify-lane` as the provider-neutral lane entrypoint. It reads
the same profile, validates the requested lane, reports the lane's Kani unwind
and output format in `--dry-run`, and delegates real execution to
`tools/formal-verify-all --lane`. CI providers should call this wrapper instead
of duplicating Kani invocation policy.

## Consequences

- `local_smoke` keeps the existing `--default-unwind 96 --output-format terse`
  behavior.
- `fast_ci`, `nightly` and `manual_deep` have explicit configured bounds before
  any CI workflow split is added.
- The repo now exposes CI/nightly/manual lane commands without checking in a
  provider-specific workflow.
- No new Kani proof semantics or active coverage claims are introduced by this
  bootstrap.
