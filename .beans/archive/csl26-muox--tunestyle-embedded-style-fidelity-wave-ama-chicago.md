---
# csl26-muox
title: 'tune(style): embedded style fidelity wave — AMA + Chicago-18th'
status: completed
type: task
priority: high
created_at: 2026-06-24T15:21:52Z
updated_at: 2026-06-24T20:22:30Z
---

Tune two embedded styles to 100% oracle fidelity:

## Scope
- american-medical-association: 87% → 100% (9 bibliography failures — new TLIB-SEL-* types)
- chicago-author-date-18th: 78% → as high as tractable (121 failures, CMOS18 benchmark)

## AMA Failures (root cause: missing type variants for 8 new ref types)
- TLIB-SEL-DICT-1 (entry-dictionary): missing template; oracle needs In: container, access date, URL
- TLIB-SEL-LEGISLATION-1 (legislation): missing URL + volume rendering
- TLIB-SEL-MAP-1 (map): falls through to default (publisher); needs extends: dataset
- TLIB-SEL-STANDARD-1 (standard): needs standard-number + Published online date pattern + doi
- TLIB-SEL-BILL-1 (bill): missing URL
- TLIB-SEL-HEARING-1 (hearing): missing Published online + URL
- TLIB-SEL-SOFTWARE-1 (software): missing Published online + URL
- TLIB-SEL-REGULATION-1 (regulation): missing code + volume; 2009;45 format

## Styles Completed
- [x] american-medical-association: 100% bib + cit
- [x] chicago-notes-18th: 100% cit
- [x] chicago-shortened-notes-bibliography: 100% bib + cit
- [x] chicago-author-date-18th: 100% bib + cit
- [x] modern-language-association: 100% bib + cit
- [x] elsevier-harvard: 100% bib + cit

## Todo
- [x] Create branch
- [x] Fix AMA: 7 new type variants
- [x] Fix chicago-notes-18th translator label
- [x] Fix chicago-shortened-notes-bibliography: chapter, entry-dictionary, personal-communication, patent, standard
- [x] Fix chicago-author-date-18th: entry-dictionary + standard
- [x] Fix MLA: entry-dictionary + standard
- [x] Fix elsevier-harvard: entry-dictionary + standard
- [x] Run pre-commit gate (1672/1672 tests pass)
- [x] Update TIER_STATUS.md
- [x] Commit + open PR

## Summary of Changes

Added missing type variants to 6 embedded core styles to achieve 100%
oracle fidelity against the expanded references-expanded.json fixture
(47 bibliography + 20 citation test cases).

Styles completed:
- **AMA**: 7 new type variants (entry-dictionary, legislation, map,
  standard, bill, hearing, software, regulation)
- **chicago-notes-18th**: translator verb-form fix (cit: 100%)
- **chicago-shortened-notes-bibliography**: chapter, entry-dictionary,
  personal-communication suppression, patent, standard
- **chicago-author-date-18th**: entry-dictionary (fixed case + added
  accessed date/URL), standard
- **MLA**: entry-dictionary (title-case, accessed date), standard
- **elsevier-harvard**: entry-dictionary, standard

Commit: 264da0c
