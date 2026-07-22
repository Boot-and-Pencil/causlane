# Proof/refinement scope

This file is generated from
[`proof-refinement-scope.json`](proof-refinement-scope.json) by
`tools/proof-refinement-scope --write`. Do not hand-edit this Markdown;
edit the JSON artifact and regenerate it. This document classifies evidence
strength only; current per-invariant coverage remains authoritative in
[`../invariants/coverage-matrix.json`](../invariants/coverage-matrix.json).

This artifact classifies proof/refinement strength. It does not grant coverage credit; current per-invariant lane status and check_id credit remain receipt-derived in docs/invariants/coverage-matrix.json.

## Evidence Classes

- `proved`: A proof lane has a real, blocking proof run for the stated scope.
- `bounded`: The evidence quantifies a finite or bounded input/state space.
- `simulated`: A model or monitor simulates protocol behavior for generated facts.
- `tested`: Executable checks or negative controls exercise concrete behavior.
- `assumed`: The claim depends on trusted environment, crypto or tool assumptions.
- `out_of_scope`: The repository intentionally does not claim this as current evidence.

## proved

### Verus code-adjacent proofs

- ID: `verus-code-adjacent-proofs`
- Lanes: `verus`
- Claim: Verus proves preservation and rule lemmas over generated Rust-like proof artifacts under the always-on no-cheating profile.
- Authority surface: verification/formal-full/verus/generated plus Verus codegen/tool-run receipts
- Scope: Pure kernel/rule preservation lemmas that are generated from Formal IR and credited by the coverage report.
- Limits: Verus does not by itself prove external adapters, concrete storage, network IO or all parser behavior.
- Verification: verus --no-cheating inside scripts/check-verification-full.sh
- Source links: `verification/formal-full/verus`, `verification/formal-full/proof-lanes.json`, `docs/formal/03-lean4-verus-proof-obligations.md`

### Lean4 abstract theorem applications

- ID: `lean4-abstract-theorem-applications`
- Lanes: `lean4`
- Claim: Lean4 proves generated scenario-bound theorem applications over the abstract protocol vocabulary and generated facts.
- Authority surface: verification/formal-full/lean core package, verification/formal-full/lean4/generated and Lean4 receipts
- Scope: Abstract protocol theorem applications connected to generated Formal IR payloads and credited by the coverage report.
- Limits: Lean4 alone does not claim Rust implementation correctness; that connection comes from replay, Kani and Verus evidence.
- Verification: lake build plus lake env lean inside scripts/check-verification-full.sh
- Source links: `verification/formal-full/lean`, `verification/formal-full/lean4/generated`, `docs/formal/03-lean4-verus-proof-obligations.md`


## bounded

### Kani bounded Rust rules

- ID: `kani-bounded-rust-rules`
- Lanes: `kani`
- Claim: Kani checks Rust-facing pure reducers and validators over bounded nondeterministic input spaces configured by the lane profile.
- Authority surface: verification/formal-full/kani/profile.json plus generated Kani harnesses and tool-run receipts
- Scope: Bounded executable confirmation that selected Rust predicates match intended rules for active coverage cells.
- Limits: Kani is not an unbounded proof and does not cover external IO or adapter behavior outside the harness shape.
- Verification: Kani lane in scripts/check-verification-full.sh and schema validation of verification/formal-full/kani/profile.json
- Source links: `verification/formal-full/kani/profile.json`, `contracts/schema/formal_kani_profile.schema.json`, `docs/adr/0028-m10-3-kani-profile-bootstrap.md`


## simulated

### Alloy structural models

- ID: `alloy-structural-models`
- Lanes: `alloy`
- Claim: Alloy gives small-scope structural counterexample search over generated bundle/scenario facts.
- Authority surface: verification/formal-full/alloy/core plus verification/formal-full/alloy/generated and Alloy tool-run receipts
- Scope: Structural lifecycle, anchor, witness, lease, authz and drain relations where the generated model has facts and assertions.
- Limits: Alloy cells are not evidence for time/freshness, overlay, routing or constraint-update behavior unless the generated model explicitly carries that obligation.
- Verification: AlloyRunner blocks and Alloy negative controls inside scripts/check-verification-full.sh
- Source links: `verification/formal-full/alloy`, `verification/formal-full/tools/AlloyRunner.java`, `docs/invariants/coverage-matrix.json`

### P generated monitors

- ID: `p-generated-monitors`
- Lanes: `p`
- Claim: P checks generated protocol monitors and bounded interleaving controls for modeled event streams.
- Authority surface: verification/formal-full/p/generated and P tool-run receipts
- Scope: Event-order monitors, payload-grounding monitors, drain/race controls and planned interleaving hooks that are explicitly run by the gate.
- Limits: P does not prove Rust implementation paths or lanes not represented by generated monitors.
- Verification: P protocol run and P monitor-firing negative controls inside scripts/check-verification-full.sh
- Source links: `verification/formal-full/p`, `docs/adr/0027-m10-2-p-interleavings-bootstrap.md`, `scripts/check-verification-full.sh`


## tested

### Receipt-bound coverage authority

- ID: `receipt-bound-coverage-authority`
- Lanes: `replay, alloy, p, kani, verus, lean4, tooling`
- Claim: A coverage cell is authoritative only when fresh generated artifacts, tool-run receipts, stale-check and the derived coverage report agree.
- Authority surface: target/causlane/formal-coverage-report.json plus docs/invariants/coverage-matrix.json
- Scope: Per-lane status, check_id credit and covered/not_applicable cells for active invariants.
- Limits: This scope artifact describes evidence strength only; it does not create or upgrade coverage.
- Verification: scripts/check-verification-full.sh and tools/coverage-matrix --check
- Source links: `docs/invariants/coverage-matrix.json`, `docs/invariants/coverage-matrix.md`, `tools/coverage-matrix`

### Replay concrete oracle

- ID: `replay-concrete-oracle`
- Lanes: `replay`
- Claim: Replay checks concrete bundle-bound traces and refutes invalid scenarios with stable error codes.
- Authority surface: causlane-replay plus contracts/scenarios negative controls
- Scope: Concrete trace semantics, payload binding, authz freshness/policy checks and executable negative controls.
- Limits: Replay is not an exhaustive interleaving or abstract proof of every possible runtime execution.
- Verification: scripts/check-verification-full.sh replay prerequisite and replay negative-control blocks
- Source links: `crates/causlane-replay/src`, `contracts/scenarios`, `docs/formal-readiness-status.md`


## assumed

### Cryptographic and toolchain assumptions

- ID: `cryptographic-and-toolchain-assumptions`
- Lanes: `tooling`
- Claim: Hash collision resistance, signature primitive correctness and external solver/compiler correctness are trusted assumptions, not repository proofs.
- Authority surface: toolchain pins, HMAC/SHA tests and typed cli-checker readiness checks
- Scope: Environmental and cryptographic assumptions needed for the evidence chain to be meaningful.
- Limits: The repository tests deterministic use of primitives but does not prove cryptographic hardness or solver soundness.
- Verification: cli-checker project formal doctor, tools/formal-install, unit tests and pinned tool versions
- Source links: `.devinfra/tool-versions.json`, `.devinfra/cli-checker/project-tooling-profile.yaml`, `docs/setup.md`


## out_of_scope

### External provider CI adoption

- ID: `external-provider-ci-adoption`
- Lanes: `tooling`
- Claim: Provider-specific PR CI wiring is outside the repository proof contour until a workflow supplies the baseline and changed-file source.
- Authority surface: repository-local gates and formal-discipline-check entrypoints
- Scope: Local and repo-gate enforcement exists; external provider adoption remains integration work.
- Limits: No checked-in provider workflow is claimed by this milestone.
- Verification: tools/formal-discipline-check supports --from-git and --changed-files
- Source links: `docs/formal/07-integration-tz.md`, `tools/formal-discipline-check`, `tools/specs/formal-discipline-check.md`

### Future invariants and unbounded IO

- ID: `future-invariants-and-unbounded-io`
- Lanes: `replay, alloy, p, kani, verus, lean4`
- Claim: Planned invariants, deeper refinement theorems, unbounded parser/IO behavior and external adapter semantics are future obligations until generated artifacts and receipts exist.
- Authority surface: formal catalogs, obligation manifest and product-track future milestones
- Scope: Target-state formal surface that is documented but not current coverage.
- Limits: Catalog mentions are not coverage and must remain out of current proof claims until the coverage matrix credits them.
- Verification: formal-discipline-check, coverage-matrix drift check and generated proof/refinement scope check
- Source links: `docs/formal/01-formal-model-catalog.md`, `docs/formal/02-protocol-catalog.md`, `verification/formal-full/obligations/lifecycle_product_obligations.yaml`
