---
# csl26-cksb
title: 'Rust simplify+refine: structural.rs contributor DRY + idiom pass'
status: in-progress
type: task
priority: normal
created_at: 2026-05-28T23:21:07Z
updated_at: 2026-05-28T23:30:22Z
parent: csl26-rfct
---

DRY the near-identical contributor-reconciliation block in all 5 From<*Deser> impls in crates/citum-schema-data/src/reference/types/structural.rs (1508 lines, 4 impls ≥238 lines each). Extract reconcile_role_shorthands helper, drop the lone clippy::indexing_slicing allow, assess field-copy macro for concision. One PR: refactor/structural-simplify-refine.\n\n## Todo\n\n- [x] Extract reconcile_role_shorthands helper (DRY all 5 From impls)\n- [x] Drop clippy::indexing_slicing allow (use .first())\n- [x] Run full gate + oracle no-diff\n- [ ] Open PR refactor/structural-simplify-refine
