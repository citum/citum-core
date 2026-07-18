---
# csl26-itri
title: Locale completeness lint + zh-CN/ar-AR typography fill
status: todo
type: task
tags:
    - multilingual
    - locale
created_at: 2026-07-18T20:32:32Z
updated_at: 2026-07-18T20:32:32Z
parent: csl26-0ugp
---

zh-CN and ar-AR embedded locales lack grammar-options and date-formats, so Chinese and Arabic output silently inherits English typography defaults (curly quotes, half-width delimiters, English date assembly). Fill both locales (full-width quote and delimiter conventions and yyyy年M月d日-style patterns for zh-CN; Arabic conventions for ar-AR) and add a CI lint that flags embedded locales missing sections, so English fallback becomes a visible choice. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(c).
