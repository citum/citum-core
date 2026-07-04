---
# csl26-nj72
title: Comprehensive review of citum-engine crate
status: completed
type: task
priority: normal
created_at: 2026-07-04T01:32:12Z
updated_at: 2026-07-04T01:40:43Z
---

Careful, comprehensive code review of crates/citum_engine. Findings recorded in a date-stamped audit document in docs/architecture/audits/ on a new branch.

## Summary of Changes

Reviewed citum-engine (~45k lines): full read of the API layer, processor
spine, FFI, and grouping/sorting; mechanical panic/unsafe/TODO scans crate-
wide. Baseline verified: clippy -D warnings clean, 846/846 nextest pass.

Findings recorded in docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md
on branch audit/citum-engine-review-2026-07: 2 high (process::exit(1) in
Processor::process_document; hardcoded harvard-cite-them-right style id in
the engine), 10 medium (heading type-name hack, session/tier-1 duplication,
uncached Sorter, per-component Config clones, pointer-keyed disambiguation
cache, etc.), 7 low. Follow-up beans suggested in the audit doc.
