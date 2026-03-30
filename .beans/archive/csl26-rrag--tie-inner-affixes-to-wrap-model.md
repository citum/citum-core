---
# csl26-rrag
title: Tie inner affixes to wrap model
status: completed
type: feature
priority: normal
created_at: 2026-03-25T20:45:08Z
updated_at: 2026-03-30T19:36:50Z
---

Tie `inner-prefix` and `inner-suffix` to an explicit wrap-owned data model.

- Redesign the wrap representation so inner affixes are not meaningful without
  `wrap`.
- Preserve authoring clarity and GUI enforceability.
- Document any serde/schema migration implications before implementation.
