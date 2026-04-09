---
# csl26-qh84
title: Expand Chicago and APA fidelity fixtures
status: completed
type: task
priority: normal
created_at: 2026-04-09T12:10:48Z
updated_at: 2026-04-09T13:15:00Z
---

Follow up `csl26-ccyy` by expanding the fidelity inputs that stress Chicago 18
and APA behavior more directly. That work is intentionally out of scope for the
schema/conversion PR: this bean is for richer fixture coverage and any
style-level adjustments that fall out of that broader verification pass.

2026-04-09 ownership note: this bean owns the Chicago/APA rich-fixture and
reporting expansion work, including benchmark configuration, committed reduced
fixtures when needed for bounded debugging, and any minimal style-local fixes
required to keep the new supplemental evidence usable. Residual Chicago
style-only follow-up remains on `csl26-tpmn`.

## Context

`csl26-ccyy` closed schema-data and coverage-analysis gaps against the Zotero
test-item libraries, but it did not add or migrate new style fixtures. If we
want stronger confidence that Chicago 18 and APA are exercised by fidelity
reporting, we need dedicated richer inputs and, potentially, follow-on style
work driven by those results.

Related:
- `docs/specs/CHICAGO_18_COVERAGE.md`
- `.beans/csl26-x1y2--hook-up-generalized-relational-fixtures-to-fidelit.md`

## Tasks

- [x] Define the target Chicago 18 / APA scenarios that are still under-stressed by the current fidelity inputs.
- [x] Add or derive richer fixtures that exercise those scenarios in the fidelity pipeline.
- [x] Run the affected styles against the expanded fixture set and capture the resulting gaps.
- [x] Split any discovered style-authoring or engine follow-up work into separate actionable beans if needed.

## Summary of Changes

Aligned ownership between `csl26-qh84` and `csl26-tpmn` so this bean owns the
Chicago/APA rich-fixture and reporting expansion while `csl26-tpmn` remains the
residual Chicago follow-up tracker.

Added official APA supplemental benchmark reporting through
`scripts/report-data/verification-policy.yaml`, registered the benchmark
fixtures in `tests/fixtures/coverage-manifest.json`, refreshed
`docs/compat.html`, and added regression coverage asserting that the APA
benchmark runs are exposed by the repo policy.

Co-evolved styles and schema/engine behavior beyond the original fixture-only
scope: APA rich bibliography work now preserves the primary `1.0` APA gate
while improving the focused broadcast / motion-picture / interview /
entry-reference cluster to `22 / 24`, and Chicago author-date now applies the
dictionary-specific path correctly while improving the style score to `0.779`.

Kept the APA Zotero benchmark diagnostic-only rather than fidelity-scoring
because the repo's hard core-quality gate requires `apa-7th` to remain at
headline fidelity `1.0`; shipping it as a scoring run would still break the
required gate. Captured the remaining APA rich-bibliography residuals in
`csl26-xgv3`, and kept bounded Chicago residual cleanup on `csl26-tpmn`.
