---
# csl26-ey6s
title: rust simplify refine grouped citation rendering frontier
status: todo
type: task
priority: high
created_at: 2026-03-15T14:07:43Z
updated_at: 2026-03-15T14:07:43Z
---

Bounded `rust-simplify` then `rust-refine` pass over
`crates/citum-engine/src/processor/rendering/grouped.rs`, focused on separating
group formation, grouped output assembly, and template-processing helpers while
preserving current grouped citation behavior.

## Checklist

- [ ] Run `rust-simplify` on `crates/citum-engine/src/processor/rendering/grouped.rs`
- [ ] Separate author-group formation from grouped-output assembly
- [ ] Separate grouped citation rendering from bibliography/template-processing
      helpers
- [ ] Extract coherent helper modules if needed
- [ ] Preserve grouped citation behavior
- [ ] Add grouped-rendering regressions
- [ ] Run `rust-refine` on the resulting owning module(s)
- [ ] Append a dated progress note to `csl26-l13e`

## Constraints

- Preserve grouped author-date, numeric, legal-case, and integral behavior.
- Do not broaden citation semantics in this bean.
