---
# csl26-qh84
title: Expand Chicago and APA fidelity fixtures
status: todo
type: task
priority: normal
created_at: 2026-04-09T12:10:48Z
updated_at: 2026-04-09T12:10:48Z
---

Follow up `csl26-ccyy` by expanding the fidelity inputs that stress Chicago 18
and APA behavior more directly. That work is intentionally out of scope for the
schema/conversion PR: this bean is for richer fixture coverage and any
style-level adjustments that fall out of that broader verification pass.

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

- [ ] Define the target Chicago 18 / APA scenarios that are still under-stressed by the current fidelity inputs.
- [ ] Add or derive richer fixtures that exercise those scenarios in the fidelity pipeline.
- [ ] Run the affected styles against the expanded fixture set and capture the resulting gaps.
- [ ] Split any discovered style-authoring or engine follow-up work into separate actionable beans if needed.
