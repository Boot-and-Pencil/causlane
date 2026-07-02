#!/usr/bin/env bash
set -euo pipefail

SCRIPT_PATH="${BASH_SOURCE[0]}"
if command -v readlink >/dev/null 2>&1; then
  SCRIPT_PATH="$(readlink -f "$SCRIPT_PATH")"
fi
ROOT="$(cd -- "$(dirname -- "$SCRIPT_PATH")/.." && pwd -P)"
. "$ROOT/tools/formal-env" --source

LANE="all"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --lane)
      LANE="${2:?missing value for --lane}"
      shift 2
      ;;
    *)
      echo "usage: scripts/check-formal-smoke.sh [--lane all|kani|verus|p-lang|lean4|property|fuzz]" >&2
      exit 2
      ;;
  esac
done

case "$LANE" in
  all|kani|verus|p-lang|lean4|property|fuzz) ;;
  *) echo "unknown lane: $LANE" >&2; exit 2 ;;
esac

ARTIFACT_ROOT="$ROOT/.agent-state/formal-smoke"
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)"
RUN_DIR="$ARTIFACT_ROOT/runs/$RUN_ID"
mkdir -p "$RUN_DIR" "$FORMAL_SMOKE_TMPDIR"
export TMPDIR="$FORMAL_SMOKE_TMPDIR"

git_field() {
  git -C "$ROOT" "$@" 2>/dev/null || true
}

write_lane_json() {
  local lane="$1" status="$2" rc="$3" command="$4" started="$5" ended="$6" stdout="$7" stderr="$8" note="${9:-}"
  python3 - "$ARTIFACT_ROOT/$lane.json" "$lane" "$status" "$rc" "$command" "$started" "$ended" "$stdout" "$stderr" "$note" <<'PY'
import json
import sys
from pathlib import Path

path, lane, status, rc, command, started, ended, stdout_path, stderr_path, note = sys.argv[1:]
data = {
    "schema_version": "hopium.formal_smoke.lane.v1",
    "lane": lane,
    "status": status,
    "rc": int(rc),
    "command": command,
    "started_at": started,
    "ended_at": ended,
    "stdout": stdout_path,
    "stderr": stderr_path,
    "note": note,
}
Path(path).write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(json.dumps(data, sort_keys=True))
PY
}

run_command() {
  local lane="$1"
  shift
  local stdout="$RUN_DIR/$lane.stdout"
  local stderr="$RUN_DIR/$lane.stderr"
  local started ended rc command
  started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  command="$*"
  set +e
  "$@" >"$stdout" 2>"$stderr"
  rc=$?
  set -e
  ended="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if [ "$rc" -eq 0 ]; then
    write_lane_json "$lane" "pass" "$rc" "$command" "$started" "$ended" "$stdout" "$stderr" ""
  else
    write_lane_json "$lane" "fail" "$rc" "$command" "$started" "$ended" "$stdout" "$stderr" ""
  fi
  return "$rc"
}

run_p_lang() {
  local lane="p-lang"
  local stdout="$RUN_DIR/$lane.stdout"
  local stderr="$RUN_DIR/$lane.stderr"
  local outdir="$RUN_DIR/p-lang-out"
  local started ended compile_rc valid_rc invalid_rc dll status rc note
  mkdir -p "$outdir"
  started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  set +e
  {
    p compile --pfiles "$ROOT/verification/formal-smoke/p-lang/Smoke.p" --projname formal_smoke --outdir "$outdir"
    compile_rc=$?
    dll="$outdir/PChecker/net8.0/formal_smoke.dll"
    if [ ! -f "$dll" ]; then
      dll="$(find "$outdir" -path '*/PChecker/net8.0/formal_smoke.dll' -print -quit)"
    fi
    if [ "$compile_rc" -eq 0 ] && [ -n "$dll" ]; then
      (cd "$outdir" && p check "$dll" --testcase TcGateBeforeUseValid --schedules 1 --max-steps 16 --outdir "$RUN_DIR/pcheck-valid")
      valid_rc=$?
      (cd "$outdir" && p check "$dll" --testcase TcGateBeforeUseInvalid --schedules 1 --max-steps 16 --outdir "$RUN_DIR/pcheck-invalid")
      invalid_rc=$?
    else
      valid_rc=99
      invalid_rc=99
    fi
    echo "compile_rc=$compile_rc valid_rc=$valid_rc invalid_rc=$invalid_rc dll=${dll:-}"
  } >"$stdout" 2>"$stderr"
  set -e
  ended="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if [ "${compile_rc:-99}" -eq 0 ] && [ "${valid_rc:-99}" -eq 0 ] && [ "${invalid_rc:-0}" -ne 0 ]; then
    status=pass
    rc=0
    note="positive testcase accepted; negative control rejected"
  else
    status=fail
    rc=1
    note="expected compile_rc=0 valid_rc=0 invalid_rc!=0"
  fi
  write_lane_json "$lane" "$status" "$rc" "p compile/check formal_smoke" "$started" "$ended" "$stdout" "$stderr" "$note"
  return "$rc"
}

run_lane() {
  case "$1" in
    kani)
      run_command kani env -u RUSTC_WRAPPER -u CARGO_BUILD_RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS cargo-kani --manifest-path "$ROOT/verification/formal-smoke/rust/Cargo.toml" --harness gate_before_use_kani_smoke --default-unwind 8 --output-format terse --target-dir "$RUN_DIR/target/kani"
      ;;
    verus)
      run_command verus verus "$ROOT/verification/formal-smoke/verus/smoke.rs" --no-cheating
      ;;
    p-lang)
      run_p_lang
      ;;
    lean4)
      run_command lean4 lean "$ROOT/verification/formal-smoke/lean4/Smoke.lean"
      ;;
    property)
      run_command property cargo test --manifest-path "$ROOT/verification/formal-smoke/rust/Cargo.toml" --test property -- --nocapture
      ;;
    fuzz)
      mkdir -p "$RUN_DIR/fuzz-corpus" "$RUN_DIR/fuzz-artifacts"
      run_command fuzz env -u RUSTC_WRAPPER -u CARGO_BUILD_RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS cargo fuzz run gate_before_use "$RUN_DIR/fuzz-corpus" --fuzz-dir "$ROOT/verification/formal-smoke/fuzz" --sanitizer none --target x86_64-unknown-linux-gnu --target-dir "$RUN_DIR/target/fuzz" -- -runs=8 -artifact_prefix="$RUN_DIR/fuzz-artifacts/"
      ;;
  esac
}

python3 "$ROOT/tools/formal-smoke-doctor" --json --require all > "$RUN_DIR/formal-doctor.json"

lanes=()
if [ "$LANE" = "all" ]; then
  lanes=(kani verus p-lang lean4 property fuzz)
else
  lanes=("$LANE")
fi

failed=0
for lane in "${lanes[@]}"; do
  if ! run_lane "$lane"; then
    failed=1
  fi
done

python3 - "$ARTIFACT_ROOT/summary.json" "$RUN_ID" "$failed" "${lanes[@]}" <<'PY'
import json
import subprocess
import sys
from pathlib import Path

path = Path(sys.argv[1])
run_id = sys.argv[2]
failed = int(sys.argv[3])
lanes = sys.argv[4:]
root = path.parents[2]
lane_reports = []
for lane in lanes:
    lane_path = path.parent / f"{lane}.json"
    lane_reports.append(json.loads(lane_path.read_text()))

def git(args):
    try:
        return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()
    except Exception:
        return ""

def repo_name():
    remote = git(["config", "--get", "remote.origin.url"])
    if remote:
        name = remote.rstrip("/").rsplit("/", 1)[-1]
        if name.endswith(".git"):
            name = name[:-4]
        if name:
            return name
    return root.name

summary = {
    "schema_version": "hopium.formal_smoke.summary.v1",
    "run_id": run_id,
    "status": "fail" if failed else "pass",
    "repo": repo_name(),
    "branch": git(["rev-parse", "--abbrev-ref", "HEAD"]),
    "head": git(["rev-parse", "HEAD"]),
    "lanes": lane_reports,
}
path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"formal-smoke: {summary['status']} repo={summary['repo']} lanes={len(lanes)}")
PY

exit "$failed"
