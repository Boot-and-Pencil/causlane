#!/usr/bin/env bash
set -uo pipefail

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

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
  echo "Usage: scripts/check-repository.sh"
  echo "Runs every required repository policy and native verification check."
  exit 0
elif [ "$#" -ne 0 ]; then
  echo "check-repository does not accept selective skip flags" >&2
  exit 2
fi

root="$(git rev-parse --show-toplevel)"
cd "$root"

if ! command -v cli-checker >/dev/null 2>&1; then
  echo "cli-checker is required but was not found in PATH" >&2
  exit 1
elif [ -f .cli-checker.toml ]; then
  run_step cli-checker validate-config --config .cli-checker.toml
  run_step cli-checker check-repo --config .cli-checker.toml
else
  run_step cli-checker check-repo
fi

if [ -f scripts/verify-architecture.py ]; then
  run_step python3 scripts/verify-architecture.py
fi

run_step cli-checker project no-compat validate --profile .devinfra/cli-checker/project-tooling-profile.yaml --format human
run_step cli-checker project dependencies validate --profile .devinfra/cli-checker/project-tooling-profile.yaml --format human
run_step cli-checker project ownership validate --profile .devinfra/cli-checker/project-tooling-profile.yaml --base HEAD --change-class evidence_only --format human
run_step cli-checker project long-lane declarations --profile .devinfra/cli-checker/project-tooling-profile.yaml --format human

if [ -f Cargo.toml ]; then
  run_step cargo fmt --all -- --check
  run_step cargo clippy --workspace --all-targets -- -D warnings
  run_step cargo test --workspace --all-targets
fi

exit "$status"
