---
# csl26-xio1
title: Extract Biblatex and external input conversions to dedicated I/O crate
status: completed
type: task
priority: normal
created_at: 2026-05-09T00:00:00Z
updated_at: 2026-06-02T10:43:40Z
---

Currently, `InputReference::from_biblatex()` lives within `citum-engine`. To maintain 
clean architectural boundaries, the engine should strictly focus on processing and 
rendering logic, while I/O and format conversion should be decoupled.

## Scope

- Create a new `citum-io` or `citum-convert` crate, or evaluate expanding 
  `citum-migrate` to handle modern external format ingest.
- Move the `biblatex` dependency and all related conversion logic out of 
  `citum-engine`.
- Update the engine API and any server/binding adapters to use the new conversion 
  pipeline.
- Ensure that the engine remains agnostic of external metadata formats.

## Rationale

Decoupling I/O from the core engine reduces the engine's dependency tree and 
allows format-specific conversion logic to evolve independently of the rendering 
pipeline.

## Summary of Changes

Work completed in c0016cec. Created citum-io crate; moved BibLaTeX mapping and RIS/JSON/YAML loading logic from citum-engine. Decoupled citum-engine from biblatex and ciborium dependencies. Updated CLI and tests.
