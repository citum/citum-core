---
# csl26-1861
title: 'migrate: fix eager disambiguation defaults bypassing Processing presets'
status: todo
type: task
priority: normal
tags:
    - migrate
    - fidelity
created_at: 2026-06-13T11:55:30Z
updated_at: 2026-06-13T11:55:30Z
---

Deferred from the synthesis-loop PR (bean csl26-8txa). Surfaced while documenting
how migration sets disambiguation rules: the answer is that defaults are *class-based*
(author-date / numeric / note), defined by `Processing::config()` in
`crates/citum-schema-style/src/options/processing.rs`. But the migrate options-extractor
sets those defaults in a way that defeats the named presets and invents behavior the
source CSL never requested.

## The defect

The extractor *eagerly materializes* disambiguation defaults, then compares the
materialized struct against the preset — so they can never match. Three "defaults"
exist and no two agree:

| source | names | add_givenname | year_suffix |
|---|---|---|---|
| CSL 1.0 spec (attribute defaults) | false | false | **false** |
| migrate extraction (`unwrap_or`) | false | false | **true** |
| schema `AuthorDate.config()` | **true** | **true** | true |

Two problems fall out:

1. **Preset bypass.** Because extraction fills `{false,false,true}` and the preset is
   `{true,true,true}`, `fold_to_named_processing` can never collapse a normal author-date
   style to `processing: author-date`. The named preset — whose purpose is the clean
   common case — is effectively dead; every author-date style emits a verbose `!custom`
   block.
2. **Invented disambiguation (fidelity-relevant).** `year_suffix: unwrap_or(true)`
   ([`processing.rs:73`](../crates/citum-migrate/src/options_extractor/processing.rs))
   *adds* year-suffix disambiguation the source CSL never set (CSL default is false;
   citeproc-js would do nothing). migrate invents behavior, defended by a heuristic
   comment rather than the source style.

### Evidence

- `test_extract_processing_disambiguation_defaults` (`crates/citum-migrate/src/options_extractor/tests.rs:198`)
  pins the current behavior: a bare author-date style (no `disambiguate-*` attrs)
  extracts to `Processing::Custom { disambiguate: {names:false, add_givenname:false,
  year_suffix:true} }`, asserted **not** `AuthorDate`.
- Extraction logic: `crates/citum-migrate/src/options_extractor/processing.rs:54-87`
  (`is_author_date` branch) and `fold_to_named_processing` at `:97`.
- Preset source of truth: `Processing::config()` in
  `crates/citum-schema-style/src/options/processing.rs:215` (author-date `{true,true,true}`,
  numeric none, note `{true,false,false}`).

## Fix direction

**Delta-based extraction.** When the CSL omits a `disambiguate-*` attribute, leave it
`None` and let the named preset / engine supply the default; record only attributes the
style *explicitly* sets and that *differ* from the preset. A bare author-date style then
extracts to "no overrides" and folds straight to `processing: author-date`. This is the
same delta philosophy `docs/specs/STYLE_PRESET_ARCHITECTURE.md` already uses for `extends:`.

## Open design questions (decide before implementing)

1. **What should "author-date, unspecified" mean?** Owner leans **(b) opinionated Citum
   default** (keep year-suffix on, on the theory that author-date styles want it) over
   **(a) CSL-faithful** `{false,false,false}`. Whichever wins becomes the single canonical
   default used by *both* the preset and extraction, so they agree and fold cleanly.
2. **AuthorDate preset variants?** A single `AuthorDate` preset may be too coarse — consider
   named variants (e.g. author-date with vs without year-suffix / given-name expansion) so
   common real-world shapes each fold to a clean named value instead of `!custom`. Scope
   this against the actual distribution of disambiguation configs in the corpus.
3. **A `citum-migrate` flag for CSL-faithful vs Citum-opinionated extraction?** Possibly a
   conversion flag toggling (a) vs (b) so strict-fidelity round-trips and opinionated
   defaults are both reachable. Unsure it's worth the surface area — evaluate, don't assume.

## Scope / impact

- Fidelity-affecting: changing the default changes every author-date style's rendered
  disambiguation, so this **moves the scorecard**. Required gates before accepting:
  - `node scripts/report-migrate-sqi.js --corpus random --sample 100 --seed 20260610`
    (headline, no regression)
  - `node scripts/report-core.js > /tmp/r.json && node scripts/check-core-quality.js
    --report /tmp/r.json --baseline scripts/report-data/core-quality-baseline.json`
- Update `test_extract_processing_disambiguation_defaults` and any author-date fixtures to
  the new expected folding.

## Documentation (do as part of this bean, describing the FIXED behavior)

- New `docs/reference/` doc: the CSL→Citum `Processing` classification and its class-default
  disambiguation/sort table; how migrate folds to named variants vs `!custom`.
- One-line pointer from `docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md` (where it notes XML
  is read "only for declarative attributes and options (… disambiguation …)"): disambiguation
  is not synthesized; it is set by class-based `Processing` defaults during extraction.
