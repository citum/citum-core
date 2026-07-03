---
# csl26-q05f
title: Expose original-publication fields to render-when
status: in-progress
type: feature
priority: normal
created_at: 2026-07-02T23:42:51Z
updated_at: 2026-07-03T00:29:04Z
parent: csl26-h7oc
blocking:
    - csl26-giun
---

TemplateConditionField (crates/citum-schema-style/src/template.rs:1335) is a closed enum and does not expose original-date / original-publisher / original-publisher-place, so type-variants cannot gate on them with render-when. This blocks the CMOS 18 Reprint / 'Originally published as X (publisher)' trailer cluster deferred from the chicago-author-date bibliography cluster-lift PR (#996, bean csl26-giun) — one of the two deferred pieces accounting for most of the gap between 344/400 and the >=360/400 target.

Scope: add the original-publication fields to TemplateConditionField (plus schema regen via just schema-gen), wire engine condition evaluation, then use them in chicago-author-date-18th.yaml to render the reprint/original-publication trailers. Accessors already exist (original_date, original_publisher_str, original_publisher_place — see csl26-ifhx scrap notes). Note from #996: fixture edition free-text has bespoke oracle capitalization that does not reduce to a simple rule; expect per-item judgment when tuning the trailer text.

## Todo
- [x] Add original-publication variants to TemplateConditionField + schema regen
- [x] Engine: evaluate the new condition fields
- [x] Rust tests per CODING_STANDARDS conventions
- [ ] Wire chicago-author-date-18th reprint/originally-published trailers; measure shared-corpus delta

2026-07-02 scoping correction (pre-implementation): TemplateConditionField already has OriginalPublished -> reference.original_date() (crates/citum-engine/src/processor/rendering/grouped/core.rs:91). The actual enum gap is original-publisher / original-publisher-place (and possibly original-title). Verify the render-side component support for those fields before assuming the condition enum is the only blocker.
