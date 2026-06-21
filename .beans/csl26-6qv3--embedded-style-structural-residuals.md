---
# csl26-6qv3
title: Embedded style structural residuals
status: todo
type: task
priority: normal
created_at: 2026-06-21T10:49:37Z
updated_at: 2026-06-21T10:49:37Z
---

Per-style structural conformance gaps found in the guide-conformance sweep (csl26-53zy / PR #946) that were too large or shared-template-sensitive for that PR. Detail in the per-style sections of docs/architecture/audits/2026-06-20_STYLE_GUIDE_CONFORMANCE.md.

- Chicago author-date (chicago-author-date-18th.yaml): chapter `In _Book_, edited by` ordering (currently `Edited by Eds, _Book_`); magazine cited by date not volume; conference acronym case `NIPS`->`Nips`; translator double-label (`Translated by X (Trans.)`); patent empty term/number.
- Elsevier with-titles (elsevier-with-titles-core.yaml): journal year-in-parens reorder (`Nature 521 (2015) 436–444` vs Citum `521 436–444, 2015`).
- Chicago notes (chicago-notes-18th.yaml): deep note-flow + legal-type review (double-year, leading comma, repeated treaty fields; empty bibliography in CLI render to investigate).

Several overlap engine residuals tracked in the sibling bean.
