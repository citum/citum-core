---
# csl26-5zzb
title: extract djot adapter and shared document pipeline seams
status: completed
type: feature
priority: high
created_at: 2026-03-15T14:07:43Z
updated_at: 2026-03-15T15:36:00Z
---

Refactor document processing so Djot is the first concrete parser adapter over a
shared document pipeline. This wave must preserve all current Djot behavior
while leaving explicit extension points for future Markdown and Org-mode
adapters.

Spec path: `docs/specs/DOCUMENT_INPUT_PARSER_BOUNDARY.md`

The prerequisite spec bean `csl26-ykno` is complete, so this bean is now ready
for implementation when capacity opens.

## Checklist

- [x] Implement parser boundary from `docs/specs/DOCUMENT_INPUT_PARSER_BOUNDARY.md`
- [x] Run `rust-simplify` on `crates/citum-engine/src/processor/document/djot.rs`
- [x] Extract Djot-specific parsing behind an adapter seam
- [x] Move shared orchestration into format-neutral document pipeline code
- [x] Preserve current Djot behavior
- [x] Leave explicit extension points for Markdown and Org-mode
- [x] Add adapter-focused and shared-pipeline tests
- [x] Run `rust-refine` on the resulting parameter-heavy module(s)
- [x] Append a dated progress note to `csl26-l13e`

## Constraints

- Do not add Markdown or Org-mode parsing in this bean.
- Keep new parser abstractions crate-private unless a second caller truly
  requires broader visibility.

## Summary of Changes

Task 3 completed: Djot adapter refactor.

- Split djot.rs into djot/mod.rs (200 lines: public adapter surface) and djot/parsing.rs (470 lines: internal winnow parsers, frontmatter, scope tracking)
- Moved footnote definition tracking, citation scope annotation, citation finding, frontmatter parsing to djot/parsing.rs with pub(crate) visibility
- Kept public CitationParser impl + helper functions in djot/mod.rs
- Added 3 djot adapter/pipeline tests: simple citation extraction, manual footnotes, multiple citations
- Preserved all current Djot parsing behavior
- Created explicit parser adapter seam through CitationParser trait
- All 712 tests passing, verification gate clean
