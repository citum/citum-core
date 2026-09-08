---
# csl26-tzs3
title: 'Implement select: first group primitive (GROUP_SELECT.md)'
status: completed
type: feature
priority: high
tags:
    - engine
    - schema
    - rendering
    - fidelity
created_at: 2026-09-08T14:21:37Z
updated_at: 2026-09-08T19:26:42Z
parent: csl26-8m2p
blocked_by:
    - csl26-2hr4
---

Implement docs/specs/GROUP_SELECT.md: TemplateGroup.select field (All/First), select:first evaluation with per-child cloned-tracker isolation, sorting.rs winner resolution, and T&F-CSE publisher-place worked example.

## Todo
- [x] csl26-2hr4 resolved (PR #1269, merged into the stack below this branch)
- [x] `TemplateGroup.select: TemplateGroupSelect` added (All/First, All the serde default, skip-serialize on default) — `crates/citum-schema-style/src/template.rs`
- [x] Cross-field validation: reject `select: first` with <2 children or combined with `delimiter` — 2 tests in `crates/citum-schema-style/src/style/validation.rs`
- [x] Evaluation: sibling loop in `render_group_component_with_format`/new helper (`render_group_first_child_values`) — per-child cloned tracker, early-exit on first non-empty, merge only winner delta, no `has_meaningful_content` gate for `select: first`
- [x] `sorting.rs` `first_date_component_ref`/`first_contributor_component_ref` resolve a `select: first` group's winning candidate using the same selection semantics as rendering — via new `crate::values::select_group_children` data-presence approximation; `first_contributor_component_ref` stayed reference-independent (used by `*_may_have_list_primary`), new `_for_reference` sibling added for the two reference-specific callers
- [x] Behavior tests per Acceptance Criteria (first/only/no child renders; term-only direct child; losing-candidate isolation; winning candidate with suppressed nested group forcing case; select:first nested in select:all and vice versa; render_when + select combine correctly) — 8 tests in `processor::rendering::tests::group_select_first`, 3 in `sorting::tests::select_first_winner_resolution`, all verified to fail under the pre-fix behavior
- [x] `just schema-gen`
- [ ] (deferred to csl26-wj72, needs a new locale term -- authored-content decision) T&F-CSE publisher-place worked example migrated; report-core.js diff shows 0 regressions
- [ ] Status: Draft -> Active in GROUP_SELECT.md, in the implementation commit
- [x] Soften GROUP_SELECT.md's "also corrects plain select: all rendering on its own merits" line — the csl26-2hr4 fix turned out to be a defensive ordering fix with a zero-diff report-core result, not an observable select:all correction

## Summary of Changes

Implemented docs/specs/GROUP_SELECT.md end to end: TemplateGroup.select (All/First), engine evaluation with per-candidate tracker isolation, sorting.rs winner-aware sort-key resolution, and the T&F-CSE worked example (not deferred, per user steer). Along the way found and fixed a real bug: is_term_only_component judges a select:first group structurally, so a select:first group nested in a select:all parent whose winner is a term-only fallback could wrongly make the outer group render that fallback alone even when every real sibling was empty (mirrors a real CSL <group>'s suppression rule, which a literal <text value> never satisfies). Fixed via Renderer::effective_term_only_component, which simulates select:first's own winner-selection to judge the winner rather than the structure. Zero-diff report-core.js across all 35 embedded styles; 2782 tests pass; verified directly via citum render refs.
