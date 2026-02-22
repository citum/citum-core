---
# csl26-grp1
title: Build sectional bibliography YAML styles for Chicago, Juris-M legal, and GOST
status: in-progress
type: feature
priority: normal
created_at: 2026-02-22T00:00:00Z
updated_at: 2026-02-22T12:00:00Z
---

Build concrete YAML style files demonstrating bibliography grouping for Chicago primary/secondary,
Juris-M legal hierarchy, and GOST category sections. Schema and processor infrastructure are complete.

**No blocking dependencies** (csl26-group is completed).

**Deliverables:**
- Extend styles/chicago-author-date.yaml: add groups for Primary Sources / Archival / Secondary
- Create styles/experimental/jm-chicago-legal.yaml: groups for Cases / Statutes / Treaties / Secondary
- Extend styles/experimental/multilingual-academic.yaml: add disambiguate: locally per language group
- Create styles/gost-r-7-0-5-2008-*.yaml: add category groups (books/articles/legal/other) in Russian headings
- Create tests/fixtures/grouping/primary-secondary.json: manuscripts + journal articles for Chicago grouping
- Add integration tests using process_document() API to validate group rendering (cited_ids populated)
- Validate per-group year suffix restart (disambiguate: locally) with mixed-year fixture

**Note:** Direct csln process command does not populate cited_ids; integration tests MUST use
process_document() to ensure grouping logic runs correctly.

**Refs:** docs/architecture/MULTILINGUAL_GROUPING_STYLE_TARGETS.md, csl26-group, docs/architecture/design/BIBLIOGRAPHY_GROUPING.md
