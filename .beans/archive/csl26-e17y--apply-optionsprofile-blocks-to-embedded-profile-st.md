---
# csl26-e17y
title: Apply options.profile blocks to embedded profile styles
status: completed
type: task
priority: normal
created_at: 2026-04-21T19:34:01Z
updated_at: 2026-04-21T19:45:10Z
---

Add options.profile: declarations to the 9 bare extends-only profile stubs in styles/embedded/ so the new ProfileConfig machinery is actually exercised. Author-date profiles get name-list-profile + date-position axes; numeric profiles get citation-label-wrap + bibliography-label-mode axes.

## Summary of Changes

Added `options.profile:` blocks to 9 embedded profile stub files and fixed a double-resolution bug in `resolve_profile_over`.

**Author-date profiles** (name-list-profile + date-position: after-author):
- elsevier-harvard.yaml → harvard
- springer-basic-author-date.yaml → springer
- taylor-and-francis-chicago-author-date.yaml → chicago
- taylor-and-francis-council-of-science-editors-author-date.yaml → vancouver

**Numeric profiles** (citation-label-wrap: brackets + bibliography-label-mode: numeric):
- elsevier-vancouver.yaml, elsevier-with-titles.yaml, springer-basic-brackets.yaml, springer-vancouver-brackets.yaml, taylor-and-francis-national-library-of-medicine.yaml

**Bug fix** (lib.rs): `resolve_profile_over` now sets `effective.extends = None` instead of copying the wrapper's `extends`. This prevents a second `into_resolved()` call (triggered by `Processor::new`) from re-running profile validation on an already-mutated style where `apply_to_style` has written `options.contributors`.
