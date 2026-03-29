---
# csl26-jgt4
title: Archival and unpublished source support
status: in-progress
type: feature
created_at: 2026-03-28T21:04:14Z
updated_at: 2026-03-29T00:00:00Z
---

Introduce ArchiveInfo and EprintInfo structs to model archival citations and preprints properly. Resolves the legacy `archive_location` semantic collision (shelfmark vs. city) by introducing spec-aligned SimpleVariables `archive-location` (shelfmark) and `archive-place` (city), adds `archive-collection`/`archive-url` fields, and adds `MonographType::Preprint` replacing the arXiv medium-hack. Spec: docs/specs/ARCHIVAL_UNPUBLISHED_SUPPORT.md

2026-03-29 follow-up: completed the public communication pass after merge. The checked-in archival demo now shows canonical structured `archive-info` hierarchy fields (`collection-id`, `series`, `box`, `folder`, `item`) instead of collapsing them into `location`, the demo style renders the corresponding `archive-*` variables directly, the examples page explains that `location` is a display override / legacy fallback, and the style author guide now points authors to `archive-box` / `archive-folder` / `archive-item` for structured archival layouts.

Open follow-up keeping this bean in progress: locale-backed labels for structured archival hierarchy components are not implemented yet. Current examples use style-authored English prefixes such as `Series`, `Box`, `Folder`, and `Item`. Before claiming engine-added localized labels for these fields, decide whether archival container labels belong in locale general terms, locator-style terms, or a dedicated archival term family; define singular/plural behavior (`box`/`boxes`, `folder`/`folders`, `item`/`items`), short-form abbreviations, and the conditions under which the engine should add labels versus styles rendering them explicitly.

## Summary of Changes

- Added checked-in examples and docs guidance that use canonical structured `archive-info` hierarchy fields.
- Added rendering coverage for both canonical structured hierarchy output and legacy `archive-location` fallback behavior.
- Corrected `archive_location()` lookup so structured `archive-info.location` is preferred before legacy flat `archive_location`.
- Recorded the unresolved locale-label design work as explicit follow-up on this bean.
