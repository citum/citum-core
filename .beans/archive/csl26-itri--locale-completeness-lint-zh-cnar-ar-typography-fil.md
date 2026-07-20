---
# csl26-itri
title: Locale completeness lint + zh-CN/ar-AR typography fill
status: completed
type: task
priority: normal
tags:
    - multilingual
    - locale
created_at: 2026-07-18T20:32:32Z
updated_at: 2026-07-20T19:51:23Z
parent: csl26-0ugp
---

zh-CN and ar-AR embedded locales lack grammar-options and date-formats, so Chinese and Arabic output silently inherits English typography defaults (curly quotes, half-width delimiters, English date assembly). Fill both locales (full-width quote and delimiter conventions and yyyy年M月d日-style patterns for zh-CN; Arabic conventions for ar-AR) and add a CI lint that flags embedded locales missing sections, so English fallback becomes a visible choice. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(c).

## Summary of Changes

Added `grammar-options` and `date-formats` to `zh-CN.yaml` (GB/T 7714 full-width quotes/delimiters, `yyyy年M月d日`-style dates) and `ar-AR.yaml` (Arabic guillemet quotes, Arabic date patterns). Both files carry an EXPERT REVIEW NEEDED comment flagging that the typography was authored from documented conventions, not by a native speaker.

Added a completeness check to `lint_raw_locale` (crates/citum-schema-style/src/lint.rs) that warns when a v2 locale is missing `grammar-options` or `date-formats`. Added a CI-enforced test (`embedded_v2_locales_pass_completeness_lint` in crates/citum-schema-style/src/locale/mod.rs) that runs the lint against every embedded locale.

Updated docs/guides/AUTHORING_LOCALES.md coverage inventory.
