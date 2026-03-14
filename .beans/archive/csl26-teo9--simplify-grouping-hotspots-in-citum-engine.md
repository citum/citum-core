---
# csl26-teo9
title: Simplify grouping hotspots in citum-engine
status: completed
type: task
priority: normal
created_at: 2026-03-14T22:56:23Z
updated_at: 2026-03-14T23:06:24Z
---

Implemented a bounded rust-simplify wave for grouping helpers in citum-engine. Simplified SelectorEvaluator predicate composition, tightened GroupSorter helper boundaries, and extracted non-stateful grouped-rendering helpers without changing grouped citation behavior. Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo nextest run; targeted grouping and grouped-bibliography tests.

Opened PR #372 on branch codex/rust-simplify-grouping. Attempted to request Copilot review via gh api, but GitHub did not record a reviewer request or expose any PR checks for this branch.
