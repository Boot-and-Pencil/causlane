# ADR-0021: Operational SLOs are a typed measurement catalog

## Status

Accepted.

## Context

M09.1 established benchmark coverage without pass/fail thresholds. M09.2-M09.5
then stabilized partition admission, batched audit append, backpressure policy
and plan/template cache identity. M09.6 needs a durable operational contract for
what must be measured without making deployment-specific latency promises in the
repo.

## Decision

`causlane-runtime` exposes `M09_6_OPERATIONAL_SLO_METRICS` as the single
machine-readable catalog for operational readiness metrics. Each entry fixes:

- stable metric id;
- runtime or replay surface;
- measurement kind and unit;
- required percentile for latency metrics;
- signal source boundary;
- threshold ownership policy.

The catalog requires `p50` and `p95` latency metrics for submit, admission,
barrier append, replay verify and replay explain. It also requires gauges for
partition queue depth and constraint snapshot stale age.

`validate_operational_slo_catalog` is the executable structural check. It
rejects duplicate metric ids, missing required metrics and shape drift from the
canonical catalog.

## Consequences

- Docs and release gates reference the typed catalog instead of maintaining a
  parallel JSON artifact.
- Numeric thresholds are `HostDefined`: release profiles and deployments own
  concrete target values because they depend on adapter and telemetry backend
  choices.
- OpenTelemetry export behavior remains fail-open and unchanged.
- No host-dispatch v2 schema, replay trace schema, Formal IR schema, generated
  model or scenario changes are introduced.
- M09.6 does not add runtime rate limiting, queue-depth enforcement,
  stale-snapshot rejection or new replay semantics.
