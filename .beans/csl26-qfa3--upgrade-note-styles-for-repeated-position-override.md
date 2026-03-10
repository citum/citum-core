---
# csl26-qfa3
title: Upgrade note styles for repeated-position overrides and refresh compat snapshot
status: todo
type: task
priority: normal
tags:
    - styles
    - compatibility
created_at: 2026-03-10T18:31:26Z
updated_at: 2026-03-10T19:39:41Z
---

Follow-up after repeated-note semantics engine/migration work:

- audit migrated note styles for use of citation.subsequent / citation.ibid (Chicago notes, OSCOLA, MHRA, Bluebook-like styles)
- run style upgrades where needed to express intended immediate-repeat behavior
- run oracle batch impact and core report checks
- refresh docs/compat.html if portfolio metrics or examples change
- decide whether baseline updates are needed in dedicated baseline PR
