# Context Pack Hygiene

Before sharing a repomix/context-pack snapshot, run:

```bash
just context-pack-scan path/to/context-pack.txt
```

If no path is supplied, the scanner checks git-visible files in the checkout:

```bash
just context-pack-scan
```

Context packs must exclude local state, generated run caches and credential-like
material. The repository carries both `.gitignore` and `.repomixignore`
coverage for:

```text
.agent-state/**
.local/**
.devinfra/logs/**
.devinfra/run/**
**/mc/config.json
**/.env
**/*.env
**/*secret*
**/*credential*
minio/**
objectstore/**
objectstore-data/**
run-cache/**
formal/receipts/*.json
formal/*/generated/*
target/**
```

`tools/context-pack-scan` fails on forbidden paths and env-like key/value
material such as access keys, tokens, passwords, credentials and private-key
blocks. If a pack containing local objectstore/minio config or credentials has
already left the trusted workspace, rotate those local credentials before
continuing work.

## What a context pack deliberately omits (and how to review it)

A context pack is a **source-and-docs** snapshot, not a buildable checkout. The
`target/**` exclusion above means **no compiled binaries are ever in a pack** —
including the `causlane-formal` executable. Likewise `formal/*/generated/*` keeps
generated Alloy/P/Kani/Verus/Lean4 artifacts and `formal/receipts/*.json` out.

This is by design, and it has a recurring consequence for **static reviewers**:

> `crates/causlane-cli/Cargo.toml` declares `[[bin]] name = "causlane-formal"`,
> but a pack contains **no compiled binary** for it (excluded via `target/**`).
> This is **not** a "missing binary" blocker. The binary's source,
> `crates/causlane-cli/src/bin/causlane-formal.rs`, **is** tracked in git and
> present in the repository; `tools/formal-verify-all` builds and runs it. A
> reviewer who only sees the `[[bin]]` declaration in a pack must verify against
> the live checkout / gate, never infer "the binary is missing" from the pack.

More generally: do not treat the absence of a build artifact, a generated formal
artifact, or a receipt in a pack as evidence that the corresponding source or
toolchain step is broken. The authority chain is the gate
(`just formal-verify-all`) run against the real checkout, not the pack contents.
