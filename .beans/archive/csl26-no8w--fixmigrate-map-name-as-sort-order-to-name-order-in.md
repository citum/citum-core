---
# csl26-no8w
title: 'fix(migrate): map name-as-sort-order to name-order in template'
status: completed
type: bug
priority: normal
created_at: 2026-04-17T18:24:28Z
updated_at: 2026-04-17T18:31:29Z
---

The compile_names() function in node_compiler.rs always sets name_order: None, ignoring the name-as-sort-order attribute from CSL. This causes bibliography entries to use given-first name order instead of family-first. Fix: map NameAsSortOrder::First → FamilyFirstOnly, NameAsSortOrder::All → FamilyFirst.

## Summary of Changes

Identified that compile_names() in node_compiler.rs (template_compiler crate) always set name_order: None, ignoring name-as-sort-order from the CSL <name> element.

Fix: map NameAsSortOrder::First → NameOrder::FamilyFirstOnly, NameAsSortOrder::All → NameOrder::FamilyFirst.

Evidence: chicago-author-date --force-migrate reduced contributors:value_mismatch from 8 to 1. 2584 CSL styles use name-as-sort-order. All 1053 tests pass; core quality gate passes (147 styles at 1.0 fidelity).
