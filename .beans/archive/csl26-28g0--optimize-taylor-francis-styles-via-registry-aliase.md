---
# csl26-28g0
title: Optimize Taylor & Francis styles via registry aliases
status: completed
type: feature
priority: normal
created_at: 2026-04-19T12:26:18Z
updated_at: 2026-04-19T12:26:18Z
---

Add three T&F style-letter aliases to registry/default.yaml to avoid duplicating style logic.

CSV at ~/Documents/tf-styles-map.csv enumerates 11 T&F style letters. CSL dependent corpus shows only 3 (B, T, S/C) parent any real journals — all already modeled as YAML + registry builtins. The other 8 have near-zero Zotero uptake and are either straight aliases to existing builtins or manuscript-submission variants without rendering deltas.

## Tasks
- [x] Add alias `taylor-and-francis-style-p` to `apa-7th` entry
- [x] Add alias `taylor-and-francis-style-f` to `chicago-author-date-18th` entry
- [x] Add alias `taylor-and-francis-style-e` to `modern-language-association` entry
- [x] Run registry tests (`cargo nextest run -p citum-schema-style`)
- [x] Run full pre-commit gate (fmt --check, clippy, nextest)

## Scope guard
Styles R (royal-society-of-chemistry — not a builtin), X (taylor-and-francis-harvard-x — needs migration), G (chicago-note-bibliography variant), V (harvard general), Q (math) deferred to follow-up beans.

## Follow-ups (defer)
- Bulk 348-journal alias registry → citum-hub scope
- Style X YAML via fresh migration — separate bean

Plan: ~/.claude/plans/implement-grammatical-gender-support-wild-micali.md

## Summary of Changes

Three T&F style-letter aliases added to registry/default.yaml:
- `taylor-and-francis-style-p` → `apa-7th` (straight APA 7th match per corpus analysis)
- `taylor-and-francis-style-f` → `chicago-author-date-18th` (straight Chicago author-date match)
- `taylor-and-francis-style-e` → `modern-language-association` (straight MLA match)

All tests passing. Registry validation confirms alias syntax. No schema changes required — YAML-only.
