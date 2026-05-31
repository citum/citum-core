---
# csl26-oyl4
title: Add gendered locale snapshot coverage
status: completed
type: task
priority: low
tags:
    - locale
    - testing
    - schema
    - multilingual
created_at: 2026-04-29T15:43:17Z
updated_at: 2026-05-31T21:54:47Z
parent: csl26-li63
---

Follow-up split from csl26-y3kj after the MaybeGendered<T> locale schema work landed. Add focused snapshot coverage for gendered locale rendering so the completed implementation has durable regression fixtures.

## Tasks

- [x] Add a French snapshot test for a gendered editor role label.
- [x] Add an Arabic snapshot test for a gendered ordinal or locator term.
- [x] Confirm existing plain-string locale fixtures still render unchanged.

## Context

The model and runtime work landed in csl26-y3kj. This bean tracks only the remaining snapshot coverage gap.

## Summary of Changes

- **fr-FR.yaml**: Expanded `roles.editor.long` from plain strings to gendered singular/plural values with masculine/feminine/common variants (éditeur/éditrice, éditeurs/éditrices).
- **ar-AR.yaml**: Added `locators.page` with Arabic text (صفحة/صفحات) and a `gender: feminine` lexical gender marker.
- **tests/gendered_locale.rs**: New focused test file with three tests:
  - `french_gendered_editor_role_label` — verifies MaybeGendered dispatch on the fr-FR editor role (gender × number combinations).
  - `arabic_gendered_page_locator_and_role` — verifies ar-AR page locator text, feminine lexical gender field, and MF2-driven masculine/feminine/neutral editor dispatch.
  - `plain_locale_fixtures_unchanged` — verifies fr-FR common fallback and en-US page terms are unaffected.
