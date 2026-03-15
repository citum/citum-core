---
# csl26-ey6s
title: rust simplify refine grouped citation rendering frontier
status: completed
type: task
priority: high
created_at: 2026-03-15T14:07:43Z
updated_at: 2026-03-15T15:22:22Z
---

Bounded `rust-simplify` then `rust-refine` pass over
`crates/citum-engine/src/processor/rendering/grouped.rs`, focused on separating
group formation, grouped output assembly, and template-processing helpers while
preserving current grouped citation behavior.

## Checklist

- [x] Run `rust-simplify` on `crates/citum-engine/src/processor/rendering/grouped.rs`
- [x] Separate author-group formation from grouped-output assembly
- [x] Separate grouped citation rendering from bibliography/template-processing
      helpers
- [x] Extract coherent helper modules if needed
- [x] Preserve grouped citation behavior
- [x] Add grouped-rendering regressions
- [x] Run `rust-refine` on the resulting owning module(s)
- [x] Append a dated progress note to `csl26-l13e`

## Constraints

- Preserve grouped author-date, numeric, legal-case, and integral behavior.
- Do not broaden citation semantics in this bean.

## Summary of Changes

Task 2 completed: Grouped citation rendering refactor.

- Split grouped.rs into grouped/core.rs (950 lines of renderer impl) and grouped/grouping.rs (60 lines for author grouping logic)
- Made TemplateRenderRequest public in rendering/mod.rs to support split impl blocks
- Removed duplicate resolve_component_for_ref_type function from mod.rs (moved to grouped/core.rs)
- Added regression tests for grouped citation modes: author-date grouping, numeric item order, integral mode

All 709 tests passing.
