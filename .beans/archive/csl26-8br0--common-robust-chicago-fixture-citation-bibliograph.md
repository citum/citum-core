---
# csl26-8br0
title: Common robust Chicago fixture (citation + bibliography, all variants)
status: completed
type: task
priority: high
created_at: 2026-06-30T14:29:49Z
updated_at: 2026-06-30T15:11:07Z
parent: csl26-40n4
blocked_by:
    - csl26-fr6f
---

Build one shared fixture with rich source types (book/chapter/periodical/media/archive/correspondence/recording/broadcast + original/event dates) exercising both citation and bibliography surfaces. Wire as benchmark_runs in scripts/report-data/verification-policy.yaml for all four Chicago variants. Add a bibliography surface to chicago-notes-18th (currently scopes: [citation] only) once the fixture exists. Replaces today's fragmented fixtures (references-expanded.json for author-date, references-humanities-note.json citation-only for notes, test-items-library/chicago-18th.json bibliography-only for author-date-18th).

## Todo
- [x] Design shared fixture covering required reference types/scenarios for all 4 variants
- [x] Build fixture JSON (reused chicago-18th.json refs + new chicago-18th-citations.json companion)
- [x] Wire benchmark_runs for the 3 bibliography-bearing variants (author-date-18th, shortened-notes-bibliography, taylor-and-francis); notes-18th deferred
- [x] Add bibliography scope to chicago-notes-18th policy entry -- deferred to csl26-qy4d (no bibliography surface; citeproc-oracle has no citation-only scope)
- [x] Verify via report-core.js: 3 variants report citation+bibliography on the shared corpus, status ok, no fidelity regression

## Summary of Changes

Landed the shared Chicago corpus as ground-prep for the substrate work (PR #986, second commit).

- Added tests/fixtures/test-items-library/chicago-18th-citations.json -- a citations companion to the existing rich 403-item chicago-18th.json refs fixture, covering single/multi-author, locator, multi-source, no-date, and representative source types.
- Wired a chicago-shared-corpus benchmark_run (citeproc-oracle, scope: both, count_toward_fidelity: false) onto chicago-author-date-18th, chicago-shortened-notes-bibliography, and taylor-and-francis-chicago-author-date. All three now measure citation AND bibliography output on one common fixture pair.
- chicago-notes-18th left off the shared corpus: it has no bibliography surface and citeproc-oracle rejects a citation-only scope. Tracked as follow-up bean csl26-qy4d.
- Diagnostic-only by design: no fidelity gate moved. Baselines recorded -- author-date/T&F citations 11/15, bibliography 298/402; shortened citations 6/15, bibliography 264/402. Promotion to gating is the substrate PR's job (csl26-h7oc).
- Note: chicago-18th.json has no event-date items (8 have original-date); event-date coverage can be added when a variant needs it.
