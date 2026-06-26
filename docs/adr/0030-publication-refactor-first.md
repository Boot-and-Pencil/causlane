# ADR-0030: Publication Preparation Starts With Refactoring

## Status

Accepted.

## Context

Causlane is approaching first public crates.io publication. The repository has
formal, replay and runtime scaffolding, but it also carries historical milestone
vocabulary, broad public re-exports, generated-artifact policy complexity and
private checkpoint history.

Publishing a crate version is irreversible for that version: source can be
yanked from dependency resolution, but it cannot be overwritten or deleted from
crates.io.

## Decision

Run a dedicated publication refactor stage before public GitHub opening or
crates.io publication.

The stage must not add new runtime semantics. It may restructure modules,
improve names, remove duplication, narrow accidental public API and clarify
crate boundaries.

## Consequences

- The first public source snapshot is more maintainable.
- Public API is less accidental.
- Humans and AI agents receive clearer boundaries.
- Publication waits until the quality baseline is acceptable.

