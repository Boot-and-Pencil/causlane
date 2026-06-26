# ADR-0001: Project name and scope

- Status: accepted
- Date: 2026-06-05

## Context

The project needs a working name suitable for a Rust crate family and a clear scope boundary to avoid becoming a workflow engine, queue, scheduler or observability platform.

## Decision

Use `causlane` as the working project/crate name.

Meaning:

```text
causal ordering + execution lanes
```

Scope the project as:

```text
A portable semantic dispatch kernel for typed, auditable, replayable, consequence-aware actions.
```

## Consequences

The name emphasizes causality and lanes, which are core to the model. The scope boundary helps avoid building a bad NIH replacement for existing runtime systems.

## Alternatives considered

- Generic names around `dispatch`, likely crowded and semantically vague.
- Workflow-oriented names, rejected because the project is not a workflow engine.
- Grammar-oriented names, rejected because dispatch/barriers/consequences are equally central.

## Enforcement

Docs and crate descriptions must repeat the non-goals. Adapters may integrate with workflow/job engines but must not force the core to become one.
