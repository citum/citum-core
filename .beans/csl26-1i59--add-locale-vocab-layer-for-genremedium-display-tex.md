---
# csl26-1i59
title: Add locale vocab layer for genre/medium display text
status: todo
type: feature
priority: normal
created_at: 2026-03-29T22:38:05Z
updated_at: 2026-03-29T22:38:08Z
blocked_by:
    - csl26-qqfa
---

Implement locale vocab lookup for canonical genre/medium keys. Create locale/en/vocab.yaml (and other locale files) mapping kebab-case keys to display strings (e.g. phd-thesis → 'PhD thesis'). Wire display-text resolution into the render layer so stored canonical values are localized at render time. Prerequisite: csl26-qqfa (normalization) must be complete first. See docs/reference/GENRE_AND_MEDIUM_VALUES.md §Localization for the planned shape.
