# PUB5 causlane-core Dry-run Evidence

**Status:** `causlane-core` dry-run passed; no crates.io upload performed.

This hand-maintained evidence records the first staged PUB5 dry-run for the
`0.0.1` pre-alpha workspace release. It is release evidence only; it does not
authorize upload by itself.

## Dry-run Scope

Reviewed source baseline:

```text
main_commit: dec18df06d93c07b2567e678363cfecb1b12d822
date: 2026-06-26
host: dispatcher
runner: local repository workspace
```

Command:

```bash
cargo publish -p causlane-core --dry-run --locked
```

## Result

```text
crate: causlane-core
version: 0.0.1
files_packaged: 45
package_size: 415.8KiB
compressed_size: 93.8KiB
verify_compile: pass
upload: skipped by --dry-run
exit_code: 0
```

Cargo packaged `causlane-core`, verified it by compiling the package from the
packaged source and aborted upload because this was a dry-run.

## Next State

The publication state machine may move to:

```text
DryRunPassed(causlane-core)
```

The next irreversible action, if maintainers choose to continue, is:

```bash
cargo publish -p causlane-core --locked
```

Do not dry-run or publish dependent crates until `causlane-core` has been
published and indexed on crates.io.
