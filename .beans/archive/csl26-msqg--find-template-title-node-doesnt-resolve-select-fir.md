---
# csl26-msqg
title: 'find_template_title_node doesn''t resolve select: first winners'
status: completed
type: bug
priority: normal
tags:
    - engine
    - rendering
    - fidelity
created_at: 2026-09-08T15:13:36Z
updated_at: 2026-09-09T00:22:42Z
---

values/contributor/substitute.rs::find_template_title_node walks render_when/suppress to find a title node, same shape as sorting.rs's date/contributor finders (GROUP_SELECT.md), but wasn't in that spec's Acceptance Criteria and is still structural-first for a select: first group. A third independent 'which branch renders' walker with the same gap -- candidate for sharing crate::values::select_group_children (values/mod.rs) once a concrete title-substitution style needs select: first.

## Summary of Changes

Fixed via an adversarial code review of this PR's implementation. find_template_title_node now routes through crate::values::select_group_children (the same select-aware winner resolver sorting.rs uses) instead of raw &group.group, so a select:first group's losing first candidate no longer supplies title-rendering: from-template substitution formatting. New regression test in values/tests.rs, verified to fail without the fix.
