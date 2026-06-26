# ADR-0032: Curated Public Git History

## Status

Accepted.

## Context

The private development history contains checkpoint commits and exploratory
iterations. Opening that history as-is may expose noise, private context or
misleading authorship.

## Decision

The first public GitHub repository should use curated public history.

Preferred: a small set of meaningful commits grouped by architecture, kernel
contracts, replay/formal/runtime, devinfra and publication readiness.

Fallback: one signed clean baseline commit when multi-commit curation is too
costly.

## Consequences

- The public repository starts with readable provenance.
- Private development history can remain archived internally.
- No public history rewrite is needed after first public release.

