---
# csl26-7hsx
title: 'GB/T bilingual template: locale type-variants don''t fall back to section type-variants'
status: completed
type: bug
priority: high
tags:
    - punctuation
    - engine
    - multilingual
created_at: 2026-07-21T13:51:49Z
updated_at: 2026-07-21T14:02:05Z
parent: csl26-0ugp
---

resolve_localized_type_variant callers pass None for the section-level type_variants fallback tier, so English article-journal/webpage/report/etc. items in gb-t-7714-2025 (whose type isn't redefined in the en locale override) render via the flat locale template instead of the correctly-delimited section-level type-variant, producing garbled bibliography entries.

## Summary of Changes

Fixed by passing `spec.type_variants.as_ref()` (or `bib_spec.type_variants.as_ref()`) instead of `None` as the section-level fallback argument to `resolve_localized_type_variant` at its four call sites (`processor/rendering/mod.rs:588`, `processor/rendering/grouped/core.rs:637,672,886`). This revives the function's already-documented three-tier fallback (locale type-variant → section type-variant → locale flat template), which had been dead code in tier 2 since all callers passed `None`.

Added a regression test (`given_gb_t_numeric_style_when_rendering_english_article_journal_then_type_variant_is_not_dropped` in `crates/citum-engine/tests/multilingual.rs`) pinning the exact corrected output for the reported "Coffee drinking and cancer of the pancreas" case.

Verified via `just pre-commit` (2109 tests, fmt + clippy clean) and an oracle re-run: gb-t-7714-2025-numeric raw bibliography matches rose from 143/203 to 146/203, with the remaining tracked ids in csl26-d3hs now failing only on the separately-registered Latin-punctuation convention divergence, not structural content.
