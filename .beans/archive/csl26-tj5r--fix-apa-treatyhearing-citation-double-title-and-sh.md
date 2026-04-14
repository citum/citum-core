---
# csl26-tj5r
title: 'fix: APA treaty/hearing citation double-title and short-form'
status: completed
type: bug
priority: high
created_at: 2026-04-14T11:33:49Z
updated_at: 2026-04-14T11:35:55Z
---

Two bugs causing broken APA parenthetical citations for treaty and hearing:
1. requires_full_group_item_rendering missing treaty/hearing → double title
2. Title::Structured ignores form: short → full compound title instead of main only

## Summary of Changes

- core.rs: added treaty and hearing to requires_full_group_item_rendering
- title.rs: render_structured_title now takes a short flag; Structured titles with form: short return main only
