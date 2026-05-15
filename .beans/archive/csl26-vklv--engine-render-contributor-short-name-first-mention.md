---
# csl26-vklv
title: 'Engine: render contributor short-name (first-mention parenthetical + subsequent short)'
status: completed
type: task
priority: normal
created_at: 2026-05-15T11:09:23Z
updated_at: 2026-05-15T11:19:23Z
parent: csl26-ycyp
---

Add integral_name_state to NameFormatContext. In format_single_name literal-name path: First+short_name -> 'Full (Short)', Subsequent+short_name -> 'Short'. Add ShortNameDisplay option to IntegralNameConfig (full-then-parenthetical, short-then-bracketed). Files: names.rs, options/integral_names.rs

## Summary of Changes\n\nAdded integral_name_state and short_name_display to NameFormatContext; plumbed from format_names via hints + options.config. Literal-name path in format_single_name now dispatches: First+short_name -> full-then-parenthetical or short-then-bracketed; Subsequent+short_name -> short name only.
