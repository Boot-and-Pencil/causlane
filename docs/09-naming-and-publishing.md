# Naming and publishing

## Working name

The selected working name is:

```text
causlane
```

Meaning:

```text
causal ordering + execution lanes
```

The name fits the core concerns:

- causal witnesses;
- happens-before semantics;
- execution lanes;
- consequence-aware dispatch;
- constrained parallelism.

## Crate family

Recommended crate names:

```text
causlane              public facade
causlane-core         pure kernel
causlane-contracts    registry/bundle contracts
causlane-runtime      runtime composition
causlane-replay       replay verifier
causlane-cli          CLI
causlane-authz        authz integration facade
causlane-observe      observability facade
```

## Availability note

A web search during project scaffolding did not find an existing `crates.io/crates/causlane` crate. This does **not** reserve the name.

Current deterministic readiness is tracked in
[`release/publish-readiness.md`](release/publish-readiness.md), generated from
[`release/publish-readiness.json`](release/publish-readiness.json). Regenerate
and check it with:

```bash
just publish-readiness
just publish-readiness-check
```

The network-dependent crates.io name probe is advisory only:

```bash
tools/publish-readiness --online
```

If the name is still available, publication execution is no longer deferred and
the project is ready, publish a minimal `0.0.1` placeholder to reserve it. This
repository currently does not publish or reserve the crate from the readiness
gate.

## Publishing readiness

Before first publish:

- fill authors;
- set repository URL;
- add real license files;
- decide public API stability policy;
- ensure README is crate-friendly;
- add crate description, categories and keywords;
- run package dry-run;
- check package contents;
- decide whether to publish only facade first or all crate family members.

The no-publish readiness gate clears repo-local blockers only. At the moment,
`cargo publish --dry-run -p causlane` is expected to remain deferred until the
generated facade publish sequence is executable against crates.io: the facade
has a normal dependency on `causlane-core`, which is not available from
crates.io.
