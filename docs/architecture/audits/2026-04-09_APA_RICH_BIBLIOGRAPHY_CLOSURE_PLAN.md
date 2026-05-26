# APA Rich Bibliography Closure Plan

**Date:** 2026-04-09
**Status:** Active
**Related:** `.beans/archive/csl26-xgv3--apa-7-rich-bibliography-follow-up.md`, `.beans/archive/csl26-zu8r--apa-web-native-packaging-follow-up.md`, `.beans/csl26-gtat--apa-container-packaging-follow-up.md`, `.beans/archive/csl26-5ap9--apa-authored-containerized-works-follow-up.md`, `.beans/csl26-mwnt--apa-year-suffix-disambiguation-cleanup.md`, `docs/specs/FIDELITY_RICH_INPUTS.md`

## Purpose
Drive `apa-test-library-diagnostic` from the current `54 / 74` to conclusion
without regressing the baseline APA `40 / 40` gate.

This plan treats the remaining APA supplemental rows as concrete
style / processor / migration work, not as an acceptable fuzzy residual.

## Current Verified State
- baseline APA gate remains `40 / 40`
- supplemental APA diagnostic benchmark is `54 / 74`
- the earlier dataset processor gap from `csl26-xgv3` is closed
- the web-native cluster (`csl26-zu8r`) is complete and moved the benchmark to
  `44 / 74`
- the first structural chapter / entry / proceedings pass in `csl26-5ap9`
  moved a reduced cluster from `0 / 10` to `9 / 10` and lifted the full
  benchmark to `54 / 74`
- `docs/compat.html` still shows an older published snapshot and is not the
  source of truth for current branch progress

## Success Definition
The APA rich-bibliography effort is concluded only when all three are true:
- `node scripts/report-core.js --style apa-7th` reports
  `apa-test-library-diagnostic` at `74 / 74`
- baseline APA remains `40 / 40`
- any non-fixable rows are explicitly recorded as intentional divergence or
  malformed-source exclusions in repo-tracked docs and policy, leaving zero
  unknown residuals

## Active Workstreams
### 1. Web-native packaging
- bean: `csl26-zu8r`
- rows: `42`, `43`, `45`
- status: completed
- before count: `3` failing rows
- after count: `0` failing rows
- expected subsystem: primarily `citum_engine`
- target issues: retrieval-date fallback, website title packaging, webpage
  part-title handling, inline editor / translator packaging

### 2. Container packaging
- bean: `csl26-gtat`
- rows: `44`, `46`, `47`, `48`, `56`, `59`
- before count: `6` failing rows
- expected subsystem: primarily `citum_migrate` / schema-data conversion, then
  `citum_engine`
- target issues: translator / editor / edition / volume / report-number
  preservation, chapter-in-report packaging, technical-report flattening,
  magazine packaging

### 3. Authored / containerized works
- bean: `csl26-5ap9`
- rows: `49` to `58`, `60` to `74`
- status: structural closure complete on 2026-04-10
- before count: `20` failing rows
- reduced structural closure fixture for rows `71`, `73`, and `74`: `3 / 3`
- remaining follow-up: any residual year/date ordering cleanup is handed to
  `csl26-mwnt`
- expected subsystem: mixed `citum_migrate`, `citum_engine`, and style YAML
- target issues:
  - chapter container-author suppression via generic `role-substitute`
  - conference / presentation classification and session packaging
  - preprint/report-like article intake and parenthetical packaging

### 4. Year-suffix / disambiguation cleanup
- bean: `csl26-mwnt`
- rows: structural handoff now includes residual ordering cleanup after rows
  `71`, `73`, and `74` were closed structurally
- before count: not yet isolated as a clean post-structure-only set
- expected subsystem: primarily `citum_engine`
- target issues: residual year-letter churn and anonymous-ordering mismatches
  that remain after structural fixes land

## Operating Rules
- Do not reopen the dataset fix unless a later pass proves regression.
- Do not attack year-suffix logic first; structural fixes must land before
  disambiguation cleanup is isolated.
- Use one reduced fixture per cluster before editing and confirm on the full
  APA benchmark after each pass.
- If a cluster spans more than one subsystem or grows beyond 3 rows, split it
  into a narrower successor bean with an explicit handoff.
- Stop after 2 distinct implementation attempts with no net gain on a cluster;
  reclassify immediately rather than continue speculative edits.

## Required Verification Per Pass
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run`
- `./scripts/check-docs-beans-hygiene.sh`
- `node scripts/report-core.js --style apa-7th`

Every pass must also:
- run a reduced-cluster oracle fixture before and after editing
- confirm the full supplemental APA benchmark after the reduced pass succeeds
- reject any change that regresses the baseline APA gate or introduces new
  off-cluster failures
