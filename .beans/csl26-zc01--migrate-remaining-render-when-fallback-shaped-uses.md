---
# csl26-zc01
title: 'Migrate remaining render-when fallback-shaped uses to select: first (corpus sweep)'
status: todo
type: task
priority: normal
tags:
    - style
    - fidelity
    - render-when
created_at: 2026-09-10T11:16:04Z
updated_at: 2026-09-10T11:16:04Z
parent: csl26-ccdt
---

docs/architecture/audits/2026-09-06_RENDER_WHEN_DISPOSITION.md inventoried 49 'Fallback'-shaped render-when uses across the embedded style corpus (of 125 total; 123 in the Chicago family, 2 in gb-t-7714-2025-base) that are structurally expressible as select:first (docs/specs/GROUP_SELECT.md, shipped in csl26-tzs3/PR #1270). This is a full-corpus follow-on: after PR #1270/#1272 shipped the primitive and migrated the T&F-CSE publisher-place worked example (7 rows), a second stacked PR migrates ONE additional example style as a proof/demo (see that PR for the specific candidate and diff). The remaining ~48 fallback-shaped uses (nearly all Chicago-family) are NOT a mechanical find-and-replace: the audit found 108 of 125 uses are a MIX of fallback and policy-gate shapes per field, and at least one 'obvious' candidate (Chicago's volume-title, chicago-author-date-18th.yaml:416-425) turned out to have a hidden third interacting condition (part-number-non-numeric) that select:first cannot safely express (verified by hand-tracing its truth table -- one combination must render nothing, which select:first structurally can't do, since it always falls through to the next candidate). Each candidate needs the same careful truth-table verification before migrating, style by style. Policy-gate uses (25) and else-branches pairing with them are explicitly out of scope -- those need work-form routing design (csl26-zmxt), not select:first.
