---
# csl26-fdzc
title: Migrate styles to localized message phrase calls
status: todo
type: task
priority: high
tags:
    - styles
    - localization
    - mf2
created_at: 2026-06-25T00:22:34Z
updated_at: 2026-06-25T00:22:34Z
---

## Problem

Styles still use English-centric template phrasing and the compatibility
`term:` component in places where locale-authored `message:` phrase calls
should own natural-language realization.

## Scope

Migrate checked-in `styles/` to the new localized message format in batches.
Start with embedded styles and embedded-locale phrase IDs, then continue
through the rest of `styles/` by style family or shared template pattern.

## Initial Batch

- Convert embedded styles from phrase-like `term:` or literal glue to
  `message:` calls where the locale should control word order.
- Prioritize `pattern.accessed-date`, `pattern.in-container`,
  `pattern.available-at`, and `pattern.retrieved-from`.
- Keep atomic labels as `term.*` locale messages where they are labels rather
  than compositional phrases.

## Acceptance Criteria

- Each batch preserves existing fidelity gates for the affected styles.
- Deprecated template `term:` use decreases monotonically across `styles/`.
- Missing message IDs and missing message args are caught by lint before merge.
- Embedded styles are completed before broad non-embedded migration begins.
- Any new phrase IDs are documented in `docs/specs/LOCALE_MESSAGES.md`.
