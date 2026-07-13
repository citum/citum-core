---
# csl26-7ip9
title: Cross-role names-collapse mechanism
status: in-progress
type: task
priority: low
tags:
    - schema
    - rendering
    - contributors
created_at: 2026-07-12T18:16:09Z
updated_at: 2026-07-13T11:44:58Z
parent: csl26-kcda
---

CSL schema#442: CitationCollapse enum is {citation-number} only — no
mechanism to collapse multiple contributor-role lists into one combined
name list (e.g. "Doe, J. (Writer), & Smith, J. (Director)" for multimedia
items with distinct writer/director credits).

- [x] Design a cross-role names-collapse option, distinct from the
      existing citation-number CitationCollapse variant —
      docs/specs/CROSS_ROLE_CONTRIBUTOR_LISTS.md (Draft)
- [ ] Implement the merged contributor list mechanism per the spec
