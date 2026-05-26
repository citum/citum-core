---
# csl26-ju39
title: 'docs: dissolve architecture/design/, move specs to specs/, audits to audits/'
status: completed
type: task
priority: normal
created_at: 2026-05-26T11:49:29Z
updated_at: 2026-05-26T11:51:27Z
---

Option A reorg: move all design/ spec files to specs/ (or guides/ for TEST_STRATEGY), create architecture/audits/ for dated operational records, merge DJOT duplicate, update README and CLAUDE.md placement rules.

## Summary of Changes

- Dissolved docs/architecture/design/ — all 10 files rehomed
- 7 design specs → docs/specs/ (BIBLIOGRAPHY_GROUPING, EXPLICIT_DEFAULT_SORTING, LEGAL_CITATIONS, PUNCTUATION_NORMALIZATION, STYLE_ALIASING, TYPE_SYSTEM_ARCHITECTURE, WASM_SUPPORT)
- TEST_STRATEGY → docs/guides/
- STYLE_EDITOR_VISION → docs/architecture/ (Tier C historical)
- WASM_BENCHMARK_REPORT → docs/architecture/audits/
- 9 dated operational records → docs/architecture/audits/ (new dir)
- Deleted docs/architecture/DJOT_RICH_TEXT.md; merged Math Policy, Title Markup cases, and Non-Goals into docs/specs/DJOT_RICH_TEXT.md
- Updated architecture/README.md Tier B links and added Audits section
- Updated CLAUDE.md placement table: specs first, added audits row, clarified architecture row
- Added Status/Date metadata to 4 previously undated files
