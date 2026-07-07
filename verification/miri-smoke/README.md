# Miri smoke

This repository exposes `scripts/check-miri-smoke.sh` as the Stage 11 Miri entrypoint.

Selected package: `causlane-core`

Semantic check: Causlane lane admission must not grant cross-tier authority.

Run from the repository root:

```sh
scripts/check-miri-smoke.sh
```

The smoke intentionally stays narrow: it runs a repo-owned semantic kernel under
`cargo +nightly miri test` instead of attempting to interpret the whole workspace.
