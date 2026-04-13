---
# csl26-zr4q
title: Personal communication citation rendering
status: completed
type: bug
priority: normal
created_at: 2026-04-13T12:15:59Z
updated_at: 2026-04-13T12:29:05Z
---

Fix personal communication rendering in APA integral and non-integral citations.

## Todo

- [x] Add `PersonalCommunication` to `GeneralTerm` enum (locale/types.rs)
- [x] Register in `parse_general_term` + `general_term_to_message_id` (locale/mod.rs)
- [x] Remove contributor engine special-case (contributor/mod.rs lines 157-168)
- [x] Remove date engine special-case (date.rs lines 694-701)
- [x] Fix apa-7th.yaml non-integral personal-communication template
- [x] Fix apa-7th.yaml integral personal-communication template
- [x] Fix Oglethorpe record in chicago-note-converted.yaml
- [x] Create spec docs/specs/PERSONAL_COMMUNICATION_CITATION.md
- [x] Verify with cargo nextest run

## Summary of Changes

- Added `GeneralTerm::PersonalCommunication` to schema with en-US term in `Terms::en_us()` and `parse_general_term`
- Removed engine special-cases in `contributor/mod.rs` and `date.rs` that hardcoded personal-communication rendering
- Restructured `apa-7th.yaml` integral template: `personal communication` label now inside the parenthetical group (not as author suffix)
- Non-integral template: replaced hardcoded suffix with `- term: personal-communication` component
- Fixed Oglethorpe input record: proper `contributors` with author + recipient roles instead of embedded title string
- Created spec `docs/specs/PERSONAL_COMMUNICATION_CITATION.md`

Output:
- Non-integral: `(J. Oglethorpe, personal communication, 1733)` ✓
- Integral: `J. Oglethorpe (personal communication, 1733)` ✓
