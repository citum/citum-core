---
# csl26-cksb
title: 'Rust simplify+refine: structural.rs contributor DRY + idiom pass'
status: completed
type: task
priority: normal
created_at: 2026-05-28T23:21:07Z
updated_at: 2026-05-28T23:32:42Z
parent: csl26-rfct
---

DRY the near-identical contributor-reconciliation block in all 5 From<*Deser> impls in crates/citum-schema-data/src/reference/types/structural.rs (1508 lines, 4 impls ≥238 lines each). Extract reconcile_role_shorthands helper, drop the lone clippy::indexing_slicing allow, assess field-copy macro for concision. One PR: refactor/structural-simplify-refine.\n\n## Todo\n\n- [x] Extract reconcile_role_shorthands helper (DRY all 5 From impls)\n- [x] Drop clippy::indexing_slicing allow (use .first())\n- [x] Run full gate + oracle no-diff\n- [x] Open PR refactor/structural-simplify-refine

## Summary of Changes\n\nExtracted reconcile_contributors helper (ContributorViews struct + fold+collect in one call). Replaced 5 repetitive fold/collect blocks across all From<*Deser> impls. Dropped #[allow(clippy::indexing_slicing)] via slice pattern. 1420/1420 tests pass, oracle no-diff, quality gate green. PR #831.
