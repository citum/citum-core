---
# csl26-ucg3
title: Chicago notes legal/treaty note-flow (Bluebook)
status: todo
type: bug
priority: normal
created_at: 2026-06-21T11:49:07Z
updated_at: 2026-06-21T11:49:07Z
---

Chicago notes (`chicago-notes-18th.yaml`) legal/treaty note-flow defects found
while completing csl26-6qv3. These are **bucket B/D** (Bluebook-specialised,
flagged-not-guessed in `docs/architecture/audits/2026-06-20_STYLE_GUIDE_CONFORMANCE.md`)
and need a content/format decision before implementing — do **not** guess.

Observed in the rendered notes (CITATIONS, references-expanded.json):

- Leading comma before an author-less legal case: `, Brown v. Board of Education,
  347 U.S. Reports 483 (U.S. Supreme Court 1954).` — title substitutes for the
  absent author but a stray leading `, ` remains.
- Repeated treaty fields: `… U.S.T. 14 (1963): 1313, in U.S.T., vol. 14 (1963),
  U.S.T., 1313.` — volume/year/pages and the `U.S.T.` reporter render twice.
- Conference note double-year/pages: `"…Phrases" (2013): 3111–19, in
  _Proceedings of NIPS 2013_ (2013), 3111–19.` — year and pages rendered before
  AND after `in Proceedings`.

These touch the chicago-notes citation type-variants (deep note-flow) and the
legal-case/treaty Bluebook formatting. Confirm the intended Bluebook output with
the maintainer before changing.

Note: the chicago-notes *bibliography* is intentionally empty (notes-only style;
citeproc `chicago-notes.json` also emits `bibliography: []`) — not in scope here.
