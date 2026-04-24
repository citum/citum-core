---
# csl26-mlc2
title: Design locale-backed archive hierarchy labels
status: todo
type: feature
priority: normal
created_at: 2026-04-23T15:08:48Z
updated_at: 2026-04-24T12:14:13Z
parent: csl26-li63
blocked_by:
    - csl26-y3kj
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
