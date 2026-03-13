---
# csl26-6ixg
title: Investigate integral citation rendering gaps
status: completed
type: bug
priority: high
created_at: 2026-03-13T11:51:00Z
updated_at: 2026-03-13T11:51:00Z
---

## Context

While simplifying `crates/citum-engine/src/processor/rendering.rs` in `csl26-xtt0`, I investigated two nearby behavior gaps:

- explicit multi-item integral citations with an explicit integral template only rendered the first grouped item;
- label-integral fallback behavior appeared to omit the rendered citation label in the narrative form.

## Why This Was Deferred

A direct fix stopped being a bounded simplify change. The attempted implementation widened into grouped-citation assembly and affix/wrap handling, which regressed existing citation behavior across author-date grouping, disambiguation scenarios, and note-style ibid formatting.

Those regressions were broad enough that the safe outcome for the simplify PR was to restore pre-existing grouped rendering semantics and defer the behavior work to a dedicated follow-up.

## Follow-up Checklist

- [x] Reproduce the current explicit-integral and label-integral gaps with focused tests before changing rendering logic.
- [x] Define expected grouped integral behavior for multi-item cites with explicit integral templates, including locators and suffixes.
- [x] Decide whether label-integral fallback should render the citation label, and document the expected narrative form.
- [x] Implement the behavior change without regressing grouped author-date, disambiguation, or note-style citation output.
- [x] Verify with `cargo test -p citum-engine --test citations`, targeted `processor::rendering` tests, and `~/.claude/scripts/verify.sh`.

## Changes Made

### Bug 1 Fix: Grouped Integral Citations with Explicit Template
**Problem**: In `render_grouped_citation_with_format`, when there was an explicit integral template, the code only rendered the first item in the group.

**Solution**: Updated the grouped integral path (lines ~835-870) to:
1. Iterate through ALL items in the group, not just `first_item`
2. Render each item with the integral template from `self.style.citation.integral.template`
3. Join all rendered items with the integral delimiter

### Bug 2 Fix: Label-Integral Rendering
**Problem**: In `render_ungrouped_citation_with_format`, the `use_label_author` branch (lines ~735-746) called `render_author_for_label_integral_with_format`, which returned only the bare author name, omitting the citation label entirely.

**Solution**: Removed the special `use_label_author` branch entirely, allowing the standard template path to handle label-integral citations correctly, which now renders both the author and the label through the full template.

### Dead Code Removal
Removed following functions that are now unused:
- `should_render_author_for_label_integral()` (was used only by the removed branch)
- `render_author_for_label_integral_with_format()` (was called only by the removed branch)

Also replaced inline duplicate `has_explicit_integral_template` check (lines ~831-836 in old code) with a call to the existing `self.has_explicit_integral_template()` method.

## Tests Updated
- Added `test_grouped_integral_citation_renders_all_items`: Verifies all items in an integral group are rendered
- Added `test_label_integral_citation_includes_label`: Verifies label is included in output
- Updated `test_label_integral_citation_uses_author_text`: Now expects both author and label in output (previously expected author-only)

All 401 engine tests pass, including 24 citation-specific integration tests.
