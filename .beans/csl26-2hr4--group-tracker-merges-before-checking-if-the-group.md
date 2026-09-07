---
# csl26-2hr4
title: Group tracker merges before checking if the group rendered
status: todo
type: bug
priority: high
tags:
    - engine
    - rendering
    - fidelity
created_at: 2026-09-06T23:13:42Z
updated_at: 2026-09-08T12:47:52Z
parent: csl26-8m2p
---

render_group_component_with_format (crates/citum-engine/src/processor/rendering/grouped/core.rs, ~line 1368) clones the tracker for a group's children, then unconditionally calls tracker.merge_from(group_tracker) BEFORE checking whether render_group_child_values actually produced output (the values? empty-check comes after the merge). So an empty/suppressed group's tracker mutations -- variable-once marks, substitution bookkeeping, date-fallback-first-issued flags -- still leak into the parent tracker even though the group rendered nothing.

This is a BLOCKING PREREQUISITE for select: first (docs/specs/GROUP_SELECT.md), not an independent someday investigation. Clone-and-discard at the select: first group's own boundary alone is insufficient -- the merge-before-check happens at EVERY level of nesting inside render_group_component_with_format, so if the WINNING candidate is itself a nested group containing a suppressed sub-group, that sub-group's tracker mutations are already baked into the candidate's own clone before select: first ever gets a say. Merging 'only the winner's' tracker still commits the pollution. select: first cannot be implemented with correct tracker isolation until this ordering is fixed (or explicitly proven safe as-is) -- see docs/specs/GROUP_SELECT.md's Implementation Notes and Acceptance Criteria for the dependency.

Impact on existing group: rendering is unclear -- may be intentional (once a variable is examined it should never be considered again regardless of group suppression) or may be a real bug (a suppressed group's exploratory tracker probing should not count). Needs investigation before deciding whether to fix.

## Todo
- [ ] Determine whether current merge-always behavior is intentional or a bug (check test coverage/git history for render_group_component_with_format's tracker handling)
- [ ] If a bug, fix by moving tracker.merge_from after the values? empty-check (or conditioning it on Some(_))
- [ ] Add a regression test: an empty/suppressed group followed by a component that should still be eligible to render the variable the suppressed group examined
- [ ] Add a regression test matching select: first's forcing case: a winning candidate containing a suppressed/empty nested group, followed by a component that depends on tracker state the nested group must not have touched
