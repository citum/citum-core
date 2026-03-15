---
# csl26-ykno
title: spec document input parser boundary for djot markdown org
status: completed
type: task
priority: high
created_at: 2026-03-15T14:07:43Z
updated_at: 2026-03-15T14:07:43Z
---

Create the required parser-boundary spec before any non-trivial document parser
refactor. Djot remains the only implemented input format in this wave, but the
design must leave a clean extension path for future Markdown and Org-mode
adapters.

This bean gates `csl26-5zzb`.

## Checklist

- [x] Create `docs/specs/DOCUMENT_INPUT_PARSER_BOUNDARY.md`
- [x] Define the parser-adapter vs shared-pipeline boundary
- [x] Define the parsed-document handoff contract
- [x] Decide ownership of frontmatter parsing
- [x] Decide ownership of bibliography block detection
- [x] Decide ownership of note/manual-note extraction
- [x] Document Djot-only assumptions that must not leak into shared layers
- [x] Reserve an explicit extension path for Markdown and Org-mode adapters
- [x] Add acceptance criteria for plugging in a second adapter without
      rewriting note/disambiguation core logic

## Notes

- Follow `docs/specs/README.md` and commit the spec with `Status: Draft` before
  implementation code.
- The resulting spec path must be referenced in `csl26-5zzb` and in the PR
  description.

## Summary of Changes

- Added `docs/specs/DOCUMENT_INPUT_PARSER_BOUNDARY.md` in `Draft` status.
- Defined adapter-owned responsibilities for frontmatter, bibliography blocks,
  note extraction, and HTML finalization.
- Defined the normalized `ParsedDocument` handoff contract and the constraint
  that future Markdown and Org-mode adapters must plug into the same shared
  pipeline.
