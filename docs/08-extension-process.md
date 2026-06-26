# Extension process

Every new meaningful action family must enter through the registry and contract pipeline.

## New predicate checklist

1. Add predicate.
2. Add subject schema.
3. Add circumstance schema.
4. Assign consequence profile.
5. Derive route from profile.
6. Assign lifecycle class.
7. Declare effect signatures.
8. Declare required witnesses.
9. Declare constraints, claims and leases.
10. Declare barrier/truth/projection policy.
11. Declare overlays.
12. Add protocol scenario.
13. Add replay expectation.
14. Add formal obligations or explicit gap.
15. Update coverage matrix.
16. Add adapter tests if execution-bearing.

## New adapter checklist

1. Identify port implemented.
2. Define failure semantics.
3. Define idempotency behavior.
4. Define capability/lease handling.
5. Define audit ordering guarantees.
6. Add certification scenarios.
7. Add replay fixtures.
8. Add observability/redaction behavior.
9. Document unsupported consequence profiles.

## Outside-kernel behavior

A feature may remain outside-kernel only if it is explicitly marked as such.

Outside-kernel behavior must not:

- claim observed truth;
- mutate canonical lifecycle;
- bypass hard-effect barrier;
- satisfy gates silently;
- become a hidden production path.
