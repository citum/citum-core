---
# csl26-1861
title: 'migrate: fix eager disambiguation defaults bypassing Processing presets'
status: in-progress
type: task
priority: normal
tags:
    - migrate
    - fidelity
created_at: 2026-06-13T11:55:30Z
updated_at: 2026-06-13T14:20:00Z
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

Resolved:

1. **What should "author-date, unspecified" mean?** Use the opinionated Citum
   B1 default: `year_suffix: true`, `names: false`, `add_givenname: false`.
   This keeps same-author/same-year disambiguation active for author-date
   styles while avoiding automatic name and given-name expansion.
2. **AuthorDate preset variants?** Add named variants so common explicit CSL
   disambiguation shapes fold to clean public values:
   - `author-date`: B1, year suffix only.
   - `author-date-givenname`: year suffix plus given-name expansion.
   - `author-date-names`: year suffix plus name-list expansion.
   - `author-date-full`: year suffix plus both name-list and given-name expansion.
3. **A `citum-migrate` flag for CSL-faithful vs Citum-opinionated extraction?**
   No. Migration uses Citum's class defaults and preserves explicit CSL
   attributes as overrides; it does not add a second extraction mode.

## Scope / impact

- Fidelity-affecting: changing the default changes every author-date style's rendered
  disambiguation, so this **moves the scorecard**. Required gates before accepting:
  - `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings
    && cargo nextest run`
  - `node scripts/report-migrate-sqi.js --corpus random --sample 100 --seed 20260610`
    (headline, no regression)
  - `node scripts/report-core.js > /tmp/r.json && node scripts/check-core-quality.js
    --report /tmp/r.json --baseline scripts/report-data/core-quality-baseline.json`
- Update `test_extract_processing_disambiguation_defaults` and any author-date fixtures to
  the new expected folding.
- Regenerate schemas because `Processing` gains public serialized variants.

## Documentation (do as part of this bean, describing the FIXED behavior)

- [x] Add `docs/reference/PROCESSING_MIGRATION.md`: the CSL→Citum
  `Processing` classification and its class-default disambiguation/sort table;
  how migrate folds to named variants vs custom processing.
- [x] Add one-line pointer from `docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md`
  (where it notes XML is read "only for declarative attributes and options
  (… disambiguation …)"): disambiguation is not synthesized; it is set by
  class-based `Processing` defaults during extraction.
