---
# csl26-7ip9
title: Cross-role names-collapse mechanism
status: completed
type: task
priority: low
tags:
    - schema
    - rendering
    - contributors
created_at: 2026-07-12T18:16:09Z
updated_at: 2026-07-13T19:50:41Z
parent: csl26-kcda
---

CSL schema#442: CitationCollapse enum is {citation-number} only — no
mechanism to collapse multiple contributor-role lists into one combined
name list (e.g. "Doe, J. (Writer), & Smith, J. (Director)" for multimedia
items with distinct writer/director credits).

- [x] Design a cross-role names-collapse option, distinct from the
      existing citation-number CitationCollapse variant —
      docs/specs/CROSS_ROLE_CONTRIBUTOR_LISTS.md (Active v1.0)
- [x] Implement the merged contributor list mechanism per the spec

Completed cross-role contributor lists across native reference/schema models,
locale terms, rendering, suppression, sorting/disambiguation, CSL migration,
fixtures, generated schemas, and APA behavior. Native Citum data explicitly
records multi-role identity; strict NFC name comparison is confined to legacy
CSL-JSON conversion. Validation evidence is recorded in PR #1052.
