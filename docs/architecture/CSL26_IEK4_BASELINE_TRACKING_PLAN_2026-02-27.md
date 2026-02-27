# CSL26-IEK4 Plan: Baseline Tracking for Regression Detection (2026-02-27)

## Bean
- ID: `csl26-iek4`
- Title: Add baseline tracking for regression detection
- Current status: `todo`
- Dependency listed in bean: `csl26-r6fn` (Testing Infrastructure)

## Evaluation Summary
`csl26-iek4` should remain open and needs addressing.

What is already implemented:
- `scripts/oracle-batch-aggregate.js` supports `--save` and `--compare` and reports regressions.
- `baselines/README.md` documents baseline capture and comparison workflow.
- CI already enforces `scripts/check-core-quality.js` against `scripts/report-data/core-quality-baseline.json`.

What is still missing relative to bean intent:
- No committed oracle baseline artifact used as a canonical CI regression reference.
- No CI gate that compares current oracle results against a pinned oracle baseline and fails on regression.
- No explicit policy for baseline scope/versioning tied to fixtures/style set changes.

## Goal
Create a deterministic, CI-enforced oracle baseline gate that detects rendering regressions automatically for the project’s priority style set.

## Scope
In scope:
- Oracle baseline artifact format and storage location.
- CI regression gate for oracle-based style fidelity.
- Maintenance workflow for baseline refreshes with review visibility.

Out of scope:
- Expanding citation fixture coverage beyond the current strict 12-scenario set.
- Replacing `check-core-quality.js`; this remains a complementary gate.

## Implementation Plan
1. Define canonical baseline target
- Baseline set: top-priority parent styles currently used for strict tracking (start with top 10 in `docs/TIER_STATUS.md`).
- Fixture set: strict 12-scenario fixture (`tests/fixtures/citations-expanded.json`) as canonical input.
- Determinism: fix sort/stable output order in saved JSON if needed.

2. Add committed oracle baseline artifact
- Add a tracked file under `scripts/report-data/`, for example:
  - `scripts/report-data/oracle-top10-baseline.json`
- Generate it via:
  - `node scripts/oracle-batch-aggregate.js styles-legacy/ --top 10 --json > scripts/report-data/oracle-top10-baseline.json`
- Include metadata fields for baseline provenance:
  - date/time, fixture path, style count, script version hash (or commit SHA).

3. Add CI regression check for oracle baseline
- Add a small checker script (for example `scripts/check-oracle-regression.js`) that:
  - loads baseline JSON,
  - runs current oracle aggregate for same style set,
  - fails on any citation or bibliography pass-count regression per style,
  - prints clear per-style deltas.
- Wire script into `.github/workflows/ci.yml` after existing Rust/tests and core-quality checks.

4. Add local workflow commands and docs
- Update `docs/TIER_STATUS.md` and/or `docs/guides/RENDERING_WORKFLOW.md` with:
  - gate command,
  - baseline refresh command,
  - policy: refresh baseline only in dedicated PRs with justification.

5. Add maintenance guardrails
- Require PR note when updating baseline artifact:
  - reason for refresh,
  - before/after summary,
  - explicit statement whether changes are expected improvements or accepted deltas.

## Acceptance Criteria
- CI fails when any tracked top-style citation/bibliography score drops below baseline.
- CI passes when scores are equal or improved.
- Baseline file is committed, reproducible, and documented.
- Developer docs include a baseline refresh protocol and rationale requirements.

## Risks and Mitigations
- Risk: baseline churn from fixture/style-set changes.
  - Mitigation: pin fixture/style-set in metadata and require explicit baseline-refresh PR.
- Risk: flaky/non-deterministic oracle output.
  - Mitigation: enforce stable ordering and deterministic JSON serialization.
- Risk: duplicated gates with core-quality check.
  - Mitigation: keep responsibilities separate (portfolio SQI/concision vs per-style oracle fidelity).

## Suggested Execution Order
1. Implement checker script + deterministic output contract.
2. Generate and commit initial oracle baseline file.
3. Wire CI gate.
4. Document refresh policy and commands.
5. Run full validation and open follow-up bean updates (`csl26-r6fn` linkage) if needed.
