---
# csl26-6ncw
title: 'Case A: fixture asserting issued-year-only year-suffix (APA §8.15 reprint scenario)'
status: todo
type: task
priority: normal
created_at: 2026-05-29T11:14:49Z
updated_at: 2026-05-29T11:37:11Z
---

Add a native integration test in `crates/citum-engine/tests/citations.rs` that asserts the three-reprint scenario produces `(1926/1967a) (1926/1967b) (1927/1967c)`. Three references, same author, issued 1967, original-dates 1926/1926/1927. Citum already produces the correct output by design; this fixture locks it against regression.

Spec: docs/specs/DISAMBIGUATION.md §1.
