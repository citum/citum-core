---
# csl26-6bul
title: 'migrate: audit fixture coverage for measured selection'
status: completed
type: task
priority: high
tags:
    - migrate
    - fixtures
    - fidelity
created_at: 2026-06-17T18:22:49Z
updated_at: 2026-06-18T00:00:00Z
---

Output-driven template selection and synthesis are explicitly coverage-bound: the selector can only prefer measured evidence for behavior exercised by the selection fixtures, and the held-out gate only rejects regressions visible in held-out fixtures. This makes fixture sufficiency a first-class migration quality risk, not just a testing nicety.

## Scope

- Produce a fixture-coverage audit for `citum-migrate` measured citation/bibliography selection and synthesis.
- Compare `tests/fixtures/references-expanded.json` and `tests/fixtures/references-heldout.json` against `tests/fixtures/coverage-manifest.json` and `scripts/report-data/fixture-sufficiency.yaml`.
- Cross-check fixture coverage against CSL type-conditioned branches and seed-winner/debug evidence, especially cases where XML fallback or XML seed still wins in the random style scorecard.
- Identify which gaps are selection-set gaps, held-out-set gaps, or latent XML-branch gaps that should remain XML-backed until exercised.
- Add the highest-value fixture items or document a ranked deferred list when fixture construction needs domain input.
- Keep selection and held-out examples disjoint so held-out validation remains a real over-fitting guard.

## Initial risk targets

- Rare or structurally distinctive item types: legal cases, legislation, patents, datasets, standards, encyclopedia/dictionary entries, newspaper/magazine articles, broadcasts, interviews, maps, theses, reports, and web pages.
- Behavior families that often drive ugly or wrong migrated structure: URL/accessed gating, DOI suppression, contributor role fallback, editor/name-order handling, title casing, volume/pages delimiters, issued/accessed/date-parts, locale terms, and type-specific bibliography flattening.
- Positional citation behavior for first, first-with-locator, subsequent, ibid, and ibid-with-locator scenarios when a style's branch behavior differs by position.

## Acceptance Criteria

- A generated or documented coverage report states which reference types and behavior families are covered by selection fixtures, held-out fixtures, both, or neither.
- The report calls out any measured-selection wins that are weak because the fixture surface is too narrow, and any XML seed wins that likely reflect missing fixture evidence rather than true converter superiority.
- High-value fixture additions land in `references-expanded` and/or `references-heldout`, or a ranked follow-up list explains why each item needs curator/domain input.
- `scripts/check-testing-infra.js` and the existing fixture/frontmatter checks pass after fixture changes.
- Targeted migration scorecard/oracle runs show no fidelity regression; improvements are accepted only when both selection and held-out evidence support them.
- `docs/architecture/MIGRATION_STRATEGY_ANALYSIS.md` or the output-driven synthesis spec links to the new coverage report once it exists.

## References

- `docs/architecture/MIGRATION_STRATEGY_ANALYSIS.md`
- `docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md`
- `tests/fixtures/references-expanded.json`
- `tests/fixtures/references-heldout.json`
- `tests/fixtures/coverage-manifest.json`
- `scripts/report-data/fixture-sufficiency.yaml`
- `scripts/check-testing-infra.js`
- `csl26-aynr` output-driven template synthesis
- `csl26-hxhx` XML compiler removal blocker

## Summary of Changes

- Added machine-readable measured-selection summaries to `citum-migrate --emit-evidence`.
- Added `scripts/audit-measured-fixture-coverage.js` for repeatable selection/held-out coverage audits.
- Added `scripts/sync-measured-selection-fixtures.js` and an allowlist to import curated CSL JSON test items into the measured-selection fixtures.
- Registered `references-heldout.json` in `coverage-manifest.json`.
- Documented `fixture-sufficiency.yaml` as a hand-maintained policy file validated by testing-infra checks.
- Added `docs/architecture/audits/2026-06-18_MEASURED_SELECTION_FIXTURE_COVERAGE.md` and linked it from the migration strategy analysis.

## Verification

- `cargo fmt`
- `cargo test -p citum-migrate --quiet`
- `cargo test -p citum-migrate js_runtime::tests::heldout_fixture_items_are_disjoint_from_selection_items --quiet`
- `node scripts/check-testing-infra.js`
- `node scripts/sync-measured-selection-fixtures.js`
- `node scripts/audit-measured-fixture-coverage.js --json`
- `cargo run --quiet --bin citum-migrate -- styles-legacy/apa.csl --emit-evidence /tmp/citum-apa-evidence.json`
- `node scripts/report-migrate-sqi.js --styles apa,ieee,chicago-notes-bibliography --out /tmp/csl26-6bul-scorecard-expanded.json`
