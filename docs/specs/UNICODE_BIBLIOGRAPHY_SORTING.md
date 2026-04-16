# Unicode Bibliography Sorting Specification

**Status:** Active
**Date:** 2026-04-16
**Related:** unicode-aware bibliography ordering regression

## Purpose
Define locale-aware bibliography sorting for Citum so accented and other non-ASCII names sort according to Unicode collation rules instead of raw bytewise string order.

## Scope
In scope: bibliography author/title sort comparisons in the engine, shared sort-key normalization, regression fixtures, examples, and tests. Out of scope: new public schema options, transliteration-aware sorting policy changes, or broader multilingual rendering redesign.

## Design
The engine uses ICU4X collation for text sort comparisons in both top-level bibliography sorting and grouped bibliography sorting. Existing article stripping and author/editor/title fallback behavior remains unchanged. Citum locale IDs are parsed as ICU locales when possible; if a locale override ID is not directly parseable, the engine falls back by dropping rightmost subtags until it finds a valid locale, then falls back to `en-US`.

## Implementation Notes
Use a crate-local sorting helper so both sorting pipelines build and compare text keys consistently. Normalize existing string keys with lowercase conversion before collation to preserve current case-insensitive bibliography behavior while gaining Unicode-aware ordering.

## Acceptance Criteria
- [ ] Author/date bibliography sorting places accented surnames near their ASCII peers instead of at the end of the list.
- [ ] Grouped bibliography sorting and top-level bibliography sorting use the same locale-aware text comparison path.
- [ ] Regression fixtures and examples include accented surnames covered by automated tests.

## Changelog
- 2026-04-16: Initial version.
