---
# csl26-gl0n
title: Reconsider date.note rendering config in style
status: todo
type: task
priority: normal
tags:
    - style
    - fidelity
    - dates
    - multilingual
created_at: 2026-07-23T00:11:03Z
updated_at: 2026-07-23T11:12:13Z
---

Reconsider how `note-wrap` (`docs/specs/CALENDAR_DATE_ANNOTATIONS.md`) is
configured and scoped, so a style can control *which* rendered occurrence of
a repeated date variable carries the opaque calendar-note annotation.

## Motivating problem

`DateConfig.note_wrap` (`crates/citum-schema-style/src/options/dates.rs:149`)
is a single flat `Option<WrapConfig>` per section (bibliography/citation/
global). `append_note` (`crates/citum-engine/src/values/date.rs:401-437`,
called from `~line 950`) applies it unconditionally to every date component
rendered in that scope that has a `note` and no per-component override —
there is no per-component override today.

This is fine while a style renders `issued` once per item. It breaks down for
`gb-t-7714-2025-author-date`'s draft (currently reverted, not landed — see
`csl26-6eak`) type-variant shape, which legitimately renders `issued` twice
per item for several types (a short `form: year` in the front author-date
block, a full `form: year-month-day` later in the body, per GB/T §7.5.4.2 for
archival/patent/report/webpage/newspaper types). With the current blanket
option, **both occurrences get the annotation wrapped**, e.g. (observed,
`gbt7714.8.12.3:1`):

```
李鸿章，1887（光绪十三年三月十三日）. 奏请上海道库洋务外销要款无款可筹仍拨药厘接济事：
04-01-35-0399-039[A]. 北京：中国第一历史档案馆，1887（光绪十三年三月十三日）
```

— the same annotated date, twice.

## Decided design (2026-07-23): per-component `suppress-note: true`

Add a plain `suppress_note: Option<bool>` field to `TemplateDate`
(`crates/citum-schema-style/src/template.rs:989`), serialized kebab-case as
`suppress-note`. `append_note` early-returns the unmodified formatted string
when the rendering component has `suppress-note: true`, regardless of the
section's `note-wrap` setting. Absent (the default) renders the note exactly
as today — every currently-landed style is unaffected.

```yaml
- date: issued
  form: year            # front block — carries the note
- date: issued
  form: year-month-day  # body — redundant occurrence
  suppress-note: true
```

This mirrors an existing pattern rather than inventing one: `TemplateTitle`
already carries two per-component boolean opt-ins,
`disambiguate-only` (`template.rs:1059`) and `strip-periods-all`
(`template.rs:1070`) — both plain `Option<bool>`, both
`skip_serializing_if = "Option::is_none"`. `suppress-note` follows the same
shape on `TemplateDate`.

**Why not a `when-form` filter (the option originally proposed):** a single
global form value can't work across the corpus — GB/T doesn't always want the
annotation on `form: year`; the §7.5.4.2 archival/patent/report/webpage types
want it on the full `year-month-day` date, and numeric/note (already at 100%
adjusted fidelity on the 203-item corpus) already annotate whichever single
form they render for those types. Making `when-form` a list, combined with a
local per-component suppression, was also considered — but once a
per-component override exists at all, the list half of that hybrid is
redundant: the component that should carry the note already has no
`suppress-note`, and the one that shouldn't has it. A bare boolean is the
simpler mechanism that fully subsumes the hybrid, and it states intent
directly on the component that shouldn't render it rather than inferring
that from `form` — matching `docs/architecture/DESIGN_PRINCIPLES.md`'s
"explicit over magic" preference.

**Not a revival of `ComponentOverride`.** `csl26-u3zy` (scrapped) removed a
`HashMap<TypeSelector, ComponentOverride>` override map per component in
favor of `type-variants` at the spec level — a type-conditional replacement
mechanism. `suppress-note` is an unconditional plain field on the component
itself, like `disambiguate-only`, not a per-type override map. No relationship
to that scrapped design.

## Clean separation from csl26-6eak

This bean delivers the *mechanism* (schema field + engine gate). `csl26-6eak`
decides the *policy* — which occurrence (front short-year vs. body
full-precision) should carry the annotation for GB/T's dual-date author-date
types, once that convention is confirmed against an authoritative source. The
mechanism supports either answer (mark whichever occurrence is redundant), so
resolving the GB/T convention no longer blocks implementing this bean — only
blocks *using* it in the `gb-t-7714-2025-author-date` style YAML.

## Implementation checklist

- [ ] Add `suppress_note: Option<bool>` to `TemplateDate`
      (`crates/citum-schema-style/src/template.rs`),
      `#[serde(skip_serializing_if = "Option::is_none")]`, with a `///` doc
      comment referencing this bean and the motivating double-annotation case.
- [ ] Gate `append_note` (`crates/citum-engine/src/values/date.rs:401`) on
      it — early-return the unmodified `formatted` string when the
      component's `suppress-note` is `Some(true)`. The owning `TemplateDate`
      is already in scope at the call site (`date.rs:950`).
- [ ] Amend `docs/specs/CALENDAR_DATE_ANNOTATIONS.md` (bump version, add a
      Changelog entry) documenting the per-component opt-out. The spec's
      §Style opt-in currently states note-wrap is "a single style-level
      setting, not a per-`TemplateDate` field, so it never has to be repeated
      across date components" — update that claim to describe the new
      escape hatch precisely (still section-scoped by default; components
      may now opt out individually).
- [ ] Unit test: `get_variable_key`-adjacent coverage or a focused
      `values::date` test asserting `append_note` is a no-op when
      `suppress-note: true` regardless of `note_wrap` being configured.
- [ ] Integration test (`crates/citum-engine/tests/bibliography.rs` or
      `date_annotations.rs`): a template rendering `issued` twice with
      `suppress-note: true` on the second component emits the note exactly
      once, attached to the first.
- [ ] Regenerate schemas (`just schema-gen`) — `citum-schema-style`'s public
      shape changed.
- [ ] Once landed, this unblocks finishing the reverted
      `gb-t-7714-2025-author-date` type-variant recipe in `csl26-6eak`.

## References

- `docs/specs/CALENDAR_DATE_ANNOTATIONS.md` — the active spec this bean
  proposes amending.
- `csl26-6eak` — the author-date tuning work that surfaced this; its
  "New finding 1" section documents the reverted YAML recipe and the
  observed double-annotation output.
- `csl26-u3zy` (scrapped) — the prior per-component override removal; this
  bean is unrelated to it (see "Not a revival" above).
