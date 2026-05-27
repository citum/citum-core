---
# csl26-5cma
title: Fix APA integral citation conjunction (and vs &)
status: completed
type: bug
priority: high
created_at: 2026-05-27T21:49:48Z
updated_at: 2026-05-27T21:55:35Z
---

APA integral/narrative citations should use 'and' but both modes render '&'. Two bugs: (1) mode-specific options never applied to Renderer config in render_citation_content(); (2) resolve_for_mode() replaces options instead of merging. Missing test coverage for this core requirement.

## Summary of Changes

Fixed two bugs causing APA integral citations to render '&' instead of 'and':

1. **Bug 1 (primary):** `render_citation_content()` in `citation.rs` now merges `effective_spec.options` over the mode-agnostic `citation_config` before creating the Renderer, so integral-specific `contributors.and: text` is applied.

2. **Bug 2 (secondary):** `resolve_for_mode()` and `resolve_for_position()` in `sections/citation.rs` now properly merge mode/position options into base citation options instead of replacing them.

3. **New `CitationOptions::merge()`** method in `options/mod.rs` mirrors `Config::merge` for field-by-field override including deep `ContributorConfig::merge`.

4. **Test:** `test_integral_vs_non_integral_conjunction` asserts integral renders 'and' and non-integral renders '&'. All 1419 tests pass.
