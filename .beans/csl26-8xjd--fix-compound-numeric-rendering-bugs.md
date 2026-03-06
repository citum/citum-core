---
# csl26-8xjd
title: Fix compound numeric rendering bugs
status: completed
type: bug
priority: high
created_at: 2026-03-06T18:56:42Z
updated_at: 2026-03-06T19:08:48Z
---

Three rendering bugs in compound numeric styles:

1. Integral citations show bare [1] instead of [1a]/[1b] — render_author_number_for_numeric_integral_with_format ignores citation_sub_label_for_ref
2. Bibliography shows duplicate [1] entries instead of merged [1a]/[1b] — CLI's print_human uses process_references() bypassing merge_compound_entries
3. Multi-item citations don't collapse [1a,1b] → [1a,b] (separate follow-up)

Existing tests pass because test_compound_numeric_bibliography_rendering calls render_bibliography() directly, not the CLI code path.

## Tasks
- [x] Add failing test for Bug 2 (integral sub-labels)
- [x] Fix Bug 2 in rendering.rs
- [x] Fix Bug 3 in main.rs (CLI ungrouped bib path)
- [x] Confirm tests pass

## Summary of Changes

- **rendering.rs**: `render_author_number_for_numeric_integral_with_format` now looks up sub-label via `citation_sub_label_for_ref` and includes it in the bracket.
- **mod.rs**: `render_bibliography_with_format` now renders sub-entry content without the citation-number label component, uses `render_entry_body_with_format` for bibliography separators without outer wrappers, and keeps label components separate in the merged `ProcEntry`.
- **main.rs**: CLI ungrouped bibliography path uses `render_selected_bibliography_with_format` for human output (handles compound merge while respecting keys); oracle/show_keys path unchanged.
- **tests.rs**: two new regression tests added

PR: https://github.com/citum/citum-core/pull/297
