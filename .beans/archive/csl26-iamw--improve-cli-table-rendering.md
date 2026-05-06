---
# csl26-iamw
title: Improve CLI table rendering
status: completed
type: task
priority: normal
created_at: 2026-05-06T10:24:42Z
updated_at: 2026-05-06T10:27:09Z
---

Replace ad-hoc format! padding and UTF8_FULL comfy-table preset with a clean shared table builder. Unify format_style_catalog_text, run_styles_list, and run_registry_list behind one consistent renderer using UTF8_BORDERS_ONLY preset with cyan headers and ContentArrangement::Dynamic.

## Summary of Changes\n\nCreated shared build_table() helper in table.rs using UTF8_BORDERS_ONLY preset with cyan headers and ContentArrangement::Dynamic. Unified format_style_catalog_text, run_styles_list, and run_registry_list behind this single renderer. Removed UTF8_FULL and fixed-width format! padding.
