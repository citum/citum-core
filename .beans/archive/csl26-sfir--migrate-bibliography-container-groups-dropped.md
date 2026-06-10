---
# csl26-sfir
title: 'migrate: bibliography container groups dropped'
status: completed
type: bug
priority: high
created_at: 2026-06-10T16:56:45Z
updated_at: 2026-06-10T17:40:43Z
parent: csl26-vmcr
---

Cluster C2 (largest reach, ~24 close-miss styles in the 82-90% band) from docs/architecture/audits/2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md. Migrated bibliographies drop container/periodical groups: journal name, year, volume vanish from rendered entries, and citation-number prefixes are lost. Evidence: zeitschrift-fur-allgemeinmedizin renders 'Rodriguez M. Major Breakthrough...' where citeproc-js renders '16.Rodriguez M. Major Breakthrough... The New York Times. 2024'. One bounded migrate-research pass; measure with node scripts/report-migrate-sqi.js --corpus random --seed 20260610.

## Summary of Changes

Root cause: the occurrence-based compiler flattens CSL conditional branches into the default template as suppress:true placeholders, but the engine renders each variable once-only (first occurrence wins) — placeholders ahead of live components consumed their variables, dropping containers/dates/volumes for every type without a variant.

Fix: new pass crates/citum-migrate/src/passes/suppression.rs (strip_suppressed_variable_poison), wired into bibliography compilation. Removes suppressed components whose variable also renders live; keeps type-only placeholders as diff anchors.

Evidence: zeitschrift-fur-allgemeinmedizin 32/38 -> 38/38 strict. Random-100 (seed 20260610): 43 -> 52 styles at >=90%; mean 83.2 -> 86.3; author-date 57.5 -> 67.5%, numeric 48.5 -> 63.6%. Per-style: +16 / -2 (ocean-and-coastal-research 98.3->93.1, journal-of-cardiothoracic-and-vascular-anesthesia 82.8->74.1 — foreign-type branch components now leak where suppressed copies were accidental sinks; residual routed to C3/C4). Sentinels hold (200/200, 375/378). Engine consumption-semantics question -> bean csl26-y4o7.
