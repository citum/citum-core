---
# csl26-ra1b
title: Multilingual architecture design audit
status: completed
type: task
priority: high
tags:
    - multilingual
    - architecture
created_at: 2026-07-18T20:11:33Z
updated_at: 2026-07-18T20:15:26Z
---

Broad design audit of multilingual architecture: locale system, script-aware punctuation (options.multilingual.scripts), data model vs locale vs schema vs renderer separation. Deliverable: architectural assessment + prioritized recommendations + follow-up beans. Requested by Bruce 2026-07-18.

## Summary of Changes

Analysis-only audit, no code changes. Reviewed MULTILINGUAL*.md, PUNCTUATION_NORMALIZATION.md, CALENDAR_DATE_ANNOTATIONS.md specs; embedded locales (en/de/fr/es/eu/tr/zh/ar); engine code: remap_to_latin_punctuation + wants_latin_punctuation (render/component.rs), is_latin_script_language (values/mod.rs), GrammarOptions (locale/types.rs), text_case.rs, MultilingualConfig (options/multilingual.rs).

Key findings delivered to Bruce:
- Data model / sorting / MF2 locale layers are sound; weakness is the punctuation *realization* layer (literal chars + late string rewrites at 3 insertion points).
- Boolean latin/not-latin classifier should become ISO 15924 effective-script resolution.
- Locale data asymmetry: zh-CN/ar-AR lack grammar-options + date patterns -> silent English typography defaults; no ja/ko/ru locales; no completeness lint.
- No bidi/RTL model; no digit-system localization; case transforms not locale-tailored (Turkish i).
- Recommended general abstraction: semantic delimiter roles + per-script/locale realization tables (subsumes csl26-kneq and the remap).
- Proposed ~9 follow-up beans (not yet created; awaiting Bruce's selection).
