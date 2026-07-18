---
# csl26-k2kp
title: 'Punctuation realization layer: semantic delimiters'
status: todo
type: feature
priority: normal
tags:
    - multilingual
    - punctuation
    - architecture
created_at: 2026-07-18T20:31:58Z
updated_at: 2026-07-18T20:32:32Z
parent: csl26-0ugp
---

Introduce a realization layer that maps semantic punctuation roles (list separator, field separator, subfield delimiter, wrap open/close, sort separator) to glyphs per (role, effective script, locale), so styles author intent once and each item's script selects half-width or full-width forms. Subsumes the one-directional remap_to_latin_punctuation pass and its three insertion points; csl26-kneq (script-aware wrap rendering) is the first implementation increment. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §5.

Relation note: blocked by csl26-kneq (script-aware wrap rendering, increment 1). That bean file currently lives on the codex/calendar-date-annotations branch; add the formal blocked-by link once both beans are on main.
