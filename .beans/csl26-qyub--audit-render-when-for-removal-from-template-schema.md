---
# csl26-qyub
title: Audit render-when for removal from template schema
status: todo
type: task
priority: normal
tags:
    - schema
    - rendering
    - styles
    - research
created_at: 2026-07-13T12:24:44Z
updated_at: 2026-07-13T12:25:06Z
blocking:
    - csl26-q05f
---

PR #1050 review raised a doctrine question: render-when (TemplateGroupCondition, field-present/field-absent) is the template language's only conditional mechanism, and Citum's declarative model otherwise pushes conditionality into options, presets, and type-variants (see the 2026-07-12 triage stance on schema#62/#320 declining template-conditional growth).

Current footprint (2026-07-13): 29 usages, all confined to the two embedded Chicago 18th styles (chicago-author-date-18th.yaml, chicago-notes-18th.yaml); citum-migrate never emits it; engine consumes it in ~5 sites. Usage patterns: recipient/title personal-communication shapes, URL-only-when-no-DOI, original-publication pairs, genre/archive-location gates.

- [ ] Map each of the 29 usages to an options/preset/type-variant replacement (or identify genuinely irreplaceable cases)
- [ ] Decide: remove render-when, or keep but freeze (no new condition kinds, documented as legacy escape hatch)
- [ ] If removing: refactor both Chicago 18th styles under fidelity gates and drop the schema field + engine sites

Related: docs/specs/CROSS_ROLE_CONTRIBUTOR_LISTS.md deliberately avoided growing render-when (same-person elision went to options.contributors.suppress instead).

Note: open bean csl26-q05f (expose original-publication fields to render-when) would grow the mechanism and is blocked on this audit's outcome.
