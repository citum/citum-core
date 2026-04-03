---
# csl26-cr7m
title: Resolve Chicago anonymous year-suffix and name-order leaks
status: todo
type: task
priority: high
created_at: 2026-04-03T17:45:00Z
updated_at: 2026-04-03T17:45:00Z
---

Target the processor/engine rearrangements that keep the Chicago 18 supplemental corpus from green-filing due to anonymous/titleless year-suffix leakage, display-as-sort name-order quirks, and related bibliography sort defects.

## Tasks
- [ ] Extract a reproducible cluster (e.g., anonymous entries and named legal acts) from the Chicago 18 supplemental benchmark via `scripts/extract-rich-benchmark-cluster.js` so the failure signal is isolated.
- [ ] Investigate whether the leakage is due to anonymous entries emitting year suffixes in the source processor or due to `group_sort`/`display-as-sort` interactions in the renderer and note the responsible code paths (`processor/disambiguation.rs`, `processor/rendering/grouped/core.rs`).
- [ ] Implement fixes that keep anonymized bibliography rows in strict year-order, suppress additional suffixes, and honor the configured `display-as-sort: first` per the locale/option overrides.
- [ ] Add regression tests that cover the targeted cluster and confirm there is no regression in the primary oracle or the Chicago rich-input report.
- [ ] Document the planned resolution in `docs/specs/TITLE_TEXT_CASE.md` or the relevant policy so future passes understand why the ordering rules are now locked.

## Classification
- processor-defect → ensures the renderer/disambiguation combo treats anonymous rows consistently.
