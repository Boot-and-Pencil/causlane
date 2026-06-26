# Adversarial Audience Publication Review — 2026-06-26

**Status:** human-maintained review evidence, not a semantic authority.

This review used `/tmp/Gemini_toxic.md` as a hostile-audience stress rubric:
assume skeptical readers will attack overclaims, security hygiene, release
process, dependency risk, public optics and documentation drift. The prompt's
tone was not adopted; only the review dimensions were used.

## Confirmed findings

| Finding | Disposition | Current owner |
|---|---|---|
| M11.5 still appeared as `planned` even though release hygiene gates now exist. | Fixed now: product-track source status is `exists_harden`, and the milestone records current evidence plus remaining backlog. | Release track |
| `cargo-deny` existed but was not a hard blocker in the publication plan. | Fixed now: `publication-prep.md` requires the gate to pass and requires warnings to be accepted or tracked. | Release track |
| The Restate runtime path previously reached the RustCrypto RSA advisory chain. | Fixed before this review by switching the Restate backend to `aws_lc_rs`; this review records the evidence to prevent regression. | Runtime adapters |
| `serde_yaml 0.9.34+deprecated` remains in contracts, replay and CLI parser boundaries, and reaches `unsafe-libyaml`. | Deferred with owner: track under M11.5 before YAML-facing crates or full workspace publication. A same-day swap would touch protocol fixtures, scenario parsing and formal/replay surfaces. | Contracts/replay tooling |
| `cargo-deny` reports duplicate-version warnings. | Deferred with owner: convergence backlog before full workspace `0.0.1`; not hidden as a clean dependency story. | Release track |
| Formal attestation tests contain a fixed key-like string. | Mitigated now: the script labels it as synthetic fixed test material. It is not a secret or environment credential. | Formal/replay tooling |

## Non-findings

- The `causlane-core` first-upload dry-run is not affected by `serde_yaml`;
  `causlane-core` has no normal runtime dependencies.
- CLI `println!`/`eprintln!` usage is confined to CLI entrypoints and is not a
  runtime-library logging policy issue.
- `panic!` occurrences observed in the review are confined to tests.
- `todo!`/`unimplemented!` occurrences observed in the review are scanner
  labels inside tooling, not unfinished production logic.

## Required follow-up

- Before publishing YAML-facing crates beyond `causlane-core`, either replace
  the deprecated YAML parser boundary or explicitly accept it in release notes
  as pre-alpha debt.
- Before full workspace `0.0.1`, rerun `./tools/cargo-dev deny check` and record
  whether duplicate-version warnings were eliminated or intentionally accepted.
- Keep future advisory/license/source exceptions narrow and dated; do not add a
  blanket advisory suppression to make a publication gate pass.
