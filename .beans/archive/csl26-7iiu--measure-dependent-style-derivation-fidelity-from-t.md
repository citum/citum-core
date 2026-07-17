---
# csl26-7iiu
title: Measure extends-delta derivability of independent styles from tuned parents
status: completed
type: task
priority: high
tags:
    - migrate
    - fidelity
    - scorecard
created_at: 2026-07-17T15:43:01Z
updated_at: 2026-07-17T17:54:19Z
---

CSL dependent styles are already zero-cost registry aliases (STYLE_ALIASING.md: 7,987 dependents -> ~300 parents). The remaining per-style migration cost is the ~2k+ independent styles, each synthesized from scratch even when it is a near-clone of a tuned parent. Measure: what fraction of the random-corpus independent styles can be expressed as a small extends delta over a tuned/embedded parent at fidelity >= their current synthesized result. Reuse base_detector, lineage/wrapper emission, template_diff; csl26-b4h2 (scripted hidden parent-candidate discovery) is the natural front end. This fraction is the true remaining cost model. Context: docs/architecture/audits/2026-07-17_MIGRATION_APPROACH_STRATEGIC_REVIEW.md

## Approach

Extend the completed csl26-b4h2 tool (scripts/find-alias-candidates.js, behavioral fingerprinting over styles-legacy via citeproc-js) from exact-alias discovery to near-clone discovery: candidates below the >=0.98 alias threshold but above a delta-worthiness floor become extends-delta candidates instead of full synthesis.

## Tasks

- [x] Add a near-clone band to find-alias-candidates.js output (e.g. 0.80-0.98 similarity vs tuned/embedded parents), TSV report over the seeded random-100 corpus
- [x] For each near-clone pair, derive an extends wrapper via existing lineage/template_diff machinery and render it
- [x] Measure: fraction of the random-100 corpus expressible as a small delta at combined fidelity >= its current synthesized result (same strict oracle instrument, seed 20260610)
- [x] Record results as a date-stamped audit in docs/architecture/audits/ with a keep/expand/stop decision
- [x] If the fraction is material, file follow-up bean to route migrate's family-candidate path (output_plan.rs) through delta derivation by default (resolved negatively: mean delta -14.9, default routing not justified; expansion beans filed instead)

## Summary of Changes

- Instrument: band column + --include-registered in scripts/find-alias-candidates.js; new scripts/measure-delta-derivability.js (standalone vs --family-candidate --minimize-wrapper comparison, --concurrency flag, laptop-safe default 2).
- Measurement (audit: docs/architecture/audits/2026-07-17_EXTENDS_DELTA_DERIVABILITY.md): 94% of 2,844 independents have a >=0.80 behavioral neighbor; 59% at >=0.98. Random-100: 6/38 near-clones delta-expressible (winners +12.2 pts mean); styles/ corpus: 1/28 (+31.8); 17/66 standalone baselines broken (type-variants anchor bug). 102/141 checked-in styles in the >=0.98 alias band.
- Decision: expand instrument, do not change migrate routing. Follow-ups: csl26-b0ud (anchor bug), csl26-10lt (top-k/embedded-only/memory), csl26-8x90 (alias-band consolidation).
- Artifacts: scripts/report-data/{alias-candidates-band,alias-candidates-band-registered,delta-derivability-random100,delta-derivability-styles}-2026-07-17.*
