---
# csl26-mlt2
title: Implement multilingual processor logic
status: completed
type: feature
priority: high
created_at: 2026-02-12T00:00:00Z
updated_at: 2026-02-27T16:08:19Z
---

Implement resolve_multilingual_string and resolve_multilingual_name in citum_engine with BCP 47 matching (exact → prefix → fallback).

**Punctuation Portability (Frank Bennett insight):**
- Apply component prefix/suffix AFTER multilingual resolution, not before
- Prevents double-punctuation in combined modes (e.g., "Tanaka Tarō [田中太郎]. . Title")
- Test cases: presets with suffix across all multilingual modes (original/transliterated/combined)

Add integration tests with multilingual reference data.

Refs:docs/architecture/MULTILINGUAL.md Section 3, csl26-mlt1, Frank Bennett CSL-M guidance

## Summary of Changes

Verified as already implemented during staleness audit 2026-02-27. The multilingual processor logic is complete with resolve_multilingual_string and resolve_multilingual_name functions in citum_engine/src/values/. Implementation includes BCP 47 language matching (exact → prefix → fallback), punctuation portability handling after resolution, and comprehensive test coverage in citum_engine/tests/i18n.rs with multilingual reference data.
