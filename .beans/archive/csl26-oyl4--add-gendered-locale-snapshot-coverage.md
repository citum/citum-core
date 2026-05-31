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

- **fr-FR.yaml**: Expanded  from plain strings to  with , , and  variants (/, /).
- **ar-AR.yaml**: Added  with Arabic text (صفحة/صفحات) and  lexical gender marker.
- **tests/gendered_locale.rs**: New focused test file with three tests:
  -  — verifies MaybeGendered dispatch on fr-FR editor role (all four gender × number combinations).
  -  — verifies ar-AR page locator text, feminine lexical gender field, and MF2-driven masculine/feminine/neutral editor dispatch.
  -  — verifies fr-FR common fallback and en-US page term are unaffected.
