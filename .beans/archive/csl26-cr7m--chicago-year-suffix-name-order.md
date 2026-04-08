---
# csl26-cr7m
title: Resolve Chicago anonymous year-suffix and name-order leaks
status: completed
type: task
priority: high
created_at: 2026-04-03T17:45:00Z
updated_at: 2026-04-08T21:22:00Z
---

Target the processor/engine rearrangements that keep the Chicago 18 supplemental corpus from green-filing due to anonymous/titleless year-suffix leakage, display-as-sort name-order quirks, and related bibliography sort defects.

## Tasks
- [x] Extract a reproducible cluster (e.g., anonymous entries and named legal acts) from the Chicago 18 supplemental benchmark via `scripts/extract-rich-benchmark-cluster.js` so the failure signal is isolated.
- [x] Investigate whether the leakage is due to anonymous entries emitting year suffixes in the source processor or due to `group_sort`/`display-as-sort` interactions in the renderer and note the responsible code paths (`processor/disambiguation.rs`, `processor/rendering/grouped/core.rs`).
- [x] Implement fixes that keep anonymized bibliography rows in strict year-order, suppress additional suffixes, and honor the configured `display-as-sort: first` per the locale/option overrides.
- [x] Add regression tests that cover the targeted cluster and confirm there is no regression in the primary oracle or the Chicago rich-input report.
- [x] Document the planned resolution in `docs/specs/TITLE_TEXT_CASE.md` or the relevant policy so future passes understand why the ordering rules are now locked.

## Classification
- processor-defect → ensures the renderer/disambiguation combo treats anonymous rows consistently.

## Summary of Changes

- **Fix A** (`build_group_key`): anonymous entries now use a non-empty per-reference fallback key when the embedded reference id is empty or missing, so year-suffix grouping cannot collapse to `anon:`.
- **Fix B** (`calculate_hints` / `build_reference_cache`): `title_key` is populated whenever year-suffix disambiguation is active, even when a `group_sort` is configured.
- **Fix C** (`sort_group_for_year_suffix`): year-suffix groups are pre-sorted by `title_key` before the primary `GroupSorter` pass, preserving deterministic title order for entries that compare equal on the primary sort key.
- **Fix D** (`styles/chicago-author-date.yaml`): bibliography contributors now set `display-as-sort: first`, matching the Chicago 18 first-author inversion rule.
