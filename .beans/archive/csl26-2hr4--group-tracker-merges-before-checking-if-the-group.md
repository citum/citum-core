---
# csl26-2hr4
title: Group tracker merges before checking if the group rendered
status: completed
type: bug
priority: high
tags:
    - engine
    - rendering
    - fidelity
created_at: 2026-09-06T23:13:42Z
updated_at: 2026-09-08T14:07:54Z
parent: csl26-8m2p
---

render_group_component_with_format (crates/citum-engine/src/processor/rendering/grouped/core.rs, ~line 1368) clones the tracker for a group's children, then unconditionally calls tracker.merge_from(group_tracker) BEFORE checking whether render_group_child_values actually produced output (the values? empty-check comes after the merge). So an empty/suppressed group's tracker mutations -- variable-once marks, substitution bookkeeping, date-fallback-first-issued flags -- still leak into the parent tracker even though the group rendered nothing.

This is a BLOCKING PREREQUISITE for select: first (docs/specs/GROUP_SELECT.md), not an independent someday investigation. Clone-and-discard at the select: first group's own boundary alone is insufficient -- the merge-before-check happens at EVERY level of nesting inside render_group_component_with_format, so if the WINNING candidate is itself a nested group containing a suppressed sub-group, that sub-group's tracker mutations are already baked into the candidate's own clone before select: first ever gets a say. Merging 'only the winner's' tracker still commits the pollution. select: first cannot be implemented with correct tracker isolation until this ordering is fixed (or explicitly proven safe as-is) -- see docs/specs/GROUP_SELECT.md's Implementation Notes and Acceptance Criteria for the dependency.

Resolved (2026-09-08): the tracker holds two semantically different kinds of state, and only one of them was actually leaking. `rendered_vars`/`substituted_bases` (variable-once marks, substitution bookkeeping) are mutated by `mark_rendered`, which only fires on the success path *after* a component has already produced non-empty content -- so a suppressed or empty group cannot mutate them at all under current code, short of an unconstructible edge case (a component whose raw value passes the empty check but whose post-format detailed text trims to empty). `issued_occurrences` (the date-fallback "is this the first occurrence" counter) is different: it is mutated by `next_issued_is_first()` unconditionally, before any suppress/empty check, and an existing test -- `blank_nested_first_issued_still_advances_the_later_lane` (crates/citum-engine/src/processor/rendering/tests.rs) -- proves this is *intentional*: a blank/suppressed `date: issued` still occupies its structural position in the template's fallback-lane ordering (see `TemplateDate::suppress_note`/`suppress_disamb_suffix` docstrings, which document the same "redundant occurrence" concept). So the merge is split: `issued_occurrences` still advances unconditionally (via a new `advance_issued_occurrences_from`), while `rendered_vars`/`substituted_bases` only merge after `render_group_child_values` confirms the group actually rendered. `node scripts/report-core.js --all-features` before/after this change on the full embedded style corpus (35 styles) shows a zero diff on citations/bibliography/exactParity, confirming no observable behavior change today -- this is a defensive ordering fix for the `select: first` invariant PR2 needs, not a fix for a currently-observable bug.

## Todo
- [x] Determine whether current merge-always behavior is intentional or a bug (check test coverage/git history for render_group_component_with_format's tracker handling) — `git log -L` shows `tracker.merge_from(group_tracker)` was introduced (not moved) by commit 4d4804ec in the Aug 16 "resolve fallback policies centrally" refactor, immediately before the `values?` check with no prior guard; plain oversight, not a documented design decision.
- [x] If a bug, fix by moving tracker.merge_from after the values? empty-check (or conditioning it on Some(_)) — split into two calls: `advance_issued_occurrences_from` (unconditional, before the `?`) and `merge_from` (after the `?`, only when the group rendered). Uniformly gating both fields on `?` broke `blank_nested_first_issued_still_advances_the_later_lane`; see the resolved note above.
- [x] Add a regression test: an empty/suppressed group followed by a component that should still be eligible to render the variable the suppressed group examined — no constructible vector exists for `rendered_vars`/`substituted_bases` in current code (see resolved note); the existing `blank_nested_first_issued_still_advances_the_later_lane` test already locks in the `issued_occurrences` half. Correctness of the split is instead guarded by `report-core.js`'s zero-diff before/after run.
- [ ] (deferred to GROUP_SELECT.md's own PR, since it needs the `select` field to exist) Add a regression test matching select: first's forcing case: a winning candidate containing a suppressed/empty nested group, followed by a component that depends on tracker state the nested group must not have touched

## Summary of Changes

Split `TemplateComponentTracker::merge_from` into two merge points in `render_group_component_with_format`: `advance_issued_occurrences_from` runs unconditionally (before the `render_group_child_values?` check), preserving the intentional "blank occurrence still counts" date-fallback semantics; the rest of `merge_from` (`rendered_vars`, `substituted_bases`) now runs only after the group is confirmed to have rendered. No currently-observable behavior change (report-core.js zero-diff across 35 embedded styles); this closes the tracker-isolation gap `docs/specs/GROUP_SELECT.md`'s `select: first` needs. PR: (to be opened on branch fix/2hr4-group-tracker-merge-order, stacked on docs/render-when-alternatives-decision).
