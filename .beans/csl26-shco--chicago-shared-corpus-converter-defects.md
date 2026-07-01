---
# csl26-shco
title: 'Investigate chicago-shared-corpus converter/engine defects (8/15 failing)'
status: todo
type: bug
priority: medium
created_at: 2026-07-01T00:00:00Z
updated_at: 2026-07-01T00:00:00Z
---

`chicago-notes-18th`'s `chicago-shared-corpus` benchmark (registered in
`node scripts/report-core.js --style chicago-notes-18th`,
`officialSupplemental.chicago-shared-corpus`) sits at 46.7% (7/15), barely
above its own `minPassRate: 0.46` gate. The primary oracle fixture
(`node scripts/oracle.js styles-legacy/chicago-notes.csl --json`, the small
20-case set) is now at 100% after the anchor/locale-term/locator-gap fixes in
this PR — these 8 remaining failures are a separate, richer fixture set
(`tests/fixtures/test-items-library/chicago-18th.json` /
`chicago-18th-citations.json`).

## Why this is not a style-YAML fix

One case (`chi-manuscript`) was root-caused: the fixture's `note` field
carries a Zotero/Better-BibTeX type-override hack (`"type: collection"`),
which `csl_legacy::csl_json::Reference::parse_note_field_hacks`
(`crates/csl-legacy/src/csl_json.rs:413-415`) unconditionally applies over
the top-level CSL `"type": "manuscript"`. `"collection"` has **no routing
arm** in `crates/citum-schema-data/src/reference/conversion/mod.rs:346-369`,
so it falls through to a generic monograph/document default with almost no
fields populated — which is why the style's `manuscript:` type-variant never
even runs (confirmed empirically: a temporary marker string injected into
that type-variant never appeared in rendered output). This is a
`processor-defect`/converter gap, not a defect in `chicago-notes-18th.yaml`.

The other 7 failures were not root-caused to this depth but show symptoms
suggesting similarly deep, unrelated causes rather than simple template
fixes:

- `chi-article-journal` — author `et al.` truncation and DOI missing from
  Citum output (oracle truncates to 3 authors + et al. and shows the DOI;
  Citum output currently shows the full contributor list untruncated and
  drops the DOI entirely)
- `chi-article-magazine` — Citum output nearly empty (`", Gourmet (2000)"`)
  vs. oracle's full `"Gourmet, Kitchen Notebook, Sep 2000"` — most fields
  not rendering
- `chi-article-newspaper` — missing review-of-recital framing, venue, date
  format, and URL
- `chi-no-date` — missing series/date-range info (`"1st series
  (1803–1820)"`)
- `chi-personal-communication` — garbled component order and missing
  archive fields
- `chi-interview` — garbled component order (interviewer/date/URL out of
  CMOS18 order) and missing venue/date-posted
- `chi-broadcast` — garbled ordering and wrong number label (`"nos. season
  3, episode 10"` should be "season 3, episode 10" with no label, or a
  different label entirely) plus missing writer/cast credits

## Approach

- [ ] Route CSL type `"collection"` (and audit for other unrouted CSL 1.0
      types) in `crates/citum-schema-data/src/reference/conversion/mod.rs`,
      deciding whether it should share `MonographType::Manuscript` handling
      or get its own `ClassExtension` treatment.
- [ ] Root-cause each of the other 7 failures individually per
      `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md` classification
      (style-defect / migration-artifact / processor-defect / intentional
      divergence) before touching any YAML.
- [ ] Only after conversion/engine gaps are fixed, revisit
      `chicago-notes-18th.yaml`'s `article-magazine`, `article-newspaper`,
      `personal-communication`, `interview`, `broadcast`, and `no-date`-shaped
      type-variants for genuine style-defects, since some fraction of the
      current bad output may resolve once the underlying reference data
      populates correctly.
- [ ] Re-run `node scripts/report-core.js --style chicago-notes-18th` after
      each fix; target the shared-corpus benchmark at 100% (or raise
      `minPassRate` deliberately with documented divergences per case).
