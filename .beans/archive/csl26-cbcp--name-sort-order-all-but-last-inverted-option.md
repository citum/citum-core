---
# csl26-cbcp
title: 'Name-sort-order: all-but-last-inverted option'
status: completed
type: task
priority: low
tags:
    - schema
    - sorting
    - contributors
created_at: 2026-07-12T15:36:48Z
updated_at: 2026-07-13T10:50:01Z
parent: csl26-kcda
---

CSL schema#134: at least two journals invert all names in an author list
except the last — [Bioscene](http://www.acube.org/bioscene/submission-guidelines/)
and the [African Journal of Food, Agriculture, Nutrition and Development](http://www.ajfand.net/AJFAND/informationtoauthors.html):

> 1. Gardner MN, Halweil AO and JM Nono
> 2. Spencer N Confirmed NameSortOrder
(style.json) has only {family-given, given-family} — a single-name display
direction, not this list-level pattern. NameOrder (a different def) has
{given-first, family-first, family-first-only} — also not a match; that's
about which contributor in a list is inverted (first only), not "all but
the last." Standalone, small — doesn't fit cleanly into another theme.

- [x] Design a new list-level name-order variant (e.g. all-but-last-inverted)

## Summary of Changes

Added `NameOrder::FamilyFirstExceptLast` (`family-first-except-last`), the
list-level display-order variant CSL schema#134 asked for: every contributor
except the last is inverted ("Family, Given"); the last is given-first. "Last"
means the last name of the full contributor list — under et-al truncation
that name may be elided, in which case all rendered names invert.

- Schema: `crates/citum-schema-style/src/template.rs` — new `NameOrder`
  variant, kebab-cased via `rename_all`.
- Engine: `crates/citum-engine/src/values/contributor/names.rs` — threaded
  `total_names` through `NameFormatContext` (2 construction sites) so
  `is_inverted_name_order` can identify the last name; added the match arm.
- Tests: a `format_names` render-path integration test (3-name list, full
  string assertion) in `citum-engine`, and a YAML deserialization test in
  `citum-schema-style` covering the serde rename.
- Regenerated `docs/schemas/style.json` / `server.json` via `just schema-gen`
  (new `const` block only, no version diff).
- No spec added — same class as sibling schema-gap beans (`csl26-5fyz`,
  `csl26-rnrd`), documented by the variant's `///` comment.
- No migrator change — CSL's `name-as-sort-order` has no `except-last`
  construct to map from; this is a Citum-only extension.
