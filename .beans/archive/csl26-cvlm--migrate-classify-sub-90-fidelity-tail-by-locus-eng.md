---
# csl26-cvlm
title: 'migrate: classify sub-90 fidelity tail by locus (engine vs converter)'
status: completed
type: task
priority: high
created_at: 2026-06-14T11:01:59Z
updated_at: 2026-06-14T11:23:22Z
parent: csl26-vmcr
---

Confirm/deny the 'remaining migrate fidelity gaps are engine-level, not converter-level' assertion (crates/citum-migrate/CLAUDE.md, OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md, 2026-06-14 audit). Its sole concrete evidence csl26-y4o7 was completed 2026-06-12; the audit's recommended classification of the sub-90 tail was never run. Re-measure the live random-100 tail, classify each sub-90 style into engine/converter/genuinely-hard via oracle diffs, file beans per cluster, fix bounded clusters, correct stale instruction text.

## Tasks
- [x] Re-measure random-100 scorecard (seed 20260610), snapshot sub-90 tail
- [x] Classify each sub-90 style by failure locus via oracle --force-migrate diffs
- [x] Write classification audit doc
- [x] File one bean per distinct root-cause cluster, parented to csl26-vmcr
- [x] Fix reasonably-bounded clusters (+ regression tests)
- [x] Correct stale instruction text + session memory

## Summary of Changes

Verdict: the blanket "remaining fidelity gaps are engine-level, not converter-level" assertion is stale and wrong. Cited evidence (csl26-y4o7) was completed 2026-06-12; the 33-style sub-90 tail is converter-dominated across all five classes. Durable record: docs/architecture/audits/2026-06-14_MIGRATE_FIDELITY_LOCUS_CLASSIFICATION.md.

Fixed two converter bugs leaving all 3 label-class styles broken (citation-label silently dropped): added CitationLabel arm to map_variable_to_number; added Label-mode detection to detect_processing_mode. Labels render ([] -> [Kuh62]); 3 regression tests; full gate green (1600 tests). Headline stayed 67/100 (compounding defects under binary threshold).

Residual clusters beaned: csl26-tzer, csl26-dc1d, csl26-c2um, csl26-ahxh, csl26-ya9b.
