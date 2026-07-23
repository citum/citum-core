---
# csl26-dxuo
title: Type-variant positional-delta inheritance (component merge + reorder)
status: draft
type: feature
priority: normal
tags:
    - style
    - inheritance
    - multilingual
created_at: 2026-07-23T15:05:11Z
updated_at: 2026-07-23T15:05:11Z
---

Type-variants inherit via whole-variant replacement per key
(`merge_bibliography_spec`/`merge_citation_spec`,
`crates/citum-schema-style/src/style/overlay.rs`): a child style overriding
one type-variant key must re-author every component of that variant, even
when its actual delta from the parent is purely positional (drop one
component, move another). This is why `gb-t-7714-2025-author-date` couldn't
stay thin the way `gb-t-7714-2025-numeric`/`-note` do — both of those carry
**zero** `bibliography.template`/`bibliography.type-variants` at all,
inheriting the whole per-work recording from `gb-t-7714-2025-base` wholesale
(confirmed by reading both files directly: no such keys present). Author-date
needed a `[N]` citation-number bracket dropped and `issued` moved to right
after the author — nothing else — but had to re-copy all ~13 variants'
components to express it, and the copy silently dropped punctuation and
mis-keyed several selectors (`csl26-6eak`'s "Bug 1"/"Bug 2", fixed by hand
this session by re-deriving from base's exact structure).

Reviewer YDX's framing (GitHub, on the GB/T 7714—2025 PR): the three
in-text/bibliography citation schemes differ, but "the rules on recording
each individual work are identical." Correct, and already the case for
numeric/note. Author-date is the one exception, for a structural reason, not
a content one.

## Proposed mechanism

Extend the type-variants merge to support a genuinely thin positional-delta
overlay instead of requiring full replacement:

- **Component-level merge within a matched key** — override a component's
  fields by position/selector without re-listing the components around it
  (mirrors how `merge_options!`-style per-field merge already works
  elsewhere in this codebase, just not at the template-component-list
  level).
- **Reorder/remove-by-key** — a child variant expressing "drop component X"
  or "move component Y after component Z" as an explicit small directive,
  rather than a full component array.

Needs a design pass before implementation: what's the smallest addressable
unit (component index? a stable per-component id/selector?), how it
interacts with locale-scoped `type-variants` overrides
(`bibliography.locales[].type-variants`), and how the resulting merged
template stays legible in `citum schema` output / debugging.

## Scope note

Bruce flagged (2026-07-23, during `csl26-6eak`'s author-date fidelity work)
that this should be the starting point for a **broader audit of Citum's
inheritance model** generally — not just type-variants. This bean is filed
as that audit's concrete entry point, not a request to solve the whole audit
here.

## References

- `csl26-6eak` — the author-date tuning work that surfaced this; its
  "Root cause found" section documents Bug 1/Bug 2 in full, and its
  2026-07-23 session-progress section is the mechanical fix that worked
  around this gap by hand.
- `crates/citum-schema-style/src/style/overlay.rs:299-319`
  (`merge_bibliography_spec`'s `type-variants` per-key merge) and
  `:210-230` (the citation-side equivalent) — the current whole-variant
  replacement mechanism.
- `crates/citum-schema-style/embedded/styles/gb-t-7714-2025-numeric.yaml`,
  `gb-t-7714-2025-note.yaml` — the two proof-of-concept thin children (no
  `bibliography.template`/`type-variants` at all).

## Checklist

- [ ] Design: smallest addressable unit for a positional-delta overlay
      (component index vs. stable selector vs. named anchor)
- [ ] Design: interaction with locale-scoped `type-variants` overrides
- [ ] Design: legibility in schema output / `citum schema` / debugging
- [ ] Prototype against `gb-t-7714-2025-author-date`'s ~13 variants as the
      first real consumer (retrofit, not net-new scope)
- [ ] Scope the broader Citum-inheritance audit as a follow-up epic once this
      lands
