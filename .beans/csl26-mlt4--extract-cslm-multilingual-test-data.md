---
# csl26-mlt4
title: Extract CSL-M multilingual test data
status: completed
type: task
priority: low
created_at: 2026-02-12T00:00:00Z
updated_at: 2026-03-01T17:00:00Z
---

Extract CJK/Arabic/Russian test cases from Juris-M/jm-styles repository for multilingual processor validation.

Focus on styles without legal extensions. Store in tests/fixtures/multilingual/

## Completed Items

- [x] Extract name_AsianGlyphs.txt (Japanese: 我妻栄 author name) → multilingual-cjk.json
- [x] Extract name_EtAlKanji.txt (et al. with multiple authors) → multilingual-cjk.json
- [x] Extract name_ArabicShortForms.txt (Arabic diacritics + transliterations) → multilingual-arabic.json
- [x] Write native Rust test module (multilingual.rs) with 4 test cases
  - test_cjk_name_rendering_asian_glyphs
  - test_cjk_et_al_rendering
  - test_arabic_short_forms_with_diacritics
  - test_arabic_transliterated_forms

## Summary of Changes

Extracted three CSL processor test files into Citum-shaped fixture JSON:
- **multilingual-cjk.json**: Added two CSL test cases (CSL-ASIAN-GLYPHS, CSL-ET-AL-KANJI) alongside existing CJK fixtures
- **multilingual-arabic.json**: Created new file with two Arabic short-form test cases (ARABIC-ASWANI-DIACRITICS, ARABIC-ASWANI-TRANSLITERATED)

Added crates/citum-engine/tests/multilingual.rs with 4 native Rust tests validating CJK name rendering and Arabic diacritical/transliteration handling against the APA-7th style.

Refs: csl26-mlt1, csl26-mlt2
