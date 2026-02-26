# Migrate+Enhance Phase 3/4 Single-PR Plan (2026-02-26)

## Scope Decision

This plan is explicitly aligned with CSLN project principles:

1. Improve `citum-migrate` so future migrations are fully automated.
2. Raise existing `styles/` toward `>= 0.95` fidelity and `>= 0.90` SQI.
3. Do not add direct CSL 1.0 support to the processor as a project goal.

### Out of Scope

1. No runtime CSL 1.0 (`.csl`) processing mode in `citum_engine`.
2. No processor-side legacy condition engine.
3. No architectural compromises to CSLN declarative design.

## Current Baseline (Fresh Run)

Source: `node scripts/report-core.js` run at commit `136ecbb` on 2026-02-26 (`/tmp/core-report-2.json`).

1. Styles measured: `139`
2. Citations: `2098/2164` (`97.0%`)
3. Bibliography: `4053/4255` (`95.3%`)
4. SQI overall: `91.5%`
5. Threshold attainment:
   - Fidelity `>= 0.95`: `118/139`
   - SQI `>= 0.90`: `107/139`
   - Both: `104/139`

## Guiding Constraints (Hard)

1. Start each target batch from fresh rerun baseline and capture before metrics.
2. Work by style family and close reusable pattern gaps first.
3. Prefer `citum-migrate` rule/preset improvements over style-specific YAML edits.
4. Use per-style patches only when safe generalization is not possible.
5. No fidelity regressions allowed.

## Main Gap Clusters

### Citation clusters (repeated)

1. `disambiguate-add-names-et-al`
2. `et-al-with-locator`
3. `suppress-author-with-locator`
4. `et-al-single-long-list`
5. `with-locator`

### Bibliography component diffs (repeated)

1. `title:extra`
2. `containerTitle:missing`
3. `publisher:extra`
4. `volume:missing`
5. `containerTitle:extra`

## Integrated Single-PR Plan

## Phase 1: Baseline and Pattern Extraction

1. Freeze target set by family:
   - all styles below `0.95` fidelity
   - near-threshold high-impact styles below `0.90` SQI
2. Capture baseline triplet per style:
   - seeded baseline
   - edited output
   - fresh rerun output from `citum-migrate`
3. Build cluster frequency tables (citation IDs + bibliography component diffs).
4. Promote only repeated patterns (2+ styles) to engine-level fixes.

Likely files/modules:

1. `scripts/prep-migration.sh`
2. `scripts/oracle.js`
3. `scripts/oracle-batch-aggregate.js`
4. `docs/architecture/MIGRATE_ENHANCE_WAVE_RUNBOOK_2026-02-21.md` (update section)

## Phase 2: Reusable Engine Enhancements

1. Add migrate-side citation rule pack for locator/et-al/suppress-author patterns.
2. Add migrate-side bibliography normalization for repeated container/title/volume issues.
3. Improve migrate template inference merge behavior to reduce underfit/overfit edge cases.
4. Tighten options/preset extraction where repeated SQI losses are due to missing presetization.
5. Add targeted processor fixes only when required for CSLN fidelity parity, not for CSL 1.0 feature support.

Processor rule in this PR:

1. Allowed: parity-preserving bug fixes for existing CSLN model behavior.
2. Not allowed: new direct legacy-style support surfaces.

Likely files/modules:

1. `crates/citum-migrate/src/main.rs`
2. `crates/citum-migrate/src/upsampler.rs`
3. `crates/citum-migrate/src/options_extractor/processing.rs`
4. `crates/citum-migrate/src/options_extractor/bibliography.rs`
5. `crates/citum-migrate/src/passes/*`
6. `crates/citum-migrate/src/template_resolver.rs`
7. `scripts/merge-migration.js`
8. `scripts/lib/template-inferrer.js`
9. `crates/citum-engine/src/processor/mod.rs` (only if needed for parity)

## Phase 3: Portfolio Lift and Stabilization

1. Re-run family batches (author-date, numeric, note) after shared fixes.
2. Apply minimal per-style patches only for non-generalizable residuals.
3. Run SQI uplift pass (preset usage and safe concision improvements) only after fidelity is stable.
4. Enforce no-regression gate across target styles.
5. Publish before/after/rerun tables and family-level outcomes in docs.

Likely files/modules:

1. `styles/*.yaml` (small residual deltas only)
2. `scripts/report-core.js`
3. `scripts/check-core-quality.js`
4. `docs/TIER_STATUS.md`
5. `docs/architecture/SQI_REFINEMENT_PLAN.md`

## Metrics Mapping

1. Citation rule pack should raise author-date failures by collapsing repeated citation mismatch IDs.
2. Bibliography rule pack should materially lift low outliers:
   - `springer-physics-author-date`
   - `royal-society-of-chemistry`
   - `gost-r-7-0-5-2008-author-date`
3. SQI preset extraction should lift many `q < 0.90` styles where preset usage is the main penalty.
4. Family expectation:
   - Author-date: largest total gain.
   - Numeric: mostly outlier closure.
   - Note: close remaining MHRA outliers while preserving current strong pass rates.

## Acceptance Criteria (PR Gate)

1. No fidelity regressions in any previously passing target style.
2. Net increase in styles meeting both thresholds (`>= 0.95` fidelity, `>= 0.90` SQI).
3. Every non-generalized style patch includes rationale.
4. Clear evidence that improvements come from reusable migration logic first.
5. No new direct CSL 1.0 processor support commitments or APIs.

