---
# csl26-ktq6
title: Wire document_options.integral_name_memory through the processor
status: completed
type: feature
priority: normal
created_at: 2026-06-04T13:59:01Z
updated_at: 2026-06-04T20:22:54Z
---

The session and Tier 1 `format_document` APIs accept `document_options.integral_name_memory` but do not apply it. The processor renders without document-level narrative (integral) name memory, so first-full-then-short behaviour across a document is not honoured.

Currently `citum-engine/src/api/session.rs` (and the Tier 1 path) emit an `integral_name_memory_not_applied` warning when the field is present.

## Scope

- Wire `DocumentOptions.integral_name_memory` through `Processor` so integral citations honour document-level first/subsequent name-form rules.
- Apply in both Tier 1 `format_document` and the Tier 2 `DocumentSession` render path.
- Remove the `integral_name_memory_not_applied` warning once applied.
- Tests: author-date document with repeated integral citations to the same author; assert first occurrence renders full name form and subsequent occurrences render short form per the option.

## Origin

Split out from csl26-3yk1 (session API). The warning string in `session.rs` previously cited csl26-wq0y in error; wq0y is integration-test scope only.

## Summary of Changes

- Added  helper method () on  in . Derives  from citation order and  (for body/note placement), then delegates to the existing  path.
- Changed  visibility from  to  so API modules can call it.
- Wired both changes into the Tier 1  path () and the Tier 2  path (): builds override processor first, then annotates citations after the missing-ref retain loop.
- Removed the  warning from both paths.
- Added 4 new unit tests covering first-full/subsequent-short and the disabled override guard, in both API modules.
