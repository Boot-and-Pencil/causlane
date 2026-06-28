# 10. Release strategy

## Versioning

- `0.0.x`: experimental bootstrap releases. They may be private/internal or
  public pre-alpha, but they carry no API stability promise.
- `0.1.x`: first usable public alpha API surface.
- `0.5.x` or similar: beta once integrations validate API shape.
- `1.0.0`: stable API and compatibility policy.

The first crates.io publication target is `0.0.1`: a public pre-alpha bootstrap
release for package availability and provenance, not a stable alpha. That
release is published and indexed, signed tag `v0.0.1` is pushed, and the GitHub
pre-release is public. Future uploads still proceed only through the staged
one-crate runbook.

## Crate publication strategy

The machine-derived dependency tiers and facade publish sequence are generated
in [`../release/publish-readiness.md`](../release/publish-readiness.md). Treat
that artifact as the source of truth for repository-local readiness and package
order; do not duplicate generated order by hand in product-track pages.

Actual crates.io upload is executed only through `PUBLISHING.md` and
[`../release/publish-all-crates-runbook.md`](../release/publish-all-crates-runbook.md).
The `0.0.1` upload evidence is recorded under `docs/release/`.

Important distinction:

```text
publish-readiness report pass
  means deterministic repo-local no-upload checks passed.

actual upload readiness
  additionally requires the recorded public-baseline evidence to remain valid,
  package-list inspection for the selected baseline, staged registry dry-run
  and publication of internal dependencies in order.
```

## Stability policy

Stabilize first:

- core newtypes;
- AuditEvent/EventKind shapes;
- CompiledDispatchBundle shape;
- ReplayTrace shape;
- lifecycle stages;
- formal receipt schema.

Keep unstable first:

- runtime adapters;
- lane scheduler policy;
- proof generators beyond smoke/bounded checks;
- high-level macros/derive;
- UI/dashboard/service mode.

## Pre-alpha bootstrap criteria (`0.0.1`)

- refactor-before-publication gate records PUB0-PUB4 complete;
- package file-list review is recorded;
- public docs say experimental/pre-alpha;
- AI/provenance policy exists;
- public GitHub baseline is curated and scanned;
- staged publish runbook is followed;
- crates do not overclaim workflow-engine, production-runtime or formal-proof
  readiness.

## Alpha criteria (`0.1.x`)

- runnable examples;
- docs/cookbook;
- replay corpus;
- public API review completed;
- feature flags/default features stabilized for alpha;
- no secret/context hygiene failures;
- clear non-goals.

## Beta criteria

- real integration feedback;
- adapter certification;
- performance baseline;
- migration/shadow mode;
- security docs.

## 1.0 criteria

- API freeze;
- semver/compat policy;
- formal/replay release gate;
- operational docs;
- at least one stable runtime adapter path;
- honest statement of proved/tested/bounded/out-of-scope properties.
