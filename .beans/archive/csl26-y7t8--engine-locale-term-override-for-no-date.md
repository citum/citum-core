---
# csl26-y7t8
title: 'engine: locale term override for no-date'
status: completed
type: bug
priority: high
created_at: 2026-03-08T13:39:54Z
updated_at: 2026-03-08T14:09:03Z
---

## Problem

harvard-cite-them-right renders (Forthcoming., n.d.) but oracle expects (Forthcoming, no date).

1. **n.d. vs "no date"**: en-US locale term hardcoded as n.d.; no style-level term override exists.
2. **Trailing period on short-form names**: form:short still appends period (Forthcoming.).

## Location (via jCodeMunch)
- parse_general_term — crates/citum-schema/src/locale/mod.rs:776
- test_citation_locator_label_renders_term_with_loaded_locale — crates/citum-engine/src/processor/tests.rs:317

## Acceptance
- (Forthcoming, no date) passes oracle for harvard-cite-them-right
- No regression on styles using n.d.

## Summary of Changes

- Normalized locale `no-date` parsing into form-aware general-term lookup while
  preserving the legacy short-form fallback.
- Fixed parsed Harvard grouped author-date citations so they no longer emit
  stray punctuation around the author chunk.
- Added regression coverage for Harvard `no date` rendering and for styles that
  still rely on `n.d.`.
