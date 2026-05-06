---
# csl26-j242
title: Add citum style browse TUI subcommand
status: draft
type: feature
priority: normal
created_at: 2026-05-06T10:24:45Z
updated_at: 2026-05-06T10:24:47Z
blocked_by:
    - csl26-iamw
---

Add an interactive citum style browse subcommand backed by ratatui + crossterm. Scrollable, filterable style list with a detail pane. Complements the plain-text citum style list. Depends on the table rendering improvement (clean list) landing first. Adds ~3 new deps and ~300 LOC.
