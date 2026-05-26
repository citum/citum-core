---
# csl26-tap8
title: Support broader per-document configuration overrides
status: completed
type: task
priority: normal
tags:
    - engine
    - schema
created_at: 2026-05-01T14:26:15Z
updated_at: 2026-05-26T23:32:49Z
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

## Spec

docs/specs/PER_DOCUMENT_CONFIG_OVERRIDES.md

## Implementation Plan\n- [x] Commit 1: DocumentOptionsOverride struct + frontmatter parsing\n- [x] Commit 2: Apply overrides in pipeline\n- [x] Commit 3: CLI --locale flag


## Summary of Changes

- `DocumentBibliographyOverride` and `DocumentOptionsOverride` structs with serde
  kebab-case + deny_unknown_fields; schema feature-gated JsonSchema derives.
- `Style::apply_scoped_options()` public method bridges engine → schema crate.
- `DocumentFrontmatter.options` field wired through `ParsedDocument.frontmatter_options`.
- Legacy `integral-name-memory:` suppressed when `options.integral-name-memory` is set.
- Pipeline applies bibliography overrides via `processor_with_bibliography_override`.
- CLI `--locale` / `-L` flag on `render doc`; frontmatter `options.locale` is the fallback.
- 9 new unit tests covering deserialization, apply, and precedence behaviour.
