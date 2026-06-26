# Publication Refactor Stage Checklist

Use this checklist for the immediate refactor stage before publication.

## Repository Shape

- [ ] `tools/architecture-lint --json` has zero errors.
- [ ] Every declared Cargo binary has a source file.
- [ ] No package includes private context packs, scratch artifacts or local tool caches.
- [ ] Workspace manifests use version `0.0.1`.
- [ ] Internal workspace dependencies use `path + version = 0.0.1`.

## Boundaries

- [ ] `causlane-core` has no runtime/adapters dependencies.
- [ ] `causlane-contracts` owns registry/bundle/hash contracts.
- [ ] `causlane-replay` owns executable trace verification.
- [ ] `causlane-codegen` owns formal artifact generation.
- [ ] `causlane-runtime` owns adapters and runtime shells.
- [ ] `causlane-cli` owns CLI boundaries and app services.
- [ ] `causlane` facade is intentionally small.

## Semantic Naming

- [ ] No production identifiers named after roadmap stages, milestones,
  patch-packs or snapshot numbers.
- [ ] Tests are named after protocol behavior.
- [ ] Variables and functions describe semantic role.
- [ ] Historical names remain only in ADRs, release notes and impact records.

## Public API

- [ ] Intended imports are documented.
- [ ] Broad public re-exports are removed or exception-recorded.
- [ ] Crate READMEs accurately describe status and role.
- [ ] `cargo doc --workspace --no-deps --locked` passes.

## Documentation

- [ ] `AI_USAGE.md` states human accountability.
- [ ] `AGENTS.md` includes publication rules.
- [ ] `PUBLISHING.md` uses staged dry-run/publish.
- [ ] `RELEASE.md` distinguishes `0.0.1` pre-alpha from `0.1.x` alpha.
- [ ] Generated readiness docs are regenerated, not hand-edited.

## Release Hygiene

- [ ] Secret scan recorded.
- [ ] Context-pack scan recorded.
- [ ] Package file lists reviewed.
- [ ] Public GitHub URL resolves.
- [ ] Curated history decision recorded.
