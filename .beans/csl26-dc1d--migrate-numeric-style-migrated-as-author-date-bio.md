---
# csl26-dc1d
title: 'migrate: numeric style migrated as author-date (bio-protocol)'
status: todo
type: bug
priority: high
tags:
    - migrate
    - fidelity
created_at: 2026-06-14T11:20:14Z
updated_at: 2026-06-14T17:17:01Z
parent: csl26-vmcr
---

bio-protocol (CSL numeric, oracle '[16]') is migrated to render author-date '(Kuhn, 1962)'. The base/processing detection picks author-date for a numeric-class style. Converter-level: detect_processing_mode or base_detector mis-routes. Repro: node scripts/oracle.js styles-legacy/bio-protocol.csl --json --force-migrate. Tail also: journal-of-contemporary-water-research-and-education.

## Root Cause (traced 2026-06-14, deferred)

A numeric style inherits an author-date citation template via a CSL formatting-heritage link — cross-cutting, not bounded.

`styles-legacy/bio-protocol.csl` is numeric but declares `<link rel="template" href=".../apa"/>`. `lineage.rs::resolve_parent_link_target` treats that link as an inheritance parent and emits `extends: apa-7th`. apa-7th is author-date and carries a `citation.non_integral` sub-spec; bio-protocol overrides `citation.template` but not `non_integral`, so `merge_style_overlay` keeps APA's. At render, `resolve_for_mode(NonIntegral)` lets APA's author-date template overwrite the numeric `[citation-number]` template → `(Kuhn, 1962)` instead of `[1]`.

The principled fix (refuse an author-date parent when the child's processing mode is numeric/label) requires wiring processing-mode detection into lineage resolution, with regression risk across every template-linked style. Deferred from the csl26-vmcr bounded PR. Repro: `node scripts/oracle.js styles-legacy/bio-protocol.csl --json --force-migrate`.
