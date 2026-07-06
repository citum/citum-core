---
# csl26-xshm
title: Remove or wire dead MigrationOutputPlan embedded-root variants
status: completed
type: task
priority: normal
created_at: 2026-07-06T18:42:20Z
updated_at: 2026-07-06T23:38:56Z
parent: csl26-al39
---

Audit F2+F7 (2026-07-06 migrate review): MigrationOutputPlan::CreateEmbeddedRootAndWrapper / UpgradeEmbeddedRootAndWrapper and requires_multi_artifact_write() are constructed nowhere in the repo; output_plan() can only return Standalone or ExistingWrapper. Either wire them to the embedded-root+wrapper workflow they anticipate or delete them until it exists. Also (F7): document in the lineage.rs module header that diff_value cannot express deletions (a wrapper silently inherits parent keys the standalone form lacked) and add a debug-log when a child drops a parent key.

## Summary of Changes

Removed `MigrationOutputPlan::CreateEmbeddedRootAndWrapper`/`UpgradeEmbeddedRootAndWrapper`
and `requires_multi_artifact_write()` (never constructed anywhere; `output_plan()`
only ever returns `Standalone` or `ExistingWrapper`), plus their dead test and the
now-unreachable match arms in `output_plan.rs`. Documented in `lineage.rs`'s module
header that `diff_value` cannot express deletions, and added a `tracing::debug!`
when a child drops a parent key so the silent inheritance is visible in debug logs.
