set shell := ["bash", "-euo", "pipefail", "-c"]

status:
  ./tools/devctl status

status-json:
  ./tools/devctl status --json

diagnostics:
  ./tools/devctl diagnostics --format=human

diagnostics-json:
  ./tools/devctl diagnostics --format=json

checker-status *args:
  ./tools/checker-wrapper status --json {{args}}

rust-rule-inventory:
  cli-checker validator-inventory --format json \
    | jq '{schema, summary, rust: [.validators[] | select(.rule_id | startswith("RUST-"))]}'

rust-prewrite-plan:
  cli-checker debug-plan \
    --mode hook \
    --config .cli-checker.toml \
    --policy-pack PACK-RUST-CORE \
    --policy-pack PACK-RUST-ARCHITECTURE \
    --policy-pack PACK-RUST-PANIC-STRICT \
    --policy-pack PACK-SADAVE-RUST-READINESS \
    --policy-pack PACK-SADAVE-RUST-READINESS-STRICT \
    --policy-pack PACK-SEMGREP-CORE \
    --policy-pack PACK-REPO-HYGIENE \
    --language rust \
    --format json

rust-full-plan:
  cli-checker debug-plan \
    --mode check-repo \
    --config .cli-checker.toml \
    --policy-pack PACK-RUST-CORE \
    --policy-pack PACK-RUST-ARCHITECTURE \
    --policy-pack PACK-RUST-PANIC-STRICT \
    --policy-pack PACK-SADAVE-RUST-READINESS \
    --policy-pack PACK-SADAVE-RUST-READINESS-STRICT \
    --policy-pack PACK-SEMGREP-CORE \
    --policy-pack PACK-REPO-HYGIENE \
    --language rust \
    --format json

rust-runtime-readiness:
  cli-checker readiness-requirements --tool cargo_clippy --format json
  cli-checker readiness-requirements --tool cargo_nextest --format json
  cli-checker readiness-requirements --tool cargo_deny --format json
  cli-checker readiness-requirements --tool cargo_llvm_cov --format json
  cli-checker readiness-requirements --tool semgrep --format json

rust-full-check:
  ./tools/cargo-dev fmt --all --check
  ./tools/cargo-dev check --workspace --all-targets --all-features --locked --keep-going
  ./tools/cargo-dev clippy --workspace --all-targets --all-features --locked -- -D warnings
  ./tools/cargo-dev nextest run --workspace --all-targets --all-features --locked --no-run
  ./tools/cargo-dev deny check
  ./tools/checker-wrapper run --mode=foreground --cache-class=dev --cargo=./tools/cargo-dev --snapshot-file=.agent-state/current-snapshot.json --output=.agent-state/checker.json --message-format=json
  ./tools/checker-wrapper status --json --require-fresh --require-pass

rust-full-check-json:
  ./tools/checker-wrapper status --json --require-fresh --require-pass

prewarm-status:
  ./tools/devctl prewarm-status --json

prewarm-up:
  ./tools/devctl env exec -- process-compose -f process-compose.yaml up

prewarm-down:
  ./tools/devctl env exec -- process-compose -f process-compose.yaml down

prewarm-up-bacon:
  ./tools/devctl env exec -- process-compose -f process-compose.with-bacon.yaml up

prewarm-once:
  ./tools/prewarmd run --config .devinfra/commands.toml --once

check:
  ./tools/cargo-dev check --workspace --all-targets --all-features --locked --keep-going

clippy:
  ./tools/cargo-dev clippy --workspace --all-targets --all-features --locked -- -D warnings

test-build:
  ./tools/cargo-dev nextest run --workspace --all-targets --all-features --locked --no-run

test:
  ./tools/cargo-dev nextest run --workspace --all-targets --all-features --locked

coverage:
  ./tools/cargo-cov nextest --workspace --all-features --no-clean --fail-under-lines 85 --lcov --output-path target/llvm-cov-target/lcov.info
  ./tools/devctl coverage-summary --lcov target/llvm-cov-target/lcov.info >/dev/null

coverage-build:
  ./tools/cargo-cov-prebuild --workspace --all-features --locked

coverage-clean:
  ./tools/cargo-cov clean --workspace

bench-dispatch-baseline-build:
  ./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked --no-run

bench-dispatch-baseline:
  ./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked

cache-key class="dev":
  ./tools/cache-key {{class}}

cache-audit:
  ./tools/checker-wrapper audit-cargo-rebuild --cargo=./tools/cargo-dev --command="check --workspace --all-targets --all-features --locked"

doctor:
  ./tools/devctl doctor

context-pack-scan *paths:
  tools/context-pack-scan {{paths}}

# --- formal contour (P0 Formal Readiness) ---

formal-install tool:
  tools/formal-install {{tool}}

formal-doctor:
  cli-checker project formal doctor --profile .devinfra/cli-checker/project-tooling-profile.yaml --require all

formal-doctor-json:
  cli-checker project formal doctor --profile .devinfra/cli-checker/project-tooling-profile.yaml --require all --format json

formal-smoke:
  bash verification/formal-full/smoke.sh

formal-ready:
  tools/formal-ready

verification-full *args:
  scripts/check-verification-full.sh {{args}}

verification-formal:
  scripts/check-verification-full.sh --suite formal --profile all --depth fast_ci

verification-property:
  scripts/check-verification-full.sh --suite property

verification-fuzz:
  scripts/check-verification-full.sh --suite fuzz --fuzz-runs 1

formal-coverage:
  if [ ! -f target/causlane/formal-coverage-report.json ]; then scripts/check-verification-full.sh --suite formal --profile all --depth fast_ci; fi
  jq -e '.status == "pass"' target/causlane/formal-coverage-report.json >/dev/null
  jq -r '.invariant_coverage[] | "\(.invariant_id): \(.status)"' target/causlane/formal-coverage-report.json

# Regenerate docs/invariants/coverage-matrix.json from the machine coverage report.
formal-coverage-matrix:
  tools/coverage-matrix --write

# Fail if the documented coverage matrix overclaims a lane the report does not back.
formal-coverage-matrix-check:
  tools/coverage-matrix --check

# Regenerate the generated proof/refinement scope Markdown from the JSON artifact.
formal-proof-refinement-scope:
  tools/proof-refinement-scope --write

# Fail if the generated proof/refinement scope Markdown has drifted.
formal-proof-refinement-scope-check:
  tools/proof-refinement-scope --check

# Enforce the executable formal-lane exceptions policy (expiry + profile rules).
formal-exceptions-check *args:
  tools/formal-exceptions-check {{args}}

product-track-check:
  tools/product-track-status-check
  tools/migration-shadow-doc-check
  tools/adapter-ecosystem-doc-check
  tools/api-validation-loop-plan-check

publish-readiness:
  tools/publish-readiness --write

publish-readiness-check:
  tools/publish-readiness --check

# Data-driven contract-test harness (M04.5): replay every scenario in the
# manifest and assert its declared contract (pass / fail:CODE). Fails closed.
contract-test:
  ./tools/cargo-dev run -q -p causlane-cli --bin causlane -- contract test --manifest contracts/contract-tests.yaml

# --- full install / full doctor (spec §9.15/§9.16) ---

# Idempotent full-toolchain installer. Pass *args, e.g. `--profile dev`.
bootstrap-full *args:
  tools/bootstrap-full {{args}}

# Fast tool-presence/version gate over the whole install surface (JSON).
# Does NOT run heavy repo gates; use tools/full-doctor --with-gates for those.
doctor-full:
  tools/full-doctor --json --profile all

ci-dispatcher-preflight *args:
  tools/ci-dispatcher-preflight {{args}}

# --- architecture / quality refactor gates ---

architecture-lint *args:
  python3 tools/architecture-lint {{args}}

architecture-lint-json *args:
  python3 tools/architecture-lint --json {{args}}

architecture-lint-strict *args:
  python3 tools/architecture-lint --strict {{args}}

refactor-readiness:
  mkdir -p target/causlane
  python3 tools/architecture-lint --json > target/causlane/architecture-lint.json
  python3 tools/validate-json-schema --schema .devinfra/state-schema/architecture-lint.schema.json target/causlane/architecture-lint.json
  jq -e '.status == "pass"' target/causlane/architecture-lint.json >/dev/null
  tools/semantic-naming-scan
  ./tools/schema-validate-all
  tools/formal-exceptions-check
  tools/product-track-status-check
  tools/migration-shadow-doc-check
  tools/adapter-ecosystem-doc-check
  tools/api-validation-loop-plan-check

commit-trailer-check count="50":
  tools/check-commit-trailers {{count}}
