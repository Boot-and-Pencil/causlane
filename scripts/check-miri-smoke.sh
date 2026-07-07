#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

PACKAGE='causlane-core'
TEST_FILTER='lane_never_grants_cross_tier_authority'
SMOKE_NOTE='Causlane lane admission must not grant cross-tier authority.'

echo "miri-smoke: ${PACKAGE}"
echo "semantic-check: ${SMOKE_NOTE}"

# Guard against a stale filter silently producing a zero-test Miri run.
cargo test -p "${PACKAGE}" --lib "${TEST_FILTER}" -- --list | grep -F "${TEST_FILTER}" >/dev/null
cargo +nightly miri test -p "${PACKAGE}" --lib "${TEST_FILTER}" -- --nocapture
