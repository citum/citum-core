---
# csl26-2zy6
title: 'Disambiguation strategy: initials/added-names + same-year order'
status: completed
type: task
priority: normal
created_at: 2026-06-21T17:46:31Z
updated_at: 2026-06-21T21:15:23Z
---

Deferred from csl26-maim. Year-suffix appears where guides (MLA/APA) use initials/added names; same-surname/year ordering wrong (Garcia 2019b before 2019a). Contradicts the deliberate CSL-faithful design in project_disambiguation_defaults; needs adjudication before code. Audit rows 114/138/173.

## Summary of Changes

Adjudicated and fixed the three guide-conformance disambiguation defects (audit
rows 114/138/173). **No engine default change** — the CSL-faithful default
(`add_givenname: false`) stays; the defects were missing per-style flags plus one
engine sort bug. See [[project_disambiguation_defaults]] and
`docs/specs/DISAMBIGUATION.md` §3, §3.1.

**Row 138 — same-year suffix order (engine).** `build_reference_cache`
(`crates/citum-engine/src/processor/disambiguation.rs`) keyed the year-suffix sort
on a raw `title.to_lowercase()`, so a leading article flipped `2019a`/`2019b`. Now
uses `sort_support::title_sort_key` (article-stripped, locale-collated) so suffix
order follows the effective bibliography sort.

**Row 114 — APA same-surname authors.** `apa-7th.yaml` used the bare `author-date`
preset (givenname off) → spurious `2020a`/`2020b`. Switched to a custom `processing`
block: `add-givenname` + `givenname-rule: primary-name-with-initials` (global
detection so initials appear in *all* in-text cites, APA §8.20) + explicit
`bibliography.sort: author-date-title` (custom processing no longer supplies the
preset default sort). Renders `(A. Johnson, 2020)` / `(B. Johnson, 2020)`.

**Row 173 — MLA.** `modern-language-association.yaml` bare preset turned year-suffix
on; MLA is author-page. Set `year-suffix: false` with `names`/`add-givenname` on;
same-author works now resolve via the existing `disambiguate-only` short title
(`(Garcia, "Rivers of Time")`), no suffix.

**Tests** (`crates/citum-engine/tests/citations.rs`):
`year_suffix_follows_article_stripped_title_order`,
`givenname_expansion_preferred_over_year_suffix`,
`primary_name_initials_expand_globally_across_citations`,
`year_suffix_off_emits_no_letter`. Updated `test_bibliography_per_group_disambiguation`
(its old expectation encoded the row-138 misalignment).

**Docs:** rewrote `docs/specs/DISAMBIGUATION.md` §3 + added §3.1 (per-guide
application); corrected cascade order + APA example + Known Limitations in
`docs/reference/DISAMBIGUATION.md`.

**Verification:** `just pre-commit` green (1667 tests), `cargo doc` clean,
`just check-core-quality` 154 styles fidelity=1.0 (no regression), APA/MLA/Chicago
oracle unchanged vs baseline.
