---
# csl26-mls1
title: Build multilingual YAML styles for ISO-690, GOST, APA, and JM-Turabian
status: in-progress
type: feature
priority: high
created_at: 2026-02-22T00:00:00Z
updated_at: 2026-02-27T16:25:40Z
---

Build concrete YAML style files and test fixtures for the priority multilingual style families.

**Blocked by:** csl26-mlt2 (processor multilingual resolve must exist first)

**Deliverables:**
- Extend styles/apa-7th.yaml: add multilingual options (combined title-mode, original + [translation])
- Create styles/iso690-author-date.yaml and styles/iso690-numeric.yaml (locale-agnostic, combined title-mode)
- Create styles/gost-r-7-0-5-2008-numeric.yaml and styles/gost-r-7-0-5-2008-author-date.yaml (ru-RU locale, Cyrillic/Latin)
- Create styles/experimental/jm-turabian-multilingual.yaml (CJK script config, given-family name order)
- Extend styles/modern-language-association.yaml with multilingual title translation behavior
- Create tests/fixtures/multilingual/multilingual-cjk.json (Japanese/Chinese/Korean with transliterations)
- Create tests/fixtures/multilingual/multilingual-cyrillic.json (Russian with ALA-LC transliterations)
- Create tests/fixtures/multilingual/multilingual-mixed.json (mixed-language disambiguation test data)

**Success criteria:**
- APA 7th renders non-English title + [English translation] in combined mode
- ISO-690 styles pass basic rendering with multilingual reference data
- GOST styles render Russian and non-Russian references correctly
- MLA style applies original + translated-title pattern consistently
- No regressions in APA 7th oracle scores (8/8 citations, 27/27 bibliography)

**Refs:** docs/architecture/MULTILINGUAL_GROUPING_STYLE_TARGETS.md, csl26-mlt2, docs/architecture/MULTILINGUAL.md

## Summary of Changes

- Added `multilingual` options block to GOST R 7.0.5-2008 author-date + numeric (Cyrl preferred-script, transliterated name-mode)
- Created `styles/iso690-author-date.yaml` and `styles/iso690-numeric.yaml` (locale-agnostic, combined title-mode)
- Created `styles/experimental/jm-turabian-multilingual.yaml` (CJK, given-family order, note format)
- Created `tests/fixtures/multilingual/` with three fixture files: cjk, cyrillic, mixed
- APA 7th oracle: 31/31 bibliography, 13/13 citations — no regression
- PR: https://github.com/citum/citum-core/pull/245
