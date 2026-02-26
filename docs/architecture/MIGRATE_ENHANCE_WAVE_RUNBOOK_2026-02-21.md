# Migrate+Enhance Wave Runbook (2026-02-21)

## Purpose
Single handoff and execution document for the current wave process.
Use this as the canonical source for status, metrics, and next actions.

## Scope
- Branch: `codex/migrate-enhance-wave-strategy`
- Draft PR: <https://github.com/citum/citum-core/pull/208>
- Primary goal: improve `citum-migrate` fidelity/SQI through wave-based style
  conversion, then promote repeated fixes into shared migrate/processor logic.

## Current Results

### Wave 1 (note-heavy, 12 styles)
- Baseline: `619/664` combined (citations `385/408`, bibliography `234/256`)
- Current: `642/664` combined (citations `408/408`, bibliography `234/256`)

### Wave 2 (numeric variants, 12 styles)
- Baseline: `450/528` combined (citations `76/144`, bibliography `374/384`)
- Script-level checkpoint: `514/528` (citations `140/144`)
- Rust/processor checkpoint: `518/528` (citations `144/144`)

Wave 2 citation status is now fully closed (`144/144`).

### Wave 3 (author-date + author/label diversity, 12 styles)
- Baseline: `458/541` combined (citations `114/156`, bibliography `344/385`)
- Rust checkpoint 1: `473/541` combined (citations `129/156`, bibliography `344/385`)
- Rust+merge checkpoint 2: `482/541` combined (citations `138/156`, bibliography `344/385`)
- Dominant citation mismatch clusters:
  - `et-al-with-locator` (4)
  - `suppress-author-with-locator` (3)
  - `et-al-single-long-list` (3)
  - `disambiguate-add-names-et-al` (3)

## Landed Enhancements

### Merge workflow (`scripts`)
- `scripts/merge-migration.js`
  - prevent empty inferred templates from clobbering non-empty base templates
  - numeric citation fallback for explicit empty citation templates
  - numeric locator normalization for AMA-like patterns

### Migration (`citum-migrate`)
- `crates/citum-migrate/src/options_extractor/bibliography.rs`
  - extract legacy bibliography sort into CSLN `GroupSort`
- `crates/citum-migrate/src/main.rs`
  - emit extracted sort into generated `bibliography.sort`
- `crates/citum-migrate/src/options_extractor/tests.rs`
  - coverage for new bibliography sort extraction
- `crates/citum-migrate/src/options_extractor/processing.rs`
  - recursively detect author-date signals through macro trees
  - default extracted disambiguation to legacy-safe values
    (`names=false`, `add-givenname=false`, `year-suffix=true`)
- `crates/citum-migrate/src/main.rs`
  - add author-date citation locator component when legacy layout uses
    `citation-locator` and inferred templates omit it
- `scripts/merge-migration.js`
  - preserve base locator component when inferred citation template lacks
    locator, preventing merge-time regression

### Processor sorting (`csln-processor`)
- `crates/citum-engine/src/grouping/sorting.rs`
  - context-aware author-key fallback behavior
  - author->title fallback only when sort template includes `title`
  - missing-name entries sort after named entries when no title key exists

## Remaining Gaps
- Wave 2 bibliography remains `374/384` (10 unmatched entries).
- Wave 3 citation gap reduced but still open (`129/156`).
- Wave 3 citation gap reduced further (`138/156`) but still open.
- Wave 3 bibliography remains `344/385`, driven by style-specific outliers
  (`springer-physics-author-date` is the largest gap at `10/33`).

## Next Execution Slice
1. Target remaining repeated citation gap:
   `suppress-author-with-locator` (9 styles).
2. Start focused bibliography pass for Wave 3 outliers, beginning with
   `springer-physics-author-date`.
3. Re-check core quality drift:
   - `node scripts/report-core.js > /tmp/core-report.json`
   - `node scripts/check-core-quality.js --report /tmp/core-report.json --baseline scripts/report-data/core-quality-baseline.json`

## Bean Link
Tracked in bean: `csl26-w2n8`.

## Related Docs
- `docs/architecture/MIGRATE_ENHANCE_WAVE_STRATEGY_2026-02-21.md`
- `docs/architecture/MIGRATE_ENHANCE_WAVE1_HANDOFF_2026-02-21.md`
- `docs/architecture/MIGRATE_ENHANCE_WAVE2_HANDOFF_2026-02-21.md`
- `docs/architecture/MIGRATE_ENHANCE_WAVE3_HANDOFF_2026-02-21.md`

## Phase 3/4 Execution Addendum (2026-02-26)

Fresh baseline snapshot (same as plan baseline):
- Styles measured: `139`
- Citations: `2098/2164` (`97.0%`)
- Bibliography: `4053/4255` (`95.3%`)
- SQI overall: `91.5%`
- Threshold attainment:
  - Fidelity `>= 0.95`: `118/139`
  - SQI `>= 0.90`: `107/139`
  - Both: `104/139`

Repeatable cluster extraction commands:
```bash
node scripts/report-core.js > /tmp/core-report-phase34.json
node scripts/analyze-migration-gaps.js \
  --report /tmp/core-report-phase34.json \
  --min-occurrences 2 > /tmp/core-gaps-phase34.json
```

Primary repeated clusters from `/tmp/core-gaps-phase34.json`:
- Citation IDs:
  - `disambiguate-add-names-et-al` (`11`)
  - `suppress-author-with-locator` (`10`)
  - `et-al-with-locator` (`7`)
  - `et-al-single-long-list` (`6`)
  - `with-locator` (`4`)
- Bibliography component diffs:
  - `title:extra` (`31`)
  - `year:missing` (`24`)
  - `publisher:extra` (`22`)
  - `containerTitle:extra` (`13`)
  - `containerTitle:missing` (`12`)
  - `volume:missing` (`9`)

Implementation focus for this slice:
1. Migration engine updates first (`citum-migrate`), specifically:
   - broader locator macro detection for citation template recovery
   - stronger inferred/XML type-template merge guardrails for repeated
     structural bibliography divergences
2. Use rerun metrics after migrate-side updates before any residual
   style-specific YAML patches.
