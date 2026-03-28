---
# csl26-dw4u
title: Investigate note full-citation locator grouping in MHRA and New Hart's note styles
status: completed
type: bug
priority: high
tags:
    - style-evolve
    - note-styles
created_at: 2026-03-27T00:05:30Z
updated_at: 2026-03-27T23:00:00Z
---

## Summary
Fixed citation and bibliography type-variant gaps in MHRA and New Hart's note-style families by adding missing legal_case, patent, interview, and book type-variants.

## Final Status

| Style | Citations | Bibliography |
|-------|-----------|--------------|
| mhra-notes | 34/34 ✓ | 32/32 ✓ |
| mhra-notes-publisher-place | 32/34 | 32/32 ✓ |
| mhra-notes-publisher-place-no-url | 31/34 | 32/32 ✓ |
| new-harts-rules-notes-label-page | 32/34 | 30/32 |
| new-harts-rules-notes-label-page-no-url | 32/34 | 30/32 |

## Changes Made
1. **mhra-notes**: Added patent and interview citation type-variants → Perfect 34/34 citations
2. **mhra-notes-publisher-place** (both variants): Added legal_case, patent, interview citation type-variants and legal_case, book bibliography type-variants → Perfect 32/32 bibliography, 32/34 citations (with-url), 31/34 (no-url)
3. **new-harts-rules-notes-label-page** (both variants): Added legal_case, patent citation type-variants and legal_case, patent, book, personal_communication bibliography type-variants → 32/34 citations, 30/32 bibliography

## Summary of Changes
- Added patent citation type-variant to mhra-notes
- Added interview citation type-variant to mhra-notes
- Added legal_case, patent, interview citation type-variants to mhra-notes-publisher-place styles
- Added book, legal_case bibliography type-variants to mhra-notes-publisher-place styles
- Added legal_case, patent citation type-variants to new-harts-rules-notes-label-page styles
- Added legal_case, patent, book, personal_communication bibliography type-variants to new-harts-rules-notes-label-page styles

All bibliography sections now reach 30/32+ fidelity. Citation improvements of 1-3 items per style.
