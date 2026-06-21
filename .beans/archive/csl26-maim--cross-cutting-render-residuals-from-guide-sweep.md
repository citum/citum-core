---
# csl26-maim
title: Cross-cutting render residuals from guide sweep
status: completed
type: task
priority: normal
created_at: 2026-06-21T10:49:53Z
updated_at: 2026-06-21T17:47:16Z
---

## Summary of Changes

Shipped the four bounded, low-risk cross-cutting engine residuals. Three deferred
items were split into child beans (referenced below).

**1. Page-range delimiter (en-dash ↔ hyphen)** — `format_page_range`
(`crates/citum-engine/src/values/number.rs`) and `apply_range_format`
(`.../values/locator.rs`) now take a `delimiter` sourced from a new style option
`page-range-delimiter` (falling back to the locale's, en-dash default). AMA sets
`-` → `100-108`. Other styles unchanged.

**2. Patent `Patent` term** — `GeneralTerm::Patent` was missing from the hardcoded
runtime locale `Terms::en_us()` (`crates/citum-schema-style/src/locale/types.rs`),
so the `term: patent` component rendered empty. Added it → Chicago author-date
patents render `Patent 7,347,809`.

**3. Month-only date form** — added `DateForm::Month`
(`crates/citum-schema-style/src/template.rs`) + render arms in `format_single_date`
and `format_range_start` (`.../values/date.rs`). Chicago magazine variant uses
`form: month` → `_Wired_, June` (year already supplied by the author-date slot).

**4. entry-suffix DOI/URL terminal-period policy** — the engine hard-coded suffix
suppression after any URL/DOI and ignored the (dead) `suppress-period-after-url`
flag. Added opt-in `entry-suffix-after-url` / `entry-suffix-after-doi` bibliography
options + a `terminal_link` classifier (`crates/citum-engine/src/render/bibliography.rs`).
Default behavior unchanged; IEEE keeps the period after a DOI, MLA after a URL, AMA
suppresses both (now correctly drops the period after `doi:…`).

Verification: full `just pre-commit` green (1663 tests), `just check-core-quality`
passes (154 styles, fidelity=1.0, no regressions), schemas regenerated, `cargo doc`
clean.

## Deferred to child beans

- **csl26-h1ms** — substitute editor label comma+short (`, eds.`); needs a new
  preset variant + per-style work, APA-regression risk.
- **csl26-4kt3** — text-case token preservation (acronyms / proper nouns).
- **csl26-2zy6** — disambiguation strategy (initials vs year-suffix, same-year order);
  needs adjudication against the CSL-faithful design.
