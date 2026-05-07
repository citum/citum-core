# SQI Refinement Plan for Core Style Portfolio

Date: 2026-02-23
Scope: all core styles in `/styles` and SQI tooling in `scripts/report-core.js`.

## Working Update (2026-03-09)

This section records the current known state from active project tracking and
keeps the 2026-02-19 SQI snapshot below as the last full portfolio baseline.

Current known fidelity status:

1. Fidelity remains the hard gate for all core-style work.
2. The maintained core portfolio currently passes at fidelity `1.0` for all
   `146` styles in `node scripts/report-core.js`.
3. The current top-10 oracle batch is at portfolio-wide `100%` under the
   strict 18-scenario citation fixture set and current bibliography fixtures.

Current priority gaps affecting both fidelity consistency and SQI progress:

1. Volume-pages delimiter variance by style (comma vs colon).
2. DOI suppression behavior for styles that should omit DOI output.
3. Editor name-order differences by style (given-first vs family-first).

Immediate next SQI portfolio step:

1. Split residual fidelity gaps into narrower follow-on beans where the work is
   still active, instead of keeping them attached to the closed umbrella epic
   `csl26-u1in`.
2. Re-baseline this plan's "Current Snapshot" table the next time SQI metrics
   are intentionally refreshed from current code and style data.

## Historical Baseline (Pre-Phase B)

All core styles were at fidelity `1.0` in `node scripts/report-core.js`,
but SQI was uneven and had measurement-integrity failures:

| Style | SQI | Main Gaps |
|---|---:|---|
| elsevier-harvard | 0.0 | style YAML parse failure in SQI pipeline |
| chicago-notes | 51.0 | fallback robustness scored as 0 (note/no-bibliography shape) |
| american-medical-association | 61.5 | concision 0, preset usage 45 |
| apa-7th | 63.3 | concision 7.2, preset usage 45 |
| elsevier-with-titles | 64.3 | concision 11, preset usage 45 |
| springer-vancouver-brackets | 64.4 | concision 11.7, preset usage 45 |
| springer-basic-author-date | 75.3 | concision 55.2, preset usage 45 |
| springer-basic-brackets | 77.4 | concision 63.7, preset usage 45 |
| elsevier-vancouver | 84.4 | incremental concision/preset optimization |
| springer-socpsych-author-date | 86.2 | incremental preset optimization |
| taylor-and-francis-chicago-author-date | 86.7 | incremental preset optimization |

## Current Snapshot (After Phase B)

Date: 2026-02-19

All core styles remain at fidelity `1.0`.
SQI is now comparable across the full portfolio (no parse failures):

| Style | SQI | Notes |
|---|---:|---|
| american-medical-association | 91.4 | high concision + option preset usage |
| apa-7th | 86.2 | improved concision normalization for broad type coverage |
| chicago-notes | 86.3 | note-class handling with bibliography penalties removed |
| elsevier-harvard | 89.0 | `!custom` processing now measured correctly |
| elsevier-vancouver | 88.7 | stable high concision |
| elsevier-with-titles | 89.8 | improved concision + option presets |
| springer-basic-author-date | 87.9 | restored comparability after `!custom` parse fix |
| springer-basic-brackets | 90.3 | high concision + moderate preset usage |
| springer-socpsych-author-date | 89.5 | compact structure; preset extraction still available |
| springer-vancouver-brackets | 88.6 | improved concision normalization |
| taylor-and-francis-chicago-author-date | 89.7 | compact structure; preset extraction still available |

Portfolio summary:

1. Minimum SQI: `86.2` (target `>= 75` exceeded)
2. Median SQI: `89.0`
3. Styles at SQI `>= 75`: `11/11`
4. SQI parse failures: `0`

## Non-Negotiable Acceptance Rule

1. Fidelity is the hard gate.
2. SQI is a secondary optimization metric.
3. Reject any SQI gain that causes oracle regression.

## Portfolio Targets

1. Keep all core styles at fidelity `1.0`.
2. Raise minimum core SQI to `>= 75` (excluding styles with active metric exceptions).
3. Raise median core SQI by `>= 10` points.
4. Eliminate SQI computation failures (parse errors / non-comparable cases).

## Execution Plan

## Phase 0: Measurement Integrity

1. Fix style parse issues that invalidate SQI.
   - Completed: SQI loader supports `!custom` processing mappings.
2. Make SQI robust for note-only styles.
   - Completed: fallback robustness is treated as `N/A` for citation-only styles.
   - Completed: remaining subscores are reweighted.

## Phase 1: Portfolio Triage by Wave

Run:

```bash
node scripts/report-core.js > /tmp/core-report.json
```

Then prioritize:

1. Wave A (`SQI < 70`): elsevier-harvard, chicago-notes, AMA, APA, elsevier-with-titles, springer-vancouver-brackets
2. Wave B (`70 <= SQI < 80`): springer-basic-author-date, springer-basic-brackets
3. Wave C (`SQI >= 80`): elsevier-vancouver, springer-socpsych-author-date, taylor-and-francis-chicago-author-date

## Phase 2: Shared Preset Extraction (Cross-Style)

Extract and reuse common structures across author-date, numeric, and note families.

Candidate bibliography pattern presets:

1. Author-date journal spine:
   - contributor -> year -> title -> parent-serial -> volume/issue/pages -> DOI/URL
2. Numeric journal spine:
   - contributor -> title -> parent-serial -> year/volume/pages -> DOI
3. Chapter-in-book block:
   - chapter title -> `In` + editor -> parent monograph -> pages
4. Legal case block:
   - case title + volume/reporter/page + authority/date
5. Patent block:
   - contributor + year + title + patent term/number + issued date
6. Online resource tail:
   - URL + accessed-date rendering

Candidate options preset extraction:

1. `contributors`: convert explicit repeated configs to preset forms
   (`apa`, `harvard`, `springer`, `vancouver`, `chicago`) plus minimal overrides.
2. `dates`: move to preset (`long`, `short`, `numeric`) where possible.
3. `titles`: use `scientific`, `humanities`, `apa`, `chicago` baselines.
4. `substitute`: move to `standard`/`editor-first`/`title-first` where compatible.

## Phase 3: Style Refactor Waves

For each style in a wave:

1. Refactor toward shared presets and a smaller base template spine.
2. Keep only genuine type-specific structural outliers in `type-variants`.
3. Prefer presets and well-designed generic templates over `type-variants`; use
   `type-variants` only when a reference type needs a structurally different component set.
4. Re-run oracle after each refactor commit:
   - `node scripts/oracle.js styles-legacy/<style>.csl --json`

## Phase 4: SQI Scoring Alignment

Improve SQI so it rewards real maintainability changes:

0. Type Coverage & Fallback Robustness:
   - incorporate diff-based `type-variants` into explicit type coverage accounting.
   - ensure `type-variants` satisfy fallback robustness checks the same as full `type-templates`.

1. Preset detection:
   - count template-level `extends` and `preset` entries.
   - count options-level preset strings plus object-form preset references.
   - weight by impact (template preset > options preset).
2. Concision:
   - normalize component target thresholds by style class (`author-date`/`numeric`/`note`).
   - apply dynamic target bonuses when styles intentionally cover many type templates
     or diff-based `type-variants`.
   - discount diff operations (`modify`, `remove`, `add`, etc.) within `type-variants`     from component count and duplication penalties to reward surgical overrides.
   - reduce over-penalization for cross-template structural reuse.
3. Note style handling:
   - avoid forcing bibliography-centric penalties when bibliography is intentionally absent.

## Phase 5: Governance and Regression Safety

1. Add a CI report check that fails if any core fidelity drops below `1.0`.
   - Completed: `scripts/check-core-quality.js` enforces this as a hard gate.
   - Wired in `.github/workflows/ci.yml`.
2. Add an SQI drift report (warn-level initially) for:
   - large concision regressions
   - preset-usage regressions
   - metric computation failures
   - Completed (warn-level): emitted by `scripts/check-core-quality.js`
     against `scripts/report-data/core-quality-baseline.json`.
3. Record per-wave before/after snapshots in `docs/compat.html` generation notes.
   - Pending follow-up: add explicit wave snapshot notes to compat page output.

## Deliverables

1. Updated core styles with maintained fidelity and improved SQI.
2. Updated `scripts/report-core.js` SQI logic for portfolio comparability.
3. Documented preset extraction map (what was extracted, where reused, by style family).

## Exit Criteria

1. Fidelity remains `1.0` for all core styles.
2. No SQI parse failures.
3. At least 8/11 core styles at SQI `>= 75`.
4. Every core style has a documented rationale for remaining low subscore areas
   (if any are intentionally deferred).
