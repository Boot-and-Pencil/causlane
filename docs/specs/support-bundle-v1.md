# Support Bundle v1

M07.6 defines a sanitized JSON artifact for operator support and bug reports:

```bash
causlane support-bundle build \
  --bundle <bundle.json> \
  --trace <trace.json> \
  --graph <graph.yaml|json> \
  --out <support-bundle.json> \
  [--op <action_id>:<op_index>]
```

The command composes existing diagnostic surfaces instead of re-deriving their
logic:

- bundle metadata from the compiled dispatch bundle;
- replay diagnosis from `ReplayTrace::verify_explain`;
- graph context from the M07.2 graph export model;
- repository-owned replay, graph and redaction evidence only; environment
  readiness is emitted separately by `cli-checker` as CI evidence;
- support-bundle redaction from the M07.5 class/profile layer.

## JSON Shape

The top-level object has `schema_version: 1` and these sections:

- `generated_at`: coarse generation timestamp token.
- `bundle`: hash, id, version, schema version, predicate count and merge-protocol
  count.
- `trace`: content hash, action id, optional bundle/predicate/plan hash, event
  count, binding counts and sanitized per-event summaries.
- `replay`: structured replay explain output.
- `graph`: typed graph export model.
- `redaction`: support-bundle field paths revealed/redacted by the M07.5 policy.

Environment/tool readiness is deliberately not part of this product DTO. The
authoritative environment report is produced by `cli-checker project formal
doctor` and attached to the CI evidence bundle. This keeps host state out of
the stable support-bundle contract.

## Sanitization

Support bundles do not embed the raw trace document. Trace subject and
circumstance binding values are represented as counts plus a redacted marker.
Raw authorization-decision payloads, execution-capability payloads and keyed
attestations are not included. Event summaries keep only operational diagnostics:
position, event id, kind, action id, plan hash, counts, booleans and optional
fact/scope/timestamp and payload-presence flags.

The sanitizer uses `RedactionSurface::SupportBundle` with a class allowlist of
`Public` and `Operational`. `Restricted` and `Secret` support-bundle field paths
are redacted by the same fail-closed mechanism as projection redaction; no
support-bundle-specific denylist or second masking engine is defined.

## Limits

Support bundle v1 is a developer/operator artifact, not a formal input and not a
replay authority. It does not replace the compiled bundle, trace, graph snapshot
or formal receipts. Hosts remain responsible for canonical field-path enumeration
and for any value-byte shaping outside this typed summary.
