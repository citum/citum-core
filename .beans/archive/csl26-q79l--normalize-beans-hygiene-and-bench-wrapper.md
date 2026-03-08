---
# csl26-q79l
title: Normalize beans hygiene and bench wrapper
status: completed
type: task
priority: high
created_at: 2026-03-08T15:22:37Z
updated_at: 2026-03-08T15:29:51Z
---

## Goals

- [x] Normalize stale bean statuses and archive scrapped/completed residue
- [x] Make citum-bean the canonical beans hygiene/audit wrapper
- [x] Delegate repo hygiene script to citum-bean
- [x] Refactor scripts/bench-check.sh to current script conventions
- [x] Update AGENTS/docs/skill guidance to match the new contract
- [x] Verify wrapper, hygiene, and bench script behavior

## Summary of Changes

- Normalized the repo to upstream `beans` lifecycle terms, scrapped legacy `canceled` beans, archived terminal beans, and removed the stale open duplicate `csl26-1csf` in favor of archived completed bean `csl26-y7t8`.
- Reworked `citum-bean` so it now owns `next`, `audit`, and `hygiene`, and delegated `scripts/check-docs-beans-hygiene.sh` to that wrapper.
- Refactored `scripts/bench-check.sh` to a verb-based interface (`capture` / `compare`) with compatibility aliases and updated the related docs.
- Verified wrapper text/JSON output and hygiene flow. Real benchmark smoke runs hit a pre-existing `cargo bench --bench formats` panic in `crates/citum-schema/benches/formats.rs:14`.
