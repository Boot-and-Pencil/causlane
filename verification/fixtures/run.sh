#!/usr/bin/env bash
set -euo pipefail

family=$1
mode=$2
root="$(cd "$(dirname "$0")/../.." && pwd)"
state="$root/.agent-state/verification-fixtures"
model="$root/verification/fixtures/model/Cargo.toml"
mkdir -p "$state"

if [ "${VERIFICATION_FIXTURE_CAPTURED:-0}" != 1 ]; then
  log="$state/probe-$family-$mode.log"
  set +e
  VERIFICATION_FIXTURE_CAPTURED=1 "$0" "$family" "$mode" >"$log" 2>&1
  run_status=$?
  set -e
  cat "$log"
  if [ "$mode" = positive ]; then
    exit "$run_status"
  fi
  case "$family" in
    unit) expected_status=101; marker="unit_detection_control_rejects_parent_fallback ... FAILED" ;;
    integration) expected_status=101; marker="integration_detection_control_rejects_parent_fallback ... FAILED" ;;
    property) expected_status=101; marker="minimal failing input" ;;
    fuzz) expected_status=1; marker="Test unit written to" ;;
    alloy) expected_status=1; marker="SelectionNeverWidens=false" ;;
    p_lang) expected_status=1; marker="Checker found a bug." ;;
    verus) expected_status=1; marker="postcondition not satisfied" ;;
    kani) expected_status=1; marker="VERIFICATION:- FAILED" ;;
    lean4) expected_status=1; marker="The left-hand side" ;;
    souffle) expected_status=1; marker="expected selected identity" ;;
    miri) expected_status=101; marker="unit_detection_control_rejects_parent_fallback ... FAILED" ;;
    loom) expected_status=101; marker="loom_detection_control_rejects_a_widened_identity ... FAILED" ;;
    mutation) expected_status=2; marker="MISSED" ;;
    *) echo "unknown detection fixture family: $family" >&2; exit 2 ;;
  esac
  if [ "$run_status" -ne "$expected_status" ] || ! grep -Fq "$marker" "$log"; then
    echo "detection probe did not produce its declared failure: family=$family exit=$run_status expected=$expected_status marker=$marker" >&2
    exit 2
  fi
  exit 1
fi

run_status=0
set +e
case "$family" in
  unit)
    if [ "$mode" = positive ]; then
      CARGO_TARGET_DIR="$state/cargo-target" cargo test --manifest-path "$model" unit_exact_selection
    else
      CARGO_TARGET_DIR="$state/cargo-target" cargo test --manifest-path "$model" --features detection-fixture unit_detection_control_rejects_parent_fallback
    fi
    run_status=$?
    ;;
  integration)
    if [ "$mode" = positive ]; then
      CARGO_TARGET_DIR="$state/cargo-target" cargo test --manifest-path "$model" --test integration integration_exact_selection_is_preserved
    else
      CARGO_TARGET_DIR="$state/cargo-target" cargo test --manifest-path "$model" --features detection-fixture --test integration integration_detection_control_rejects_parent_fallback
    fi
    run_status=$?
    ;;
  property)
    if [ "$mode" = positive ]; then
      CARGO_TARGET_DIR="$state/cargo-target" cargo test --manifest-path "$model" --test property property_exact_selection_never_widens
    else
      CARGO_TARGET_DIR="$state/cargo-target" cargo test --manifest-path "$model" --features detection-fixture --test property property_detection_control_shrinks_parent_fallback
    fi
    run_status=$?
    ;;
  fuzz)
    target=exact_selection
    source_corpus="$root/verification/fixtures/fuzz/corpus/exact_selection"
    seed=seed
    if [ "$mode" = detection ]; then
      target=detection
      source_corpus="$root/verification/fixtures/fuzz/corpus/detection"
      seed=trigger
    fi
    corpus="$state/fuzz-corpus/$target"
    mkdir -p "$corpus"
    cp "$source_corpus/$seed" "$corpus/$seed"
    mkdir -p "$state/fuzz-artifacts/$target"
    env -u RUSTC_WRAPPER -u CARGO_BUILD_RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS cargo fuzz run "$target" "$corpus" --fuzz-dir "$root/verification/fixtures/fuzz" --sanitizer none --target x86_64-unknown-linux-gnu --target-dir "$state/fuzz-target/$target" -- -runs=64 -artifact_prefix="$state/fuzz-artifacts/$target/"
    run_status=$?
    ;;
  alloy)
    classes="$state/alloy/classes"
    mkdir -p "$classes"
    javac -cp "$root/.tools/alloy/alloy.jar" -d "$classes" "$root/verification/fixtures/alloy/AlloyRunner.java"
    compile_status=$?
    if [ "$compile_status" -ne 0 ]; then
      run_status=90
    else
      java -cp "$root/.tools/alloy/alloy.jar:$classes" verification.fixtures.alloy.AlloyRunner "$root/verification/fixtures/alloy/$mode.als"
      run_status=$?
    fi
    ;;
  p_lang)
    out="$state/p-lang"
    mkdir -p "$out"
    p compile --pfiles "$root/verification/fixtures/p-lang/exact-selection.p" --projname exact_selection --outdir "$out"
    compile_status=$?
    dll="$out/PChecker/net8.0/exact_selection.dll"
    if [ ! -f "$dll" ]; then
      dll="$(find "$out" -path '*/PChecker/net8.0/exact_selection.dll' -print -quit)"
    fi
    if [ "$compile_status" -ne 0 ] || [ -z "$dll" ]; then
      run_status=91
    else
      testcase=TcExactSelectionPositive
      if [ "$mode" = detection ]; then
        testcase=TcExactSelectionDetection
      fi
      p check "$dll" --testcase "$testcase" --schedules 1 --max-steps 16 --seed 1058 --outdir "$out/check-$mode"
      run_status=$?
    fi
    ;;
  verus)
    mkdir -p "$state/verus"
    (cd "$state/verus" && verus "$root/verification/fixtures/verus/$mode.rs" --no-cheating)
    run_status=$?
    ;;
  kani)
    harness=exact_selection_kani
    if [ "$mode" = detection ]; then
      harness=exact_selection_kani_detection
    fi
    env -u RUSTC_WRAPPER -u CARGO_BUILD_RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS cargo-kani --manifest-path "$model" --harness "$harness" --exact --default-unwind 8 --output-format terse --target-dir "$state/kani-target"
    run_status=$?
    ;;
  lean4)
    lean "$root/verification/fixtures/lean4/$mode.lean"
    run_status=$?
    ;;
  souffle)
    out="$state/souffle-$mode"
    mkdir -p "$out"
    souffle -D "$out" "$root/verification/fixtures/souffle/selection.dl"
    run_status=$?
    if [ "$run_status" -eq 0 ]; then
      if [ "$mode" = positive ]; then
        grep -Fxq $'3' "$out/selected.csv" || run_status=1
      else
        grep -Fxq $'5' "$out/selected.csv" || {
          echo "expected selected identity 5 was not derived" >&2
          run_status=1
        }
      fi
    fi
    ;;
  miri)
    miri_args=(+nightly-2026-04-16 miri test --manifest-path "$model")
    if [ "$mode" = positive ]; then
      CARGO_TARGET_DIR="$state/miri-target" cargo "${miri_args[@]}" unit_exact_selection
    else
      CARGO_TARGET_DIR="$state/miri-target" cargo "${miri_args[@]}" --features detection-fixture unit_detection_control_rejects_parent_fallback
    fi
    run_status=$?
    ;;
  loom)
    if [ "$mode" = positive ]; then
      CARGO_TARGET_DIR="$state/loom-target" cargo test --manifest-path "$model" --test loom loom_publishes_only_the_exact_selected_identity
    else
      CARGO_TARGET_DIR="$state/loom-target" cargo test --manifest-path "$model" --features detection-fixture --test loom loom_detection_control_rejects_a_widened_identity
    fi
    run_status=$?
    ;;
  mutation)
    pattern=select_exact
    if [ "$mode" = detection ]; then
      pattern=unobserved_parent_fallback
    fi
    cargo mutants --dir "$root/verification/fixtures/model" --file src/lib.rs --re "$pattern" --timeout 120 --jobs 1 --no-config --no-times --colors never --output "$state/mutation-$mode"
    run_status=$?
    ;;
  *)
    echo "unknown fixture family: $family" >&2
    exit 2
    ;;
esac
set -e

exit "$run_status"

