---
# csl26-5q59
title: Digit-system localization in number-formats
status: completed
type: feature
priority: normal
tags:
    - multilingual
    - locale
created_at: 2026-07-18T20:32:33Z
updated_at: 2026-07-22T13:32:58Z
parent: csl26-0ugp
---

Locale number-formats covers separators but not digit systems: Arabic-Indic (٠١٢), Persian, and Devanagari digits are unrepresentable in rendered output. Add an optional digit-system field to locale number-formats with a conversion pass at number rendering, defaulting to Western digits so existing locales are unaffected. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(f).

## Summary of Changes

- Added locale-configured Western, Arabic-Indic, extended Arabic-Indic, and Devanagari digit rendering for number template components.
- Enabled Arabic-Indic digits for ar-AR and documented the public locale schema contract.
- Added schema, renderer, and regression coverage; regenerated locale schema.
