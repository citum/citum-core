# CSL26 E6V4 Schema-First Fixture Architecture Plan

## Purpose

This document defines the follow-on architecture track that was explicitly split
out from `csl26-e6v4`.

`csl26-e6v4` implements a narrow contributor tool that writes the current
oracle-compatible legacy JSON fixtures. It does **not** redesign canonical test
fixture storage.

This document covers that wider redesign so the work remains visible, scoped,
and reviewable as a separate initiative.

## Problem Statement

Current fixture authoring and validation are split across two different
capabilities:

- JS oracle tooling consumes the legacy/CSL-JSON-like fixture at
  `tests/fixtures/references-expanded.json`
- Rust engine loading already accepts both legacy JSON and `InputReference`
  formats

That creates a tooling mismatch:

- contributor editing follows the legacy/oracle format
- core engine evolution prefers schema-aligned `InputReference`
- any attempt to make `csl26-e6v4` schema-first would implicitly widen into
  changes to `scripts/oracle.js`, `scripts/oracle-batch-aggregate.js`,
  contributor docs, and fixture ownership

## Recommendation

Adopt a **schema-first canonical source plus legacy export** model.

This keeps the long-term authoring model aligned with `InputReference` while
preserving `citeproc-js` compatibility during the migration window.

## Goals

- Make schema-aligned fixture authoring possible without breaking current oracle
  workflows
- Preserve deterministic ids and insertion order across generated exports
- Keep batch oracle and single-style oracle working during migration
- Avoid a flag-day conversion of existing contributor workflows

## Non-Goals

- Replacing the oracle fixture format in-place as part of `csl26-e6v4`
- Making `scripts/oracle.js` directly schema-aware in the first step
- Folding export, validation, and authoring migration into the interactive test
  item generator bean

## Proposed Architecture

### Canonical Source

Introduce a schema-first bibliography fixture source under `tests/fixtures/`,
for example:

- `tests/fixtures/references-expanded.source.json`

This source should be authored in a structure that maps directly to
`InputReference` semantics.

### Generated Oracle Artifact

Continue producing:

- `tests/fixtures/references-expanded.json`

This remains the oracle-facing legacy export until the JS oracle stack gains an
internal conversion layer.

### Supporting Tooling

Add two helper scripts in a follow-on implementation:

- `scripts/validate-test-fixtures.js`
- `scripts/export-test-fixtures.js`

Responsibilities:

- `validate-test-fixtures.js`
  - validate the canonical source structure
  - fail clearly on id collisions, malformed dates, or unsupported field shapes
- `export-test-fixtures.js`
  - read canonical schema-first fixtures
  - convert them to the legacy/citeproc-js-compatible shape
  - write deterministic output in the same stable order

## Migration Sequence

1. Define the canonical source filename and JSON shape.
2. Implement validation for the canonical source.
3. Implement deterministic export into
   `tests/fixtures/references-expanded.json`.
4. Update contributor docs to treat the schema-first file as the hand-edited
   source of truth.
5. Update generator ownership so future item creation writes the canonical
   source, not the generated export.
6. Optionally teach `scripts/oracle.js` to consume canonical input through an
   internal adapter layer.
7. Only after the adapter path is stable, decide whether the legacy export
   should remain checked in or become generated-on-demand.

## Interface Boundaries

### Track A: Current Bean

`scripts/generate-test-item.js` continues to:

- write the current legacy/oracle-compatible JSON fixture
- optionally scaffold simple citation scenarios
- optionally run single-style oracle validation

### Follow-On Architecture Track

The schema-first redesign owns:

- canonical fixture format choice
- export contract to legacy oracle format
- oracle and batch tooling compatibility plan
- contributor workflow updates
- ownership rules for source vs generated fixture files

## Acceptance Criteria

This architecture track is ready for implementation when:

- the canonical schema-first file format is explicitly documented
- the export contract to legacy oracle JSON is fully specified
- file ownership is unambiguous (`source` vs `generated`)
- JS oracle compatibility requirements are documented
- contributor workflow and CI/documentation changes are listed concretely

## Decision Summary

- Keep `csl26-e6v4` tactical
- Treat schema-first fixture authoring as a separate architecture initiative
- Prefer source-plus-export over in-place replacement of the current oracle
  fixture format
