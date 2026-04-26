# Document Input Parser Boundary Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-03-15
**Supersedes:** None
**Related:** `csl26-ykno`, `csl26-5zzb`, `docs/specs/PANDOC_MARKDOWN_CITATIONS.md`, `docs/specs/NOTE_STYLE_DOCUMENT_NOTE_CONTEXT.md`

## Purpose
Define the boundary between format-specific document parsers and the shared
document-processing pipeline so Djot remains the first fully featured adapter
while future Markdown and Org-mode adapters can plug into the same downstream
processing model without redesigning note, bibliography, or disambiguation
logic.

## Scope
In scope:
- The parser adapter contract for document input formats
- The shared parsed-document handoff model
- Responsibility boundaries for frontmatter, bibliography blocks, note/manual
  note extraction, and HTML finalization
- Design constraints for future Markdown and Org-mode adapters

Out of scope:
- Implementing new Markdown or Org-mode behavior in this wave
- Changing citation semantics or note-style policy
- Replacing Djot syntax or adding a format-agnostic AST

## Design
The document-processing system must have two layers:

1. A format-specific adapter layer that understands source syntax.
2. A shared pipeline layer that consumes normalized document facts.

The existing `CitationParser` contract remains the main entry point, but it is
treated as a document-adapter contract rather than a “find citations” helper.
Each adapter is responsible for extracting source-specific structure and
returning one normalized `ParsedDocument`.

### Adapter responsibilities
Each document adapter owns:
- source-syntax citation parsing
- source-syntax note/manual-footnote discovery
- source-syntax frontmatter parsing
- source-syntax bibliography block discovery
- source-syntax-to-HTML finalization when HTML output requires a format-native
  final conversion step

Djot-specific examples that remain adapter-owned:
- Djot citation token parsing
- Djot heading/scope scanning
- Djot bibliography block syntax (`::: bibliography`)
- Djot frontmatter stripping and body offset calculation
- Djot-to-HTML conversion via `jotdown`

### Shared pipeline responsibilities
The shared pipeline owns:
- applying parsed citations to rendered output
- note numbering and shared note-stream behavior
- generated-note rendering
- bibliography rendering and placement orchestration
- placeholder staging and replacement for output formats
- use of parsed frontmatter groups and integral-name overrides after extraction

The shared pipeline must not inspect Djot tokens or Djot parser internals.

### Parsed-document handoff contract
`ParsedDocument` is the normalized handoff object between adapter and pipeline.
It must contain:
- parsed citations in source order
- citation placement metadata
- citation structure metadata needed downstream
- manual note ordering and note references
- bibliography block directives
- frontmatter-derived bibliography groups
- frontmatter-derived integral-name override
- body start offset for the original source

Future adapters may return empty values for unsupported capabilities, but they
must use the same fields rather than inventing adapter-specific side channels.

### Bibliography blocks
Bibliography block detection is adapter-owned because the syntax is
format-specific. Block replacement and rendering orchestration remain
pipeline-owned because the rendered bibliography behavior is format-neutral once
block positions are normalized.

### Frontmatter
Frontmatter parsing is adapter-owned because frontmatter syntax varies by input
format. The adapter must convert recognized metadata into the normalized
`ParsedDocument` fields used by the pipeline.

### Notes and manual footnotes
Manual-note detection is adapter-owned because note syntax is format-specific.
Generated-note numbering and note-stream behavior remain shared pipeline
responsibilities and must continue to follow
`docs/specs/NOTE_STYLE_DOCUMENT_NOTE_CONTEXT.md`.

### Future adapters
Markdown and Org-mode are explicit future targets. This spec does not require
them to support Djot feature parity in v1, but it does require that they plug
into the same `ParsedDocument` handoff and shared pipeline.

That means:
- a new adapter may initially leave bibliography blocks or manual notes empty
- the shared pipeline must still accept the adapter without format-specific
  branching outside the adapter contract
- adding a second adapter must not require redesigning note numbering,
  bibliography placement, or citation rendering contracts

## Implementation Notes
The current `CitationParser` trait already points in the right direction. This
wave should refine the surrounding module structure so Djot-specific parsing
helpers move behind a clearer adapter seam and `pipeline.rs` depends only on
normalized document data.

Prefer crate-private adapter helpers and crate-private normalized helper types.
Do not widen visibility unless another module truly needs it.

The Markdown parser added for Pandoc citations should remain lightweight in this
wave. It is evidence that the adapter model is viable, not a parity target.

## Acceptance Criteria
- [ ] A committed Draft spec defines the parser adapter boundary before the
      Djot pipeline refactor starts
- [ ] Djot-specific parsing concerns are isolated behind an adapter seam
- [ ] Shared document-processing code depends on normalized parsed-document data
      rather than Djot syntax details
- [ ] Frontmatter, bibliography blocks, and manual-note extraction have explicit
      ownership rules
- [ ] A future Markdown or Org-mode adapter can plug into the same pipeline
      without redesigning note/disambiguation core logic
- [ ] Existing Djot document behavior remains unchanged after the refactor

## Changelog
- v1.0 (2026-03-15): Initial draft.
