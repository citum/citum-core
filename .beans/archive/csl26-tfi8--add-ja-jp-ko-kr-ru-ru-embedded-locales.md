---
# csl26-tfi8
title: Add ja-JP, ko-KR, ru-RU embedded locales
status: completed
type: task
priority: normal
tags:
    - multilingual
    - locale
created_at: 2026-07-18T20:32:33Z
updated_at: 2026-07-20T20:08:10Z
parent: csl26-0ugp
---

No Japanese, Korean, or Russian embedded locales exist, despite Katakana name handling being the motivating case of MULTILINGUAL_NAMES.md and Cyrillic sorting a motivating case of MULTILINGUAL_SORTING.md. Author schema-v2/MF2 locales for ja-JP first, then ko-KR and ru-RU, using en-US and de-DE as structural references. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(c).

## Summary of Changes

Authored ja-JP.yaml, ko-KR.yaml, ru-RU.yaml (crates/citum-schema-style/embedded/locales/) at de-DE structural depth: dates, roles, terms (general + reference-type + locator), messages (MF2), date-formats, grammar-options, legacy-term-aliases. Each carries an EXPERT REVIEW NEEDED comment flagging the vocabulary as unreviewed by a native speaker.

Registered all three in crates/citum-schema-style/src/embedded/locales.rs (get_locale_bytes + EMBEDDED_LOCALE_IDS). Extended embedded_locale_ids_include_all_bundled_locale_files and added bundled_ja_jp_ko_kr_ru_ru_locales_are_embedded_and_parseable round-trip tests in crates/citum-schema-style/src/locale/mod.rs.

Spot-checked ru-RU by rendering a reference through styles/gost-r-7-0-5-2008-author-date.yaml — author/date/general terms resolve correctly; the style's own literal "Vol."/"P." prefixes are a pre-existing style-authoring gap, unrelated to this locale addition (out of scope here).

Updated docs/guides/AUTHORING_LOCALES.md coverage inventory.
