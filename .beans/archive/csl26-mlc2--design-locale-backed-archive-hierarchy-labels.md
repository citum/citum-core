---
# csl26-mlc2
title: Design locale-backed archive hierarchy labels
status: completed
type: feature
priority: normal
tags:
    - schema
    - locale
created_at: 2026-04-23T15:08:48Z
updated_at: 2026-04-27T12:07:57Z
parent: csl26-li63
---

Scope the deferred localization work for archival container labels introduced by csl26-jgt4. Define where archive hierarchy labels belong in the locale model, how singular/plural and short forms work, and when the engine should add labels versus requiring styles to render them explicitly.

Related: csl26-jgt4, docs/specs/ARCHIVAL_UNPUBLISHED_SUPPORT.md

Target variables and behaviors:
- archive-box
- archive-folder
- archive-item
- related archival container labels such as collection/series when engine-backed labels are desired

Expected outcome:
- decision on locale term family placement
- explicit singular/plural and abbreviation rules
- engine-backed rendering plan and acceptance tests for localized archival labels

## Summary of Changes

Added archive hierarchy locale terms (collection, series, box, folder, item) to en-US, fr-FR, and de-DE locales in messages: block with plural-aware rendering for container fields. Implemented resolved_archive_term() method in Locale to look up these terms. Modified ArchiveLocation variable resolution in the engine to fall back to assemble_archive_hierarchy() when location is absent, joining structured fields with locale-backed labels and comma separation. Documented assembly defaults in archival spec. Added archive term IDs to authoring guide catalog. Created locale tests in i18n.rs and integration tests in bibliography.rs. All 1112 tests passing.
