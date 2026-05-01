---
# csl26-tap8
title: Support broader per-document configuration overrides
status: todo
type: task
priority: normal
tags:
    - engine
    - schema
created_at: 2026-05-01T14:26:15Z
updated_at: 2026-05-01T14:26:15Z
---

Extend the document frontmatter parsing to support broader configuration
overrides beyond just `bibliography` groups and `integral-names`.

## Context

Currently, `sort-partitioning` and other behavioral settings only live on
the `Style`'s `BibliographyOptions` or `Config`. A user cannot override
these on a per-document basis without modifying the style.

## Required Changes

- Extend `DocumentFrontmatter` in `crates/citum-engine/src/processor/document/djot/parsing.rs`
  to allow an `options` block (analogous to `BibliographyOptions` or
  `CitationOptions`).
- Update `ParsedDocument` in `crates/citum-engine/src/processor/document/types.rs`
  to hold these new configuration overrides.
- Update `Processor` (likely in `setup.rs`) to merge document-level
  configuration overrides into the style's default configuration.


