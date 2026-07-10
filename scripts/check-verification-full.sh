#!/usr/bin/env bash
# Formal models/contracts acceptance gate.
set -euo pipefail
cd "$(dirname "$0")/.."

REPO_ROOT="$(pwd)"
CLI=(./tools/cargo-dev run -q -p causlane-cli --bin causlane --)
FORMAL=(./tools/cargo-dev run -q -p causlane-cli --bin causlane-formal --)
REGISTRY="contracts/examples/release_promote.registry.yaml"
TRACE="contracts/examples/release_promote.trace.json"
SCENARIO="contracts/scenarios/release_promote_success.scenario.yaml"
TARGET_DIR="target/causlane"
BUNDLE="$TARGET_DIR/release_promote.bundle.json"
EMITTED_TRACE="$TARGET_DIR/release_promote_success.trace.json"
COVERAGE="$TARGET_DIR/formal-coverage-report.json"
ALLOY_GENERATED="verification/formal-full/alloy/generated/release_promote_success.als"
ALLOY_RUN_RECEIPT="verification/formal-full/receipts/release_promote_success.alloy.tool-run.json"
P_GENERATED="verification/formal-full/p/generated/release_promote_success.p"
P_RUN_RECEIPT="verification/formal-full/receipts/release_promote_success.p.tool-run.json"
KANI_PROFILE="verification/formal-full/kani/profile.json"
VERUS_GENERATED="verification/formal-full/verus/generated/release_promote_success.rs"
VERUS_RUN_RECEIPT="verification/formal-full/receipts/release_promote_success.verus.tool-run.json"
LEAN4_GENERATED="verification/formal-full/lean4/generated/release_promote_success.lean"
LEAN4_RUN_RECEIPT="verification/formal-full/receipts/release_promote_success.lean4.tool-run.json"
SUITE="${VERIFICATION_SUITE:-all}"
PROFILE="${FORMAL_PROFILE:-all}"
DEPTH="${VERIFICATION_DEPTH:-${FORMAL_LANE:-fast_ci}}"
FUZZ_TOOLCHAIN="${FUZZ_TOOLCHAIN:-nightly-2025-11-21}"
FUZZ_TARGET="${FUZZ_TARGET:-runtime_guarded_audit_projection}"
FUZZ_RUNS="${FUZZ_RUNS:-1}"

usage() {
  cat >&2 <<'EOF'
usage: scripts/check-verification-full.sh [--suite all|formal|property|fuzz] [--profile base|rust|proof|all] [--depth local_smoke|fast_ci|nightly|manual_deep] [--fuzz-target TARGET] [--fuzz-runs N]
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --suite)
      SUITE="${2:?missing value for --suite}"
      shift 2
      ;;
    --profile)
      PROFILE="${2:?missing value for --profile}"
      shift 2
      ;;
    --depth|--lane)
      DEPTH="${2:?missing value for --depth}"
      shift 2
      ;;
    --fuzz-target)
      FUZZ_TARGET="${2:?missing value for --fuzz-target}"
      shift 2
      ;;
    --fuzz-runs)
      FUZZ_RUNS="${2:?missing value for --fuzz-runs}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "check-verification-full: unknown argument $1" >&2
      usage
      exit 2
      ;;
  esac
done

case "$SUITE" in
  all|formal|property|fuzz) ;;
  *) echo "check-verification-full: unsupported suite $SUITE" >&2; exit 2 ;;
esac
case "$FUZZ_RUNS" in
  ''|*[!0-9]*) echo "check-verification-full: --fuzz-runs must be a non-negative integer" >&2; exit 2 ;;
esac
LANE="$DEPTH"

run_property() {
  echo "== property tests =="
  ./tools/cargo-dev test -p causlane-core --test proptest_protocol_properties --locked
  ./tools/cargo-dev test -p causlane-replay --test proptest_parse_boundaries --locked
}

run_fuzz() {
  echo "== fuzz compile =="
  rustup run "$FUZZ_TOOLCHAIN" cargo test --manifest-path verification/fuzz/Cargo.toml --no-run --bins --locked
  echo "== fuzz one-run smoke =="
  local artifact_dir="$TARGET_DIR/fuzz-artifacts/$FUZZ_TARGET"
  mkdir -p "$artifact_dir"
  rustup run "$FUZZ_TOOLCHAIN" cargo fuzz run "$FUZZ_TARGET" --fuzz-dir verification/fuzz -- \
    -runs="$FUZZ_RUNS" -artifact_prefix="${artifact_dir%/}/"
}

run_formal() {
KANI_FIXTURE_STEM="$(jq -er '.fixture_stem' "$KANI_PROFILE")"
KANI_PACKAGE_NAME="$(jq -er '.package_name' "$KANI_PROFILE")"
KANI_OUTPUT_FORMAT="$(jq -er '.output_format' "$KANI_PROFILE")"
KANI_DEFAULT_UNWIND="$(jq -er --arg lane "$LANE" '.lanes[$lane].default_unwind' "$KANI_PROFILE")"
KANI_GENERATED="verification/formal-full/kani/generated/${KANI_FIXTURE_STEM}.rs"
KANI_RUN_RECEIPT="verification/formal-full/receipts/${KANI_FIXTURE_STEM}.kani.tool-run.json"

config_tool_dir() {
  local query="$1"
  local path dir
  path="$(jq -r "$query // empty" .devinfra/tool-versions.json)"
  [ -n "$path" ] || return 0
  case "$path" in
    /*) dir="$(dirname "$path")" ;;
    */*) dir="$(dirname "$REPO_ROOT/$path")" ;;
    *) return 0 ;;
  esac
  [ -d "$dir" ] || return 0
  printf '%s\n' "$dir"
}

prepend_config_tool_dir() {
  local dir
  dir="$(config_tool_dir "$1")"
  [ -n "$dir" ] || return 0
  PATH="$dir:$PATH"
}

prepend_config_tool_dir '.tools.formal_tools.verus.archive_path'
prepend_config_tool_dir '.tools.formal_tools.z3.binary'
export PATH

# Proof lanes (Verus + Lean4) are ALWAYS run. The proof-lane
# reality exceptions were dropped (2026-06-21): there is no non-blocking skip,
# so this gate proves on every run and the full toolchain is always required.
# The fast dev loop lives in `formal-ready` / `cargo test` / `clippy`, not here.
RUN_PROOF=1
case "$PROFILE" in
  base|rust|proof|all)
    ;;
  *)
    echo "check-verification-full: unsupported profile $PROFILE" >&2
    exit 2
    ;;
esac

mkdir -p "$TARGET_DIR" verification/formal-full/receipts

echo "== formal doctor (profile=$PROFILE lane=$LANE) =="
cli-checker project formal doctor \
  --profile .devinfra/cli-checker/project-tooling-profile.yaml \
  --require all --format json > "$TARGET_DIR/formal-doctor.$PROFILE.$LANE.json"

echo "== formal exceptions policy (P1-FM-012) =="
# Fail fast on any expired lane exception, regardless of profile.
tools/formal-exceptions-check --profile "$PROFILE"

# Record the REAL outcome of a tool run into its receipt: the parsed result
# token AND the real process exit code. We never hardcode "pass" — the derived
# coverage report (P0-FM-002) computes status from these receipts, and a
# non-zero exit code can never be upgraded to pass.
update_tool_receipt() {
  local receipt="$1"
  local tool="$2"
  local version="$3"
  local command="$4"
  local result="$5"
  local exit_code="$6"
  local tmp
  tmp="$(mktemp)"
  jq \
    --arg tool "$tool" \
    --arg version "$version" \
    --arg command "$command" \
    --arg result "$result" \
    --argjson exit_code "$exit_code" \
    '.tool = $tool | .tool_version = $version | .command = $command | .actual_result = $result | .exit_code = $exit_code' \
    "$receipt" > "$tmp"
  mv "$tmp" "$receipt"
}

echo "== rust formal build checks =="
./tools/cargo-dev fmt --all --check
./tools/cargo-dev clippy --workspace --all-targets --all-features --locked -- -D warnings
./tools/cargo-dev check --workspace --all-targets --all-features --locked --keep-going
./tools/cargo-dev test -p causlane-codegen --all-targets --all-features --locked

echo "== schema validation (P0-008: drift fails closed, before any generation) =="
./tools/schema-validate-all

echo "== bundle/replay prerequisite =="
"${CLI[@]}" bundle compile --registry "$REGISTRY" --out "$BUNDLE"
"${CLI[@]}" replay verify --bundle "$BUNDLE" --trace "$TRACE"
# Emit the scenario trace bound to the compiled bundle, then verify it strictly
# (--require-bundle-hash) so the emitted trace is provably tied to this bundle in
# the authority chain rather than accepted unbound (P0-005).
"${CLI[@]}" scenario emit-trace --scenario "$SCENARIO" --bundle "$BUNDLE" --out "$EMITTED_TRACE"
"${CLI[@]}" replay verify --bundle "$BUNDLE" --trace "$EMITTED_TRACE" --require-bundle-hash

echo "== multi-action reference (P2-004): replay-valid two-action history =="
# A non-RuntimeExecution (EvidenceMeta) bundle carrying ONE trace with TWO
# independent actions. RuntimeExecution requires a barrier ceremony per action, so
# a minimal multi-action trace cannot replay-validate against it; the meta
# lifecycle has no such obligation and each action's lifecycle is reduced on its
# own substream (per-action validate_lifecycle).
MULTI_REGISTRY="contracts/examples/multi_action_reference.registry.yaml"
MULTI_SCENARIO="contracts/scenarios/multi_action_reference.scenario.yaml"
MULTI_BUNDLE="$TARGET_DIR/multi_action_reference.bundle.json"
MULTI_TRACE="$TARGET_DIR/multi_action_reference.trace.json"
"${CLI[@]}" bundle compile --registry "$MULTI_REGISTRY" --out "$MULTI_BUNDLE"
"${CLI[@]}" scenario emit-trace --scenario "$MULTI_SCENARIO" --bundle "$MULTI_BUNDLE" --out "$MULTI_TRACE"
"${CLI[@]}" replay verify --bundle "$MULTI_BUNDLE" --trace "$MULTI_TRACE" --require-bundle-hash

echo "== attestation gate (P1-006): kernel-secret-attested verify accepts valid, refutes missing/wrong =="
# A minted capability attestation passes attested verify; a missing one (no secret
# at emit time) and a wrong one (minted under a different secret) must refute with
# CapabilityMismatch. Proves the attested path is non-vacuous and the CLI flag works.
# Synthetic fixed test material only; this is not an environment credential.
ATTESTATION_TEST_KEY_MATERIAL="causlane-test-kernel-secret"
ATTEST_OK="$TARGET_DIR/attested_ok.trace.json"
ATTEST_MISSING="$TARGET_DIR/attested_missing.trace.json"
ATTEST_WRONG="$TARGET_DIR/attested_wrong.trace.json"
"${CLI[@]}" scenario emit-trace --scenario "$SCENARIO" --bundle "$BUNDLE" --kernel-secret "$ATTESTATION_TEST_KEY_MATERIAL" --out "$ATTEST_OK" >/dev/null
"${CLI[@]}" replay verify --bundle "$BUNDLE" --trace "$ATTEST_OK" --kernel-secret "$ATTESTATION_TEST_KEY_MATERIAL" >/dev/null
echo "  attestation: minted capability passes attested verify"
"${CLI[@]}" scenario emit-trace --scenario "$SCENARIO" --bundle "$BUNDLE" --out "$ATTEST_MISSING" >/dev/null
"${CLI[@]}" scenario emit-trace --scenario "$SCENARIO" --bundle "$BUNDLE" --kernel-secret "a-different-secret" --out "$ATTEST_WRONG" >/dev/null
for t in "$ATTEST_MISSING" "$ATTEST_WRONG"; do
  set +e
  out="$("${CLI[@]}" replay verify --bundle "$BUNDLE" --trace "$t" --kernel-secret "$ATTESTATION_TEST_KEY_MATERIAL" 2>&1)"; rc=$?
  set -e
  case "$out" in
    *CapabilityMismatch*) echo "  attestation: $(basename "$t") refuted (CapabilityMismatch)";;
    *) echo "check-verification-full: FAILED (attested verify accepted bad attestation: $out)" >&2; exit 1;;
  esac
  [ "$rc" -ne 0 ] || { echo "check-verification-full: FAILED (attested verify did not reject $(basename "$t"))" >&2; exit 1; }
done

echo "== authz-required slice (P0-010): replay accepts a fresh bound Allow, refutes each authz defect =="
# A dedicated authz-required bundle (the main slice runs authz disabled). The
# success scenario records an Allow decision the barrier references; each control
# mutates one authz fact and must be refuted with its exact stable code.
AUTHZ_BUNDLE="$TARGET_DIR/release_promote_authz.bundle.json"
"${CLI[@]}" bundle compile --registry contracts/examples/release_promote_authz.registry.yaml --out "$AUTHZ_BUNDLE"
authz_verify() {
  local scen="$1" expect="$2"
  local trace="$TARGET_DIR/authz-$(basename "$scen" .scenario.yaml).trace.json"
  "${CLI[@]}" scenario emit-trace --scenario "$scen" --bundle "$AUTHZ_BUNDLE" --out "$trace" >/dev/null
  if [ "$expect" = "pass" ]; then
    "${CLI[@]}" replay verify --bundle "$AUTHZ_BUNDLE" --trace "$trace" --require-bundle-hash >/dev/null
    echo "  authz success: $(basename "$scen" .scenario.yaml) verified"
    return
  fi
  set +e
  local out rc
  out="$("${CLI[@]}" replay verify --bundle "$AUTHZ_BUNDLE" --trace "$trace" --require-bundle-hash 2>&1)"
  rc=$?
  set -e
  if [ "$rc" -eq 0 ] || ! printf '%s' "$out" | grep -q "$expect"; then
    echo "check-verification-full: FAILED (authz control $(basename "$scen") expected $expect; rc=$rc: $out)" >&2
    exit 1
  fi
  echo "  authz control: $(basename "$scen" .scenario.yaml) => $expect (refuted)"
}
authz_verify contracts/scenarios/authz/authz_success.scenario.yaml pass
authz_verify contracts/scenarios/authz/authz_denied_invalid.scenario.yaml AuthzDecisionDenied
authz_verify contracts/scenarios/authz/authz_missing_invalid.scenario.yaml AuthzDecisionMissing
authz_verify contracts/scenarios/authz/authz_expired_invalid.scenario.yaml AuthzDecisionExpired
authz_verify contracts/scenarios/authz/authz_wrong_policy_invalid.scenario.yaml AuthzPolicyMismatch
authz_verify contracts/scenarios/authz/authz_issued_after_barrier_invalid.scenario.yaml AuthzIssuedAfterBarrier
authz_verify contracts/scenarios/authz/authz_stale_invalid.scenario.yaml AuthzDecisionStale

echo "== formal target generation and receipts =="
"${FORMAL[@]}" verify-all \
  --bundle "$BUNDLE" \
  --scenario "$SCENARIO" \
  --artifact-dir verification/formal-full \
  --receipt-dir verification/formal-full/receipts \
  --coverage "$COVERAGE"

echo "== Alloy tool run (REAL, discriminating) =="
# Alloy is the one generated lane that runs a real, discriminating check. We
# parse AlloyRunner's JSON status and record the real exit code; the derived
# coverage report decides pass/fail (no early greening).
if [ ! -f .tools/alloy/classes/AlloyRunner.class ] || [ verification/formal-full/tools/AlloyRunner.java -nt .tools/alloy/classes/AlloyRunner.class ]; then
  mkdir -p .tools/alloy/classes
  javac -cp .tools/alloy/alloy.jar -d .tools/alloy/classes verification/formal-full/tools/AlloyRunner.java
fi
ALLOY_VERSION="$(jq -r '.tools.formal_tools.alloy.version' .devinfra/tool-versions.json)"
set +e
ALLOY_OUT="$(java -cp .tools/alloy/alloy.jar:.tools/alloy/classes AlloyRunner "$ALLOY_GENERATED")"
ALLOY_RC=$?
set -e
ALLOY_STATUS="$(printf '%s' "$ALLOY_OUT" | jq -r '.status // "fail"')"
ALLOY_RESULT="fail"
if [ "$ALLOY_RC" -eq 0 ] && [ "$ALLOY_STATUS" = "pass" ]; then ALLOY_RESULT="pass"; fi
update_tool_receipt "$ALLOY_RUN_RECEIPT" "alloy" "$ALLOY_VERSION" "java -cp .tools/alloy/alloy.jar:.tools/alloy/classes AlloyRunner $ALLOY_GENERATED" "$ALLOY_RESULT" "$ALLOY_RC"
echo "  alloy: AlloyRunner status=$ALLOY_STATUS rc=$ALLOY_RC -> $ALLOY_RESULT"

echo "== Alloy negative controls (payload-bound discrimination must refute, I-001/I-003/I-004/I-006/I-009) =="
# The payload-bound Alloy model must REFUTE each forged scenario (status=fail).
# A pass here means the discriminating check is vacuous — that is a gate failure.
assert_alloy_refutes() {
  local scenario="$1"
  local bundle="${2:-$BUNDLE}"
  local stem
  stem="$(basename "$scenario" .scenario.yaml)"
  local als="verification/formal-full/alloy/generated/${stem}.als"
  "${CLI[@]}" formal generate alloy --bundle "$bundle" --scenario "$scenario" --out "$als" >/dev/null
  set +e
  local out
  out="$(java -cp .tools/alloy/alloy.jar:.tools/alloy/classes AlloyRunner "$als")"
  set -e
  local status
  status="$(printf '%s' "$out" | jq -r '.status // "pass"')"
  echo "  alloy negative control: $stem status=$status (expect fail)"
  if [ "$status" != "fail" ]; then
    echo "check-verification-full: FAILED (Alloy did not refute $stem; status=$status)" >&2
    exit 1
  fi
}
assert_alloy_refutes contracts/scenarios/forged_capability_invalid.scenario.yaml
assert_alloy_refutes contracts/scenarios/approval_wrong_impact_invalid.scenario.yaml
# I-009 plan binding (P0-FM-004): a gate-approval witness that binds the WRONG
# plan_hash must be refuted (GeneratedApprovalBindingHolds now checks wbPlan).
assert_alloy_refutes contracts/scenarios/approval_wrong_plan_invalid.scenario.yaml
# I-006 interval-aware leases (P0-FM-004): two exclusive leases whose active
# windows overlap on the same resource+scope must be refuted.
assert_alloy_refutes contracts/scenarios/conflicting_leases_invalid.scenario.yaml
# I-003 projection anchor: a projection without a prior observed-truth anchor.
assert_alloy_refutes contracts/scenarios/projection_without_anchor_invalid.scenario.yaml
# P0-004 producer attestation (Formal IR v2): a witness/anchor that claims a
# fact_kind/scope its producer/observed event did not attest must be refuted
# (GeneratedWitnessFactGrounded / GeneratedAnchorFactGrounded ground the ref in
# the producer event's own attestation, the same grounding replay does).
assert_alloy_refutes contracts/scenarios/witness_event_wrong_fact_kind_invalid.scenario.yaml
assert_alloy_refutes contracts/scenarios/witness_wrong_scope_invalid.scenario.yaml
assert_alloy_refutes contracts/scenarios/projection_anchor_wrong_fact_invalid.scenario.yaml
assert_alloy_refutes contracts/scenarios/projection_anchor_wrong_scope_invalid.scenario.yaml
# I-008 closed-terminal: an event after an action's LifecycleClosed must be refuted
# (the base-model `Enforced` now includes the ClosedIsTerminal clause).
assert_alloy_refutes contracts/scenarios/event_after_closed_invalid.scenario.yaml
# I-007 drain-fence (structural): a drain over a scope with an active overlapping
# exclusive lease must be refuted (GeneratedDrainFenceClear). Alloy has no time, so
# the expiry refinement stays the replay oracle's job.
assert_alloy_refutes contracts/scenarios/drain_with_active_lease_invalid.scenario.yaml
# I-006 / P0-005: on a bundle with a VERIFIED, APPLICABLE merge protocol the
# `mergeable` relation is non-empty (here `{environment:staging}`), but a conflict
# on an UNRELATED scope (`queue:deploy`) must STILL refute — the merge relaxation is
# per-scope, not a global disable. The old `some mergeable or ...` assertion would
# have wrongly accepted this (verified manually). mergeable is non-empty here.
MERGE_BUNDLE="$TARGET_DIR/release_promote_merge.bundle.json"
"${CLI[@]}" bundle compile --registry contracts/examples/release_promote_merge.registry.yaml --out "$MERGE_BUNDLE" >/dev/null
assert_alloy_refutes contracts/scenarios/unrelated_merge_protocol_does_not_allow_conflict_invalid.scenario.yaml "$MERGE_BUNDLE"

echo "== authz Alloy grounding (P0-010): structural Allow+binding (refutes denied/missing) =="
# Alloy grounds the STRUCTURAL authz (a barrier must reference a bound Allow);
# the policy/temporal defects (wrong-policy/expired/stale/issued-after) are the
# replay oracle's domain above. The authz bundle was compiled in the authz slice.
authz_alloy() {
  local scen="$1" expect="$2"
  local als="verification/formal-full/alloy/generated/$(basename "$scen" .scenario.yaml).als"
  "${CLI[@]}" formal generate alloy --bundle "$AUTHZ_BUNDLE" --scenario "$scen" --out "$als" >/dev/null
  set +e
  local out
  out="$(java -cp .tools/alloy/alloy.jar:.tools/alloy/classes AlloyRunner "$als")"
  set -e
  local status
  status="$(printf '%s' "$out" | jq -r '.status // "fail"')"
  if [ "$status" != "$expect" ]; then
    echo "check-verification-full: FAILED (authz alloy $(basename "$scen") status=$status, expected $expect)" >&2
    exit 1
  fi
  echo "  authz alloy: $(basename "$scen" .scenario.yaml) status=$status (expect $expect)"
}
authz_alloy contracts/scenarios/authz/authz_success.scenario.yaml pass
authz_alloy contracts/scenarios/authz/authz_denied_invalid.scenario.yaml fail
authz_alloy contracts/scenarios/authz/authz_missing_invalid.scenario.yaml fail

echo "== P protocol run (REAL monitors) =="
P_RUN_DIR="$TARGET_DIR/formal-runs/p/release_promote_success"
mkdir -p "$P_RUN_DIR"
P_VERSION="$(p --version | sed -n '1p')"
set +e
(cd "$P_RUN_DIR" \
  && p compile --pfiles "$REPO_ROOT/$P_GENERATED" --projname release_promote_success --outdir out >/dev/null \
  && p check out/PChecker/net8.0/release_promote_success.dll --testcase release_promote_generated --schedules 20 --max-steps 200 >/dev/null)
P_RC=$?
set -e
P_RESULT="fail"
[ "$P_RC" -eq 0 ] && P_RESULT="pass"
update_tool_receipt "$P_RUN_RECEIPT" "p" "$P_VERSION" "(cd $P_RUN_DIR && p compile --pfiles $REPO_ROOT/$P_GENERATED --projname release_promote_success --outdir out && p check out/PChecker/net8.0/release_promote_success.dll --testcase release_promote_generated --schedules 20 --max-steps 200)" "$P_RESULT" "$P_RC"
echo "  p: rc=$P_RC -> $P_RESULT"

echo "== P multi-action interleaving (P1-001 part 3): action-sharded drivers =="
# The action-sharded codegen emits one driver machine per action, created by a
# bootstrap, so P's scheduler interleaves the two actions' event streams (the old
# single ScenarioDriver could not interleave anything). The keyed monitors hold
# across every interleaving; the cross-action negative control below proves the
# keying is load-bearing (a flat monitor would be fooled).
MULTI_P="verification/formal-full/p/generated/multi_action_reference.p"
MULTI_P_DIR="$TARGET_DIR/formal-runs/p/multi_action_reference"
mkdir -p "$MULTI_P_DIR"
"${CLI[@]}" formal generate p --bundle "$MULTI_BUNDLE" --scenario "$MULTI_SCENARIO" --out "$MULTI_P" >/dev/null
(cd "$MULTI_P_DIR" && p compile --pfiles "$REPO_ROOT/$MULTI_P" --projname multi_action_reference --outdir out >/dev/null)
(cd "$MULTI_P_DIR" && p check "out/PChecker/net8.0/multi_action_reference.dll" --testcase release_promote_generated --schedules 20 --max-steps 200 >/dev/null)
echo "  p multi-action: two action drivers interleave; keyed monitors hold"

echo "== P negative controls (payload-bound monitors must fire, I-001/P0-004) =="
# The payload-bound P model must REFUTE each forged scenario: it compiles, but
# `p check` finds the monitor violation (non-zero exit). A passing check would
# mean the monitor is vacuous — that is a gate failure.
assert_p_refutes() {
  local scenario="$1" bundle="${2:-$BUNDLE}" stem
  stem="$(basename "$scenario" .scenario.yaml)"
  local pf="verification/formal-full/p/generated/${stem}.p"
  local dir="$TARGET_DIR/formal-runs/p/${stem}"
  mkdir -p "$dir"
  "${CLI[@]}" formal generate p --bundle "$bundle" --scenario "$scenario" --out "$pf" >/dev/null
  (cd "$dir" && p compile --pfiles "$REPO_ROOT/$pf" --projname "$stem" --outdir out >/dev/null)
  set +e
  (cd "$dir" && p check "out/PChecker/net8.0/${stem}.dll" --testcase release_promote_generated --schedules 20 --max-steps 200 >/dev/null)
  local rc=$?
  set -e
  echo "  p negative control: $stem p-check rc=$rc (expect non-zero = monitor fired)"
  if [ "$rc" -eq 0 ]; then
    echo "check-verification-full: FAILED (P did not refute $stem; p check passed — monitor vacuous)" >&2
    exit 1
  fi
}
# I-001 forged capability (CapabilityBindsToBarrier).
assert_p_refutes contracts/scenarios/forged_capability_invalid.scenario.yaml
# P0-004 producer attestation (WitnessFactGrounded / AnchorFactGrounded): a
# witness/anchor that claims a fact_kind/scope its producer/observed event did
# not attest must be refuted, the same grounding replay and Alloy do.
assert_p_refutes contracts/scenarios/witness_event_wrong_fact_kind_invalid.scenario.yaml
assert_p_refutes contracts/scenarios/witness_wrong_scope_invalid.scenario.yaml
assert_p_refutes contracts/scenarios/projection_anchor_wrong_fact_invalid.scenario.yaml
assert_p_refutes contracts/scenarios/projection_anchor_wrong_scope_invalid.scenario.yaml
# P1-001 keyed lifecycle monitors (interleaving lane): the lifecycle-ordering
# monitors are now keyed by action, so each refutes its own ordering violation.
assert_p_refutes contracts/scenarios/execution_without_barrier_invalid.scenario.yaml
assert_p_refutes contracts/scenarios/observed_without_execution_invalid.scenario.yaml
assert_p_refutes contracts/scenarios/projection_without_anchor_invalid.scenario.yaml
assert_p_refutes contracts/scenarios/event_after_closed_invalid.scenario.yaml
# P1-001 keyed lease/drain monitors (interleaving lane): NoConflictingActiveLeases
# keys by lease scope and DrainBlocks keys by scope (lease events are expanded one
# send per lease). conflicting_leases_invalid grants two exclusive leases on the
# SAME scope and refutes, while release_promote_success holds two on DIFFERENT
# scopes and still passes (the positive P check above) — proving the key is
# load-bearing: a flat boolean would refute the success case once expanded.
# lease_during_drain_invalid grants a lease on a draining scope and refutes.
assert_p_refutes contracts/scenarios/conflicting_leases_invalid.scenario.yaml
assert_p_refutes contracts/scenarios/lease_during_drain_invalid.scenario.yaml
# P1-001 part 3 cross-action keying (interleaving lane): act_y executes by riding
# act_x's barrier. A flat monitor that only asked "was SOME barrier seen" would be
# fooled when act_x's barrier is observed first; the action-keyed
# NoExecutionBeforeBarrier / barrier-keyed CapabilityBindsToBarrier refute it.
assert_p_refutes contracts/scenarios/multi_action_cross_action_barrier_invalid.scenario.yaml

echo "== P depth controls (planned I-012/I-014/I-018 evidence hooks) =="
# These controls are intentionally NOT named *_invalid.scenario.yaml: they are
# P-only depth checks for planned invariants and must not be counted as replay
# negative controls or active coverage credit.
assert_p_refutes contracts/scenarios/p_controls/retry_duplicate_execution_control.scenario.yaml
assert_p_refutes contracts/scenarios/p_controls/authz_revoked_old_allow_control.scenario.yaml "$AUTHZ_BUNDLE"
assert_p_refutes contracts/scenarios/p_controls/constraint_epoch_regression_control.scenario.yaml

echo "== authz P grounding (P0-010): AuthzDecisionGroundsBarrier refutes a referenced Deny =="
# authz_success must pass (an Allow grounds the barrier); authz_denied must fire.
AUTHZ_P="verification/formal-full/p/generated/authz_success.p"
AUTHZ_P_DIR="$TARGET_DIR/formal-runs/p/authz_success"
mkdir -p "$AUTHZ_P_DIR"
"${CLI[@]}" formal generate p --bundle "$AUTHZ_BUNDLE" --scenario contracts/scenarios/authz/authz_success.scenario.yaml --out "$AUTHZ_P" >/dev/null
(cd "$AUTHZ_P_DIR" && p compile --pfiles "$REPO_ROOT/$AUTHZ_P" --projname authz_success --outdir out >/dev/null)
(cd "$AUTHZ_P_DIR" && p check out/PChecker/net8.0/authz_success.dll --testcase release_promote_generated --schedules 20 --max-steps 200 >/dev/null)
echo "  authz p: authz_success p-check passed"
assert_p_refutes contracts/scenarios/authz/authz_denied_invalid.scenario.yaml "$AUTHZ_BUNDLE"

echo "== Kani bounded Rust harnesses (REAL) =="
KANI_RUN_DIR="$TARGET_DIR/formal-runs/kani/$KANI_FIXTURE_STEM"
mkdir -p "$KANI_RUN_DIR"
cat > "$KANI_RUN_DIR/Cargo.toml" <<TOML
[package]
name = "$KANI_PACKAGE_NAME"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
path = "$REPO_ROOT/$KANI_GENERATED"

[dependencies]
causlane-core = { path = "$REPO_ROOT/crates/causlane-core" }

[workspace]
TOML
KANI_VERSION="$(cargo-kani --version)"
set +e
env -u RUSTC_WRAPPER -u CARGO_BUILD_RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS \
  cargo-kani --manifest-path "$KANI_RUN_DIR/Cargo.toml" --default-unwind "$KANI_DEFAULT_UNWIND" --output-format "$KANI_OUTPUT_FORMAT" >/dev/null
KANI_RC=$?
set -e
KANI_RESULT="fail"
[ "$KANI_RC" -eq 0 ] && KANI_RESULT="pass"
update_tool_receipt "$KANI_RUN_RECEIPT" "cargo-kani" "$KANI_VERSION" "env -u RUSTC_WRAPPER -u CARGO_BUILD_RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS cargo-kani --manifest-path $KANI_RUN_DIR/Cargo.toml --default-unwind $KANI_DEFAULT_UNWIND --output-format $KANI_OUTPUT_FORMAT" "$KANI_RESULT" "$KANI_RC"
echo "  kani: rc=$KANI_RC -> $KANI_RESULT"

echo "== Verus abstract preservation proofs (always-on, blocking) =="
VERUS_RUN_DIR="$TARGET_DIR/formal-runs/verus/release_promote_success"
mkdir -p "$VERUS_RUN_DIR"
if [ "$RUN_PROOF" = "1" ]; then
  if verus --version >/dev/null 2>&1; then
    set +e
    (cd "$VERUS_RUN_DIR" && verus "$REPO_ROOT/$VERUS_GENERATED" --no-cheating >/dev/null)
    VERUS_RC=$?
    set -e
    VERUS_VERSION="$(verus --version | sed -n 's/^  Version: //p')"
    if [ -z "$VERUS_VERSION" ]; then
      VERUS_VERSION="$(verus --version | sed -n '1p')"
    fi
    VERUS_RESULT="fail"
    [ "$VERUS_RC" -eq 0 ] && VERUS_RESULT="pass"
    update_tool_receipt "$VERUS_RUN_RECEIPT" "verus" "$VERUS_VERSION" "(cd $VERUS_RUN_DIR && verus $REPO_ROOT/$VERUS_GENERATED --no-cheating)" "$VERUS_RESULT" "$VERUS_RC"
    echo "  verus: rc=$VERUS_RC -> $VERUS_RESULT"
  else
    echo "check-verification-full: FAILED (verus unavailable; proof lane is always-on — run tools/formal-install verus)" >&2
    exit 1
  fi
fi

echo "== Lean4 theorem applications (always-on, blocking) =="
LEAN4_RUN_DIR="$TARGET_DIR/formal-runs/lean4/release_promote_success"
mkdir -p "$LEAN4_RUN_DIR"
if grep -R -n -E '(^|[^[:alnum:]_])(sorry|admit|axiom)([^[:alnum:]_]|$)' verification/formal-full/lean/CauslaneFormal verification/formal-full/lean4/generated; then
  echo "check-verification-full: FAILED (Lean4 artifacts must not contain sorry/admit/axiom)" >&2
  exit 1
fi
if [ "$RUN_PROOF" = "1" ]; then
  if tools/lean4-env lean --version >/dev/null 2>&1 && tools/lean4-env lake --version >/dev/null 2>&1; then
    set +e
    (cd verification/formal-full/lean \
      && ../../../tools/lean4-env lake build CauslaneFormal >/dev/null \
      && ../../../tools/lean4-env lake env lean "$REPO_ROOT/$LEAN4_GENERATED" >/dev/null)
    LEAN4_RC=$?
    set -e
    LEAN4_VERSION="$(tools/lean4-env lean --version | sed -n '1p')"
    LEAN4_RESULT="fail"
    [ "$LEAN4_RC" -eq 0 ] && LEAN4_RESULT="pass"
    update_tool_receipt "$LEAN4_RUN_RECEIPT" "lean4" "$LEAN4_VERSION" "(cd verification/formal-full/lean && ../../../tools/lean4-env lake build CauslaneFormal && ../../../tools/lean4-env lake env lean $REPO_ROOT/$LEAN4_GENERATED)" "$LEAN4_RESULT" "$LEAN4_RC"
    echo "  lean4: rc=$LEAN4_RC -> $LEAN4_RESULT"
  else
    echo "check-verification-full: FAILED (Lean4 unavailable; proof lane is always-on — run tools/formal-install lean4)" >&2
    exit 1
  fi
fi

echo "== stale-check all generated targets (P1-FM-010) =="
# All-target staleness: every generated Alloy/P/Kani/Verus/Lean4 artifact must match
# its source bundle + scenario + codegen receipt, not just Alloy.
"${CLI[@]}" formal stale-check-all \
  --bundle "$BUNDLE" \
  --scenario "$SCENARIO" \
  --artifact-dir verification/formal-full \
  --receipt-dir verification/formal-full/receipts

echo "== derive coverage from receipts (P0-FM-002: derived, never greened) =="
# Re-derive the coverage report from the tool-run receipts on disk. Status is
# computed from each receipt's actual_result + exit_code; a failed tool run
# (or an un-refuted negative control) yields status=fail with no jq patching.
"${FORMAL[@]}" coverage \
  --bundle "$BUNDLE" \
  --scenario "$SCENARIO" \
  --artifact-dir verification/formal-full \
  --receipt-dir verification/formal-full/receipts \
  --coverage "$COVERAGE"

COV_STATUS="$(jq -r '.status' "$COVERAGE")"
echo "coverage: $COVERAGE (status=$COV_STATUS)"
echo "-- artifact lanes (derived from tool-run receipts) --"
jq -r '.artifacts[] | "  \(.target): status=\(.status) exit_code=\(.exit_code // "null")"' "$COVERAGE"
echo "-- invariant coverage (reconciled with artifact reality) --"
jq -r '.invariant_coverage[] | "  \(.invariant_id): replay=\(.replay) alloy=\(.alloy) p=\(.p) kani=\(.kani) verus=\(.verus) lean4=\(.lean4) => \(.status)"' "$COVERAGE"
echo "-- negative controls (executed through the replay oracle) --"
jq -r '.negative_controls[] | "  \(.scenario): expected=\(.expected_error_code // "none") actual=\(.actual_error_code // "none") => \(.status)"' "$COVERAGE"

# P0-012: the documented coverage matrix must not overclaim a lane the machine
# report does not back. Fail closed if the JSON or Markdown matrix has drifted
# from this run's report.
echo "== documented coverage matrix matches the report (P0-012) =="
tools/coverage-matrix --check

echo "== proof/refinement scope docs match the artifact =="
tools/proof-refinement-scope --check

echo "== formal discipline check (mandatory gate) =="
tools/formal-discipline-check --profile "$PROFILE" --no-diff --json > "$TARGET_DIR/formal-discipline.$PROFILE.$LANE.json"

if [ "$COV_STATUS" = "pass" ]; then
  echo "check-verification-full: OK (coverage derived from receipts; status=pass)"
  return 0
fi

echo "check-verification-full: FAILED (derived coverage status=$COV_STATUS)" >&2
jq -r '.artifacts[] | select(.status != "pass" and .status != "non_blocking_skipped" and .status != "expected_fail_refuted") | "  artifact \(.target): \(.status)"' "$COVERAGE" >&2 || true
jq -r '.negative_controls[] | select(.status != "refuted_by_replay") | "  negative control \(.scenario): \(.status)"' "$COVERAGE" >&2 || true
return 1
}

case "$SUITE" in
  formal)
    run_formal
    ;;
  property)
    run_property
    ;;
  fuzz)
    run_fuzz
    ;;
  all)
    run_formal
    run_property
    run_fuzz
    ;;
esac
