#!/usr/bin/env bash
# causlane formal smoke: compile the headless Alloy runner (if needed), run the
# core model (expected pass) and the negative control (expected fail), and write
# a receipt. Exit 0 only if the core model holds and the negative control is
# correctly refuted — proving the Alloy lane is real and discriminating.
set -euo pipefail
cd "$(dirname "$0")/../.."

ALLOY_JAR=".tools/alloy/alloy.jar"
CLASSES=".tools/alloy/classes"
CORE="verification/formal-full/alloy/core/causlane_core.als"
NEG="verification/formal-full/alloy/checks/unconstrained_counterexample.als"
RECEIPT="verification/formal-full/receipts/alloy-core.json"
REGISTRY="contracts/examples/release_promote.registry.yaml"
SUCCESS_SCENARIO="contracts/scenarios/release_promote_success.scenario.yaml"
NEG_EXEC_SCENARIO="contracts/scenarios/execution_without_barrier_invalid.scenario.yaml"
NEG_LEASE_SCENARIO="contracts/scenarios/conflicting_leases_invalid.scenario.yaml"
BUNDLE="target/formal/release_promote.bundle.json"
GEN_SUCCESS="verification/formal-full/alloy/generated/release_promote_success.als"
GEN_NEG_EXEC="verification/formal-full/alloy/generated/execution_without_barrier_invalid.als"
GEN_NEG_LEASE="verification/formal-full/alloy/generated/conflicting_leases_invalid.als"
GEN_RECEIPT="verification/formal-full/receipts/release_promote.codegen.json"

if [ ! -f "$ALLOY_JAR" ]; then
  echo "alloy jar missing: $ALLOY_JAR; run: just formal-doctor" >&2
  exit 2
fi

mkdir -p "$CLASSES" verification/formal-full/receipts verification/formal-full/alloy/generated target/formal
if [ ! -f "$CLASSES/AlloyRunner.class" ] || [ verification/formal-full/tools/AlloyRunner.java -nt "$CLASSES/AlloyRunner.class" ]; then
  javac -cp "$ALLOY_JAR" -d "$CLASSES" verification/formal-full/tools/AlloyRunner.java
fi

echo "== generate bundle + scenario Alloy facts =="
./tools/cargo-dev run -q -p causlane-cli --bin causlane -- bundle compile --registry "$REGISTRY" --out "$BUNDLE"
./tools/cargo-dev run -q -p causlane-cli --bin causlane -- formal generate alloy --bundle "$BUNDLE" --scenario "$SUCCESS_SCENARIO" --out "$GEN_SUCCESS" --receipt "$GEN_RECEIPT"
./tools/cargo-dev run -q -p causlane-cli --bin causlane -- formal stale-check --bundle "$BUNDLE" --scenario "$SUCCESS_SCENARIO" --generated "$GEN_SUCCESS" --receipt "$GEN_RECEIPT"
./tools/cargo-dev run -q -p causlane-cli --bin causlane -- formal generate alloy --bundle "$BUNDLE" --scenario "$NEG_EXEC_SCENARIO" --out "$GEN_NEG_EXEC"
./tools/cargo-dev run -q -p causlane-cli --bin causlane -- formal generate alloy --bundle "$BUNDLE" --scenario "$NEG_LEASE_SCENARIO" --out "$GEN_NEG_LEASE"

echo "== core model (expect pass) =="
CORE_OUT="$(java -cp "$ALLOY_JAR:$CLASSES" AlloyRunner "$CORE" 2>/dev/null)"
echo "$CORE_OUT"
CORE_STATUS="$(printf '%s' "$CORE_OUT" | jq -r .status)"

echo "== negative control (expect fail) =="
NEG_OUT="$(java -cp "$ALLOY_JAR:$CLASSES" AlloyRunner "$NEG" 2>/dev/null || true)"
echo "$NEG_OUT"
NEG_STATUS="$(printf '%s' "$NEG_OUT" | jq -r .status)"

echo "== generated release_promote scenario (expect pass) =="
GEN_OUT="$(java -cp "$ALLOY_JAR:$CLASSES" AlloyRunner "$GEN_SUCCESS" 2>/dev/null)"
echo "$GEN_OUT"
GEN_STATUS="$(printf '%s' "$GEN_OUT" | jq -r .status)"

echo "== generated negative: execution without barrier (expect fail) =="
GEN_NEG_EXEC_OUT="$(java -cp "$ALLOY_JAR:$CLASSES" AlloyRunner "$GEN_NEG_EXEC" 2>/dev/null || true)"
echo "$GEN_NEG_EXEC_OUT"
GEN_NEG_EXEC_STATUS="$(printf '%s' "$GEN_NEG_EXEC_OUT" | jq -r .status)"

echo "== generated negative: conflicting leases (expect fail) =="
GEN_NEG_LEASE_OUT="$(java -cp "$ALLOY_JAR:$CLASSES" AlloyRunner "$GEN_NEG_LEASE" 2>/dev/null || true)"
echo "$GEN_NEG_LEASE_OUT"
GEN_NEG_LEASE_STATUS="$(printf '%s' "$GEN_NEG_LEASE_OUT" | jq -r .status)"

MODEL_HASH="sha256:$(sha256sum "$CORE" | cut -d' ' -f1)"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
ALLOY_VER="$(jq -r '.tools.formal_tools.alloy.version' .devinfra/tool-versions.json 2>/dev/null || echo unknown)"

cat > "$RECEIPT" <<JSON
{
  "schema_version": 2,
  "receipt_kind": "tool_run",
  "artifact_kind": "alloy_core",
  "tool": "alloy",
  "tool_version": "$ALLOY_VER",
  "generator_version": "manual-core",
  "source_bundle_hash": null,
  "scenario_hash": null,
  "core_model_hash": "$MODEL_HASH",
  "generated_artifact_hash": "$MODEL_HASH",
  "command": "java -cp $ALLOY_JAR:$CLASSES AlloyRunner $CORE",
  "expected_result": "pass",
  "actual_result": "$CORE_STATUS",
  "invariant_ids": ["I-001", "I-002", "I-003"],
  "checked_at": "$TS",
  "scope": { "predicates": 0, "scenarios": 0 }
}
JSON
echo "receipt: $RECEIPT"

if [ "$CORE_STATUS" = "pass" ] \
  && [ "$NEG_STATUS" = "fail" ] \
  && [ "$GEN_STATUS" = "pass" ] \
  && [ "$GEN_NEG_EXEC_STATUS" = "fail" ] \
  && [ "$GEN_NEG_LEASE_STATUS" = "fail" ]; then
  echo "formal-smoke: OK (core and generated scenario hold; negative controls correctly refuted)"
  exit 0
fi
echo "formal-smoke: FAILED (core=$CORE_STATUS neg=$NEG_STATUS generated=$GEN_STATUS gen_exec=$GEN_NEG_EXEC_STATUS gen_lease=$GEN_NEG_LEASE_STATUS)" >&2
exit 1
