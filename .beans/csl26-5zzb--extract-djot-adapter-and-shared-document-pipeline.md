---
# csl26-5zzb
title: extract djot adapter and shared document pipeline seams
status: todo
type: feature
priority: high
created_at: 2026-03-15T14:07:43Z
updated_at: 2026-03-15T14:07:43Z
---

Refactor document processing so Djot is the first concrete parser adapter over a
shared document pipeline. This wave must preserve all current Djot behavior
while leaving explicit extension points for future Markdown and Org-mode
adapters.

Spec path: `docs/specs/DOCUMENT_INPUT_PARSER_BOUNDARY.md`

The prerequisite spec bean `csl26-ykno` is complete, so this bean is now ready
for implementation when capacity opens.

## Checklist

- [ ] Implement parser boundary from `docs/specs/DOCUMENT_INPUT_PARSER_BOUNDARY.md`
- [ ] Run `rust-simplify` on `crates/citum-engine/src/processor/document/djot.rs`
- [ ] Extract Djot-specific parsing behind an adapter seam
- [ ] Move shared orchestration into format-neutral document pipeline code
- [ ] Preserve current Djot behavior
- [ ] Leave explicit extension points for Markdown and Org-mode
- [ ] Add adapter-focused and shared-pipeline tests
- [ ] Run `rust-refine` on the resulting parameter-heavy module(s)
- [ ] Append a dated progress note to `csl26-l13e`

## Constraints

- Do not add Markdown or Org-mode parsing in this bean.
- Keep new parser abstractions crate-private unless a second caller truly
  requires broader visibility.
