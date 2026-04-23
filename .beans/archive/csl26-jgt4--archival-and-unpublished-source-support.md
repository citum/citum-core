---
# csl26-jgt4
title: Archival and unpublished source support
status: completed
type: feature
created_at: 2026-03-28T21:04:14Z
updated_at: 2026-04-23T15:08:22Z
---

Introduce ArchiveInfo and EprintInfo structs to model archival citations and preprints properly. Resolves the legacy `archive_location` semantic collision (shelfmark vs. city) by introducing spec-aligned SimpleVariables `archive-location` (shelfmark) and `archive-place` (city), adds `archive-collection`/`archive-url` fields, and adds `MonographType::Preprint` replacing the arXiv medium-hack. Spec: docs/specs/ARCHIVAL_UNPUBLISHED_SUPPORT.md

2026-03-29 follow-up: completed the public communication pass after merge. The checked-in archival demo now shows canonical structured `archive-info` hierarchy fields (`collection-id`, `series`, `box`, `folder`, `item`) instead of collapsing them into `location`, the demo style renders the corresponding `archive-*` variables directly, the examples page explains that `location` is a display override / legacy fallback, and the style author guide now points authors to `archive-box` / `archive-folder` / `archive-item` for structured archival layouts.

2026-04-23 closure: the archival/eprint implementation this bean tracked is now complete on `main`, so this bean is closed. The remaining locale-backed label design for structured archival hierarchy components is intentionally deferred to follow-up bean `csl26-mlc2`. That follow-up will decide whether archival container labels belong in locale general terms, locator-style terms, or a dedicated archival term family; define singular/plural behavior (`box`/`boxes`, `folder`/`folders`, `item`/`items`) and short-form abbreviations; and specify when the engine should add labels versus styles rendering them explicitly.

## Summary of Changes

- Added checked-in examples and docs guidance that use canonical structured `archive-info` hierarchy fields.
- Added rendering coverage for both canonical structured hierarchy output and legacy `archive-location` fallback behavior.
- Corrected `archive_location()` lookup so structured `archive-info.location` is preferred before legacy flat `archive_location`.
- Deferred locale-backed archival hierarchy labels to follow-up bean `csl26-mlc2`.
