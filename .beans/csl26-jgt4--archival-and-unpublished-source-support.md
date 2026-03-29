---
# csl26-jgt4
title: Archival and unpublished source support
status: in-progress
type: feature
created_at: 2026-03-28T21:04:14Z
updated_at: 2026-03-28T21:04:14Z
---

Introduce ArchiveInfo and EprintInfo structs to model archival citations and preprints properly. Resolves the legacy `archive_location` semantic collision (shelfmark vs. city) by introducing spec-aligned SimpleVariables `archive-location` (shelfmark) and `archive-place` (city), adds `archive-collection`/`archive-url` fields, and adds `MonographType::Preprint` replacing the arXiv medium-hack. Spec: docs/specs/ARCHIVAL_UNPUBLISHED_SUPPORT.md
