---
# csl26-shco
title: Investigate chicago-shared-corpus converter/engine defects (8/15 failing)
status: completed
type: bug
priority: medium
created_at: 2026-07-01T00:00:00Z
updated_at: 2026-07-03T19:41:11Z
parent: csl26-h7oc
---

`chicago-notes-18th`'s `chicago-shared-corpus` benchmark (registered in
`node scripts/report-core.js --style chicago-notes-18th`,
`officialSupplemental.chicago-shared-corpus`) was independently confirmed at
46.7% (7/15) on `origin/main`, barely above its own `minPassRate: 0.46` gate.
The richer shared corpus cases are in
`tests/fixtures/test-items-library/chicago-18th.json` /
`chicago-18th-citations.json`.

## Independent Correction

The original bean over-classified `chi-manuscript` as a missing conversion
route. A fresh pre-flight showed that `collection` routing and the manuscript
conversion contract already exist. The residual `chi-manuscript` failure was a
style/accessor/rendering issue: archive collection and archive name were not
being selected in the Chicago notes rendering path, and generic accessors
needed to expose the relevant converted facts.

The other seven shared-corpus failures split across style defects and
processor-side data reachability gaps:

- `chi-article-journal` needed Chicago note author shortening and DOI output.
- `chi-article-magazine` needed a title-less magazine branch that renders
  parent serial, section, and date.
- `chi-article-newspaper` needed review framing plus reachable event,
  reviewed-author, section, URL, and date facts.
- `chi-no-date` needed collection/series and issued date-range rendering.
- `chi-personal-communication` needed sender-to-recipient, date, archive
  location, archive collection, and archive name ordering.
- `chi-interview` needed interviewer, event place/date, posted date, medium,
  and URL ordering.
- `chi-broadcast` needed series, unlabelled season/episode text, title,
  writer/cast contributors, aired date, network, and duration.

## Approach

- [x] Verify CSL type `"collection"` and the `chi-manuscript` conversion
      contract before changing routing. The route already exists; the fix was
      style/accessor/rendering, not a new route.
- [x] Root-cause each of the other 7 failures individually per
      `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md` classification
      (style-defect / migration-artifact / processor-defect / intentional
      divergence) before touching any YAML.
- [x] Only after conversion/engine gaps are fixed, revisit
      `chicago-notes-18th.yaml`'s `article-magazine`, `article-newspaper`,
      `personal-communication`, `interview`, `broadcast`, and `no-date`-shaped
      type-variants for genuine style-defects, since some fraction of the
      current bad output may resolve once the underlying reference data
      populates correctly.
- [x] Re-run `node scripts/report-core.js --style chicago-notes-18th` after
      each fix; target the shared-corpus benchmark at 100% (or raise
      `minPassRate` deliberately with documented divergences per case).

## Summary of Changes

This change preserves serial-component `section`, review event facts,
broadcast event facts, writer/cast-style contributors, broadcaster/publisher,
duration/dimensions, and EDTF issued date ranges through conversion and engine
accessors. It also adds writer/performer template role plumbing and a
`collection-title` condition field, then regenerates schemas for the expanded
style surface.

`chicago-notes-18th.yaml` was tuned for the eight independently confirmed
shared-corpus failures: journal DOI and et al. shortening, title-less magazine
items, newspaper reviews, no-date collection ranges, manuscript archive
collections, personal communications, interviews, and broadcasts.

Final evidence:

- Shared corpus oracle for `styles-legacy/chicago-notes.csl` against
  `chicago-18th.json` / `chicago-18th-citations.json`: 15/15.
- `node scripts/report-core.js --style chicago-notes-18th`: improved from
  66/74 overall, quality 0.857, shared corpus 7/15 on `origin/main` to 73/74
  overall, quality 0.914, shared corpus 15/15 on this branch.
- Focused conversion/engine/style tests passed for the new accessors,
  date-range conversion, broadcast preservation, note position behavior, and
  humanities fixture preservation.
- `NODE_PATH=/home/bruce/Code/citum/citum-core/scripts/node_modules just
  pre-commit` passed with 1718/1718 tests.
