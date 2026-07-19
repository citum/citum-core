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
updated_at: 2026-07-19T11:05:15Z
parent: csl26-0ugp
---

Introduce a realization layer that maps semantic punctuation roles (list separator, field separator, subfield delimiter, wrap open/close, sort separator) to glyphs per (role, effective script, locale), so styles author intent once and each item's script selects half-width or full-width forms. Subsumes the one-directional remap_to_latin_punctuation pass and its three insertion points; csl26-kneq (script-aware wrap rendering) is the first implementation increment. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §5.

## Increment 1 (absorbs the retired draft bean csl26-kneq)

- [ ] Thread item script/language into wrap rendering
- [ ] Render full-width CJK delimiters (（ ）, 【 】) for CJK-script items via wrap: parentheses/brackets
- [ ] Byte-for-byte parity preserved for Latin-script styles (oracle gate)
- [ ] Tests covering CJK + Latin items in one bilingual style

Unblocks csl26-0kqf (calendar-note wraps), which depends on this increment alone. Add the formal blocking link to csl26-0kqf once both bean files are on main; csl26-kneq itself is retired on the codex/calendar-date-annotations branch.

Spec: docs/specs/PUNCTUATION_REALIZATION.md (Draft). Increment 1 is csl26-kneq (script-aware WrapPunctuation rendering); increments 2-3 add the { mark: ... } token form, realization tables with per-script style overrides, realization-default, and the GB/T migration that demotes remap_to_latin_punctuation to a compatibility shim.
