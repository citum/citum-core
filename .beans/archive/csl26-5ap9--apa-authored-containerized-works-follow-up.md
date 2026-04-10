---
# csl26-5ap9
title: apa authored-containerized works follow-up
status: completed
type: task
priority: high
created_at: 2026-04-09T15:40:00Z
updated_at: 2026-04-10T19:10:00Z
---

Own the remaining APA rich bibliography rows after the web-native and
container-packaging clusters are removed.

Current verified state (post PR #501 cleanup):
- baseline APA gate remains `40 / 40`
- reduced structural closure fixture for rows `71`, `73`, and `74`: `3 / 3`
- any remaining APA ordering-only cleanup is explicitly handed to
  `csl26-mwnt`

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
- Activated generic `role-substitute` behavior so bibliography-local
  substitute presets preserve top-level `role-substitute` chains.
- Restored the chapter "In" container-author/editor behavior without duplicate
  editor rendering when `container-author` is present.
- Added `archive-name`/`archive-location` group to the default template.
- Added event/session rendering support for APA chair + session packaging.
- Routed container-less legacy `article` intake through a preprint shape and
  added an APA `preprint` variant so report-style parentheticals can render
  `editor`, `translator`, and `No. 123445` without broadening generic
  article-journal behavior.

## Resolved sub-buckets (closed rows)

- audiovisual: `49`, `50`, `61`, `63` — song routing + numbering fix
- chapter / entry / proceedings: all rows except `71` (see below)
- archive cluster: resolved via archive group on default template

## Closed structural rows

### Row 71 — `27 Book chapter` (container-author + editor coexist)

- closed by preserving `role-substitute` chains across bibliography-config
  merges and using the shared role-resolution path for suppression
- result: the chapter "In" clause now keeps `container-author` and suppresses
  the explicit `editor` fallback when both are present

### Row 73 — `33 Conference presentation`

- closed structurally by routing legacy `type: event` rows into `Event`,
  preserving `chair`, `event-title`, and session container data, and adding an
  APA `event` type variant
- result: APA now renders year-month dates plus the chair/session
  `In ... Session title [Symposium]` packaging
- residual citeproc ordering/date-letter cleanup, if still needed after the
  full benchmark pass, belongs to `csl26-mwnt`

### Row 74 — `9 Preprint with archive`

- closed structurally by routing container-less legacy `article` intake to a
  preprint monograph shape and adding an APA `preprint` type variant
- result: the parenthetical now includes `editor`, `translator`, and
  `No. 123445` on the actually exercised path
- residual citeproc year-letter cleanup, if still present on the full
  benchmark, belongs to `csl26-mwnt`

## Acceptance

- every row originally owned by this bean is structurally closed or handed to a
  narrower successor bean with explicit classification
- baseline APA remains `40 / 40`
- no unknown residuals remain in this bucket

## Stop-Loss Rule

- Stop after 2 distinct implementation attempts per sub-bucket with no
  net gain.
- Reclassify immediately as `style-defect`, `processor-defect`,
  `migration-artifact`, or explicit divergence and record the handoff.
