#!/usr/bin/env bash
set -euo pipefail

skip_cargo=0
skip_checker=0
skip_private_deps=0

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
    -h|--help)
      cat <<'EOF'
Usage: scripts/check-migration-gate.sh [--skip-cargo] [--skip-checker] [--skip-private-deps]

Runs the repo-local migration gate:
  - cli-checker config validation and check-repo
  - private git dependency policy scanner
  - optional architecture verifier
  - cargo fmt, clippy, and tests unless --skip-cargo is set
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
    exit 1
  fi
  if [ -f .cli-checker.toml ]; then
    cli-checker validate-config --config .cli-checker.toml
    cli-checker check-repo --config .cli-checker.toml
  else
    cli-checker check-repo
  fi
fi

if [ "$skip_private_deps" -eq 0 ]; then
  python3 scripts/check-private-deps.py
fi

if [ -f scripts/verify-architecture.py ]; then
  python3 scripts/verify-architecture.py
fi

if [ "$skip_cargo" -eq 0 ] && [ -f Cargo.toml ]; then
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace --all-targets
fi
