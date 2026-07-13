---
# csl26-qyub
title: Audit render-when for removal from template schema
status: in-progress
type: task
priority: normal
tags:
    - schema
    - rendering
    - styles
    - research
created_at: 2026-07-13T12:24:44Z
updated_at: 2026-07-13T16:04:45Z
---

PR #1050 review raised a doctrine question: render-when (TemplateGroupCondition, field-present/field-absent) is the template language's only conditional mechanism, and Citum's declarative model otherwise pushes conditionality into options, presets, and type-variants (see the 2026-07-12 triage stance on schema#62/#320 declining template-conditional growth).

Current footprint (2026-07-13): 29 usages, all confined to the two embedded Chicago 18th styles (chicago-author-date-18th.yaml, chicago-notes-18th.yaml); citum-migrate never emits it; engine consumes it in ~5 sites. Usage patterns: recipient/title personal-communication shapes, URL-only-when-no-DOI, original-publication pairs, genre/archive-location gates.

- [x] Map each of the 29 usages to an options/preset/type-variant replacement (or identify genuinely irreplaceable cases)
- [x] Select semantic-only removal or specified retention: **specified retention selected**
- [ ] Implement retention validation (empty/contradictory-condition rejection) and behavior tests under fidelity gates; promote spec Status to Active

Related: docs/specs/CROSS_ROLE_CONTRIBUTOR_LISTS.md deliberately avoided growing render-when (same-person elision went to options.contributors.suppress instead).

Note: csl26-q05f is completed; its original-publication condition fields are included in this removal audit.

## Decision (2026-07-13)

Specified retention selected over semantic-only removal. `docs/specs/RENDER_WHEN_CONTRACT.md` documents the normative wire contract and reviewed field-extension rules; this bean is the record of the decision and rejected alternatives.

Rejected alternatives:

- **Semantic-only removal** solves 8 of 29 sites (3 existing composition, 5 plausible policies) but has no honest home for the other 21 structural-layout sites: options cannot embed templates, and naming the decision (e.g. `content-layouts: interview: title-sensitive`) without a place for both template bodies is engine magic under a typed name.
- **First-renderable branching** is equivalent to `if probe has data { render A } else { render B }` for structural layouts — a second conditional mechanism, not an architectural alternative — and its no-nesting rule can't transcribe AD-B6 nested inside AD-B7 without either reintroducing nesting or rewriting that cluster under an undesigned mechanism anyway.

This reverses the *removal* assumption from the 2026-07-12 schema#62/#320 triage, not the anti-growth stance behind it. AND-combined conditions (AD-C1) and nesting (AD-B6 inside AD-B7) are already part of the existing, working mechanism and are retained as-is, not newly granted. The typed field vocabulary may grow through ordinary reviewed schema changes — the same governance as any other schema addition — when a real forcing case is genuinely a field-presence layout/value selection within one type, not a stand-in for a semantic option. There is no special freeze on render-when: there is simply no current proposal to add new operators (OR, comparisons, expressions), same as any unimplemented feature.

Remaining work (schema validation, behavior tests, schema regen, Status: Active promotion) is tracked implementation, not yet landed. Bean stays in-progress until that lands.
