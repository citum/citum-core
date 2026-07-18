---
# csl26-5q59
title: Digit-system localization in number-formats
status: todo
type: feature
tags:
    - multilingual
    - locale
created_at: 2026-07-18T20:32:33Z
updated_at: 2026-07-18T20:32:33Z
parent: csl26-0ugp
---

Locale number-formats covers separators but not digit systems: Arabic-Indic (٠١٢), Persian, and Devanagari digits are unrepresentable in rendered output. Add an optional digit-system field to locale number-formats with a conversion pass at number rendering, defaulting to Western digits so existing locales are unaffected. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(f).
