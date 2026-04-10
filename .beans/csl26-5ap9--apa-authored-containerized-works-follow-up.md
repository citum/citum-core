---
# csl26-5ap9
title: apa authored-containerized works follow-up
status: in-progress
type: task
priority: high
created_at: 2026-04-09T15:40:00Z
updated_at: 2026-04-09T20:45:00Z
---

Own the remaining APA rich bibliography rows after the web-native and
container-packaging clusters are removed.

Current verified state (post PR #500):
- baseline APA gate remains `40 / 40`
- supplemental APA diagnostic benchmark: `71 / 74` (up from `54 / 74`)
- 3 rows remain open: `71`, `73`, `74`

## Completed in PR #500

- Routed `song` CSL type through `AudioVisual` (Recording subtype);
  corrected Composer/Performer/Translator role assignment.
- Implemented `HasNumbering` for `AudioVisualWork`; `volume()` and
  `number()` were silently returning `None` for all AudioVisual refs.
- Applied `text-case` transforms in term renderer (was ignored).
- Removed all "Retrieved DATE, from URL" patterns (7 occurrences) from
  `apa-7th.yaml` and `preset-bases/apa-7th.yaml`.
- Added `song:` type variant template with catalog number, volume/track
  parenthetical, and `[medium]` bracket rendering.
- Restored `editor` contributor to chapter "In" group.
- Added `archive-name`/`archive-location` group to the default template.

## Resolved sub-buckets (closed rows)

- audiovisual: `49`, `50`, `61`, `63` — song routing + numbering fix
- chapter / entry / proceedings: all rows except `71` (see below)
- archive cluster: resolved via archive group on default template

## Remaining open rows

### Row 71 — `27 Book chapter` (container-author + editor coexist)

oracle: `In C. Author, Title of book...`
citum:  `In C. Author, S. S. Editor, editors., Title of book...`

Root cause: chapter "In" group renders both `container-author` AND
`editor` simultaneously. APA rule: when `container-author` is present,
suppress editors from the "In" clause.

Fix path: need conditional suppression in the template engine — show
`editor` only when `container-author` is absent. This is a template
engine capability gap (no `fallback` or `if-empty` between contributor
components). Engine work required before this can be fixed in YAML.

### Row 73 — `33 Conference presentation`

oracle: `Author, F. (2013a, May). 33 Conference... In F. Chair & S. Chair
         (Chairs), Session title [Symposium]...`
citum:  `Author, F. (2013g). 33 Conference... [Symposium], Session title...`

Two distinct issues:
1. **Year-suffix order** — `2013g` vs `2013a`; disambiguation ordering
   across the full fixture differs from citeproc-js. Owned by
   `csl26-mwnt`.
2. **Month in date** — oracle renders `(2013a, May)`; citum renders year
   only. The `event-date` field in CSL extra notes carries the month.
3. **Session/chair format** — oracle renders `In F. Chair & S. Chair
   (Chairs), Session title`; citum omits the "In" clause entirely.
   The `event` type variant needs a chair/session rendering block.

### Row 74 — `9 Preprint with archive`

oracle: `Author, A. A. (2018d). 9 Preprint with archive (A. A. Editor,
         editors.; A. A. Translator, Trans.; No. 123445). PsyArXiv...`
citum:  `Author, A. A. (2018q). 9 Preprint with archive (A. A. Translator,
         Trans.)`

Two distinct issues:
1. **Year-suffix order** — `2018q` vs `2018d`. Owned by `csl26-mwnt`.
2. **Missing editor in parenthetical** — editor is not rendering inside
   the `(...)` info group for preprints/reports. The preprint/report
   template needs an `editor` component inside the parenthetical group.
3. **Missing report number** — `No. 123445` absent; number accessor for
   this reference type likely needs a fix or the template needs a
   `number: report-number` component.

## Remaining tasks

- [ ] Engine: add conditional suppression (show editor only when
      container-author absent) so row 71 can be fixed in YAML.
- [ ] Style YAML: add `event:` type variant with chair/session "In" block
      for conference presentations (row 73).
- [ ] Style YAML: add `editor` component inside preprint parenthetical
      group; verify report-number accessor for preprint type (row 74).
- [ ] Delegate year-suffix rows 73 and 74 to `csl26-mwnt` once structural
      data is rendering correctly.

## Acceptance

- every row currently owned by this bean either matches exactly or is
  moved to a narrower successor bean with explicit classification
- baseline APA remains `40 / 40`
- no unknown residuals remain in this bucket

## Stop-Loss Rule

- Stop after 2 distinct implementation attempts per sub-bucket with no
  net gain.
- Reclassify immediately as `style-defect`, `processor-defect`,
  `migration-artifact`, or explicit divergence and record the handoff.
