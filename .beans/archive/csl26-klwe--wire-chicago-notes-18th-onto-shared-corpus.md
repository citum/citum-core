---
# csl26-klwe
title: Wire chicago-notes-18th onto shared corpus
status: scrapped
type: task
priority: normal
created_at: 2026-06-30T15:07:11Z
updated_at: 2026-06-30T15:11:50Z
parent: csl26-40n4
---

Follow-up from csl26-8br0: notes-18th cannot use the shared Chicago corpus until it has a bibliography surface (bibliography.template is [] today) or citeproc-oracle supports a citation-only scope (scripts/lib/verification-policy.js asserts scope != citation). Once either lands, wire chicago-notes-18th onto the shared-corpus benchmark_run in scripts/report-data/verification-policy.yaml to match the other three Chicago variants.

## Todo
- [ ] Decide: add a bibliography surface to chicago-notes-18th, or lift the citeproc-oracle citation-scope restriction (or both)
- [ ] Implement the chosen path
- [ ] Wire chicago-notes-18th onto the shared-corpus benchmark_run
- [ ] Confirm all four Chicago variants report on the shared corpus

## Reasons for Scrapping

Accidental duplicate of csl26-qy4d, created when an earlier 'beans create' invocation errored on its output pipe (the bean was written before the pipe failed). csl26-qy4d is the canonical follow-up.
