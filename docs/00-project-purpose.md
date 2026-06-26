# Project purpose

## Problem

Complex systems usually grow through surfaces and mechanisms:

- UI buttons;
- API endpoints;
- CLI commands;
- queue job types;
- workflow steps;
- worker handlers;
- policy rules;
- telemetry events;
- dashboard statuses.

Without a common semantic layer, the same meaningful action becomes multiple private languages. Eventually the system can no longer answer simple questions:

```text
What exactly was the system trying to do?
Who authorized it?
What plan was used?
What consequences were expected?
Why was this allowed to run now?
What actually happened?
Which projection is derived from which truth event?
Can this run be replayed?
```

## Purpose

`causlane` exists to define and enforce a portable protocol for meaningful actions:

```text
Input / command / event
  -> semantic normalization
  -> ActionCall
  -> pure ActionPlan
  -> consequence-aware dispatch
  -> required witnesses / constraints / leases
  -> execution barrier, if needed
  -> controlled execution
  -> observed truth
  -> projection / replay / explanation
```

## Core promise

Define actions once. Get validation, dispatch lifecycle, side-effect barriers, audit events, constraints, replay and explanation across API, CLI, workers and workflow backends.

## What is universal

The project does **not** try to define a universal ontology of all business actions.

The reusable part is the protocol:

```text
intent normalization;
planning contract;
consequence classification;
dispatch admission;
lifecycle transitions;
barriers;
witnesses;
leases;
observed truth;
projections;
replay;
explainability.
```

## What remains domain-specific

Applications define:

- predicates;
- subject and circumstance schemas;
- planners;
- effect signatures;
- policy rules;
- constraint providers;
- operation adapters;
- projections;
- evidence sources;
- business-specific scenarios.
