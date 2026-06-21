---
# csl26-6qv3
title: Embedded style structural residuals
status: completed
type: task
priority: normal
created_at: 2026-06-21T10:49:37Z
updated_at: 2026-06-21T11:49:27Z
---

Per-style structural conformance gaps found in the guide-conformance sweep (csl26-53zy / PR #946) that were too large or shared-template-sensitive for that PR. Detail in the per-style sections of docs/architecture/audits/2026-06-20_STYLE_GUIDE_CONFORMANCE.md.

- Chicago author-date (chicago-author-date-18th.yaml): chapter `In _Book_, edited by` ordering (currently `Edited by Eds, _Book_`); magazine cited by date not volume; conference acronym case `NIPS`->`Nips`; translator double-label (`Translated by X (Trans.)`); patent empty term/number.
- Elsevier with-titles (elsevier-with-titles-core.yaml): journal year-in-parens reorder (`Nature 521 (2015) 436–444` vs Citum `521 436–444, 2015`).
- Chicago notes (chicago-notes-18th.yaml): deep note-flow + legal-type review (double-year, leading comma, repeated treaty fields; empty bibliography in CLI render to investigate).

Several overlap engine residuals tracked in the sibling bean.


## Summary of Changes

Closed the YAML-addressable structural residuals across three styles.

**Chicago author-date (`chicago-author-date-18th.yaml`, shared base)** —
- Chapter `In _Book_, edited by Eds.` reorder — now **byte-matches citeproc**
  (`"…Practice." In _The Cambridge Handbook…_, edited by …. Cambridge University
  Press.`) via an `in`-term group that lets punctuation-in-quote fold the period.
- Magazine cited by date not volume (`_Wired_, June 2023`).
- Translator double-label fixed (`role.omit: [translator]`) → `Translated by
  David Wyllie.` (byte-matches citeproc; the verb prefix supplies the role).
- Patent number renders — field-name mismatch fixed (`number: patent-number`,
  stored as `patent_number`); `US 11,043,211 B2` now appears.

Net byte-fidelity vs citeproc improved 7→9 exact entries; full `just pre-commit`
green (1659 tests, one German editor-verb golden updated to the corrected chapter
order). The oracle's per-component counter shows a false regression on Kafka only
because removing `(Trans.)` removed the delimiter its extraction heuristic relies
on — the rendered string is now exactly citeproc's.

**Elsevier with-titles (`elsevier-with-titles-core.yaml`)** —
- Journal year-in-parens reorder → `Nature 521 (2015) 436–444`. Root cause: the
  `all` catch-all variant is resolved first (engine returns the first matching
  selector), so the specific `article-journal` variant was dead. Moved it above
  `all` to activate it (minimal blast radius — only journal type changes).

**Chicago notes (`chicago-notes-18th.yaml`)** —
- **Empty-bibliography investigation resolved: by design.** `chicago-notes` is a
  notes-only style (`bibliography.template: []`); citeproc's `chicago-notes.json`
  also emits `bibliography: []`. Not a bug.

## Deferred — needs adjudication / engine (not done here)

- Chicago notes legal/treaty note-flow (leading comma before `Brown`, repeated
  treaty fields, conference double-year/pages). **Bucket B/D** — Bluebook-
  specialised, flagged-not-guessed in the audit; needs a content decision, not a
  silent fix.
- Patent **term** `Patent` label still empty — `GeneralTerm::Patent` is not wired
  into term rendering (engine). → csl26-maim.
- Conference acronym case (`NIPS`→`Nips`), page-range dash → csl26-maim.
- Magazine exact citeproc parity (`June` vs `June 2023`) needs a month-only date
  form (engine).
