#!/usr/bin/env bash
set -uo pipefail

skip_cargo=0
skip_checker=0
skip_private_deps=0
skip_parallel_dev=0
status=0

run_step() {
  echo "==> $*"
  "$@"
  local rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "step failed ($rc): $*" >&2
    status=1
  fi
}

for arg in "$@"; do
  case "$arg" in
    --skip-cargo)
      skip_cargo=1
      ;;
    --skip-checker)
      skip_checker=1
      ;;
    --skip-private-deps)
      skip_private_deps=1
      ;;
    --skip-parallel-dev)
      skip_parallel_dev=1
      ;;
    -h|--help)
      cat <<'EOF'
Usage: scripts/check-migration-gate.sh [--skip-cargo] [--skip-checker] [--skip-private-deps] [--skip-parallel-dev]

Runs the repo-local migration gate:
  - cli-checker config validation and check-repo
  - private git dependency policy scanner
  - optional architecture verifier
  - parallel-development readiness checks
  - cargo fmt, clippy, and tests unless --skip-cargo is set

All available checks run; the script exits non-zero at the end if any step failed.
EOF
      exit 0
      ;;
    *)
      echo "unknown argument: $arg" >&2
      exit 2
      ;;
  esac
done

root="$(git rev-parse --show-toplevel)"
cd "$root"

if [ "$skip_checker" -eq 0 ]; then
  if ! command -v cli-checker >/dev/null 2>&1; then
    echo "cli-checker is required but was not found in PATH" >&2
    status=1
  elif [ -f .cli-checker.toml ]; then
    run_step cli-checker validate-config --config .cli-checker.toml
    run_step cli-checker check-repo --config .cli-checker.toml
  else
    run_step cli-checker check-repo
  fi
fi

if [ "$skip_private_deps" -eq 0 ]; then
  run_step cli-checker project dependencies validate --profile .devinfra/cli-checker/project-tooling-profile.yaml --format human
fi

if [ -f scripts/verify-architecture.py ]; then
  run_step python3 scripts/verify-architecture.py
fi

if [ "$skip_parallel_dev" -eq 0 ] && [ -d scripts/parallel-dev ]; then
  run_step python3 scripts/parallel-dev/check_parallel_dev.py --scan-bridge-terms
  run_step python3 scripts/parallel-dev/check_version_set.py
  run_step python3 scripts/parallel-dev/check_touch_ownership.py --base HEAD --change-class evidence_only
  run_step python3 scripts/parallel-dev/check_long_lane_registry.py
  run_step python3 scripts/parallel-dev/summarize_parallel_dev_readiness.py
fi

if [ "$skip_cargo" -eq 0 ] && [ -f Cargo.toml ]; then
  run_step cargo fmt --all -- --check
  run_step cargo clippy --workspace --all-targets -- -D warnings
  run_step cargo test --workspace --all-targets
fi

exit "$status"
