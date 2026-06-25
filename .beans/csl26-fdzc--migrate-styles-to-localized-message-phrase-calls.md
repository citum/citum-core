---
# csl26-fdzc
title: Migrate styles to localized message phrase calls
status: in-progress
type: task
priority: high
tags:
    - styles
    - localization
    - mf2
created_at: 2026-06-25T00:22:34Z
updated_at: 2026-06-25T10:16:23Z
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
- Keep lexical and inflectional labels as `term.*` or `role.*` locale messages
  where they are labels rather than phrase realization.

## Acceptance Criteria

- Each batch preserves existing fidelity gates for the affected styles.
- Deprecated template `term:` use decreases monotonically across `styles/`.
- Missing message IDs and missing message args are caught by lint before merge.
- Embedded styles are completed before broad non-embedded migration begins.
- Any new phrase IDs are documented in `docs/specs/LOCALE_MESSAGES.md`.


## Embedded Proof Batch (PR #965)

- Started the embedded migration with output-equivalent `message:` calls in embedded-core styles.
- Converted representative `pattern.accessed-date` call sites in AMA, Chicago, Elsevier, MLA, and Springer Vancouver styles.
- Converted representative `pattern.in-container` call sites in Chicago, IEEE, and Springer author-date styles.
- Added grouped message arguments so `pattern.in-container` can receive an already-rendered container cluster such as editor plus parent-monograph title.
- Deferred APA's container-author site, colon-bearing `in:` sites, `URL ` labels, and role-plus-name phrases until later batches define phrase IDs that preserve those outputs without encoding English glue inside arguments.
