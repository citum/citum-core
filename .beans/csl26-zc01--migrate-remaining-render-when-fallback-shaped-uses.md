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
updated_at: 2026-09-10T12:12:30Z
parent: csl26-ccdt
---

docs/architecture/audits/2026-09-06_RENDER_WHEN_DISPOSITION.md inventoried 49 'Fallback'-shaped render-when uses across the embedded style corpus (of 125 total; 123 in the Chicago family, 2 in gb-t-7714-2025-base) that are structurally expressible as select:first (docs/specs/GROUP_SELECT.md, shipped in csl26-tzs3/PR #1270). This is a full-corpus follow-on: after PR #1270/#1272 shipped the primitive and migrated the T&F-CSE publisher-place worked example (7 rows), a second stacked PR migrates ONE additional example style as a proof/demo (see that PR for the specific candidate and diff). The remaining ~48 fallback-shaped uses (nearly all Chicago-family) are NOT a mechanical find-and-replace: the audit found 108 of 125 uses are a MIX of fallback and policy-gate shapes per field, and at least one 'obvious' candidate (Chicago's volume-title, chicago-author-date-18th.yaml:416-425) turned out to have a hidden third interacting condition (part-number-non-numeric) that select:first cannot safely express (verified by hand-tracing its truth table -- one combination must render nothing, which select:first structurally can't do, since it always falls through to the next candidate). Each candidate needs the same careful truth-table verification before migrating, style by style. Policy-gate uses (25) and else-branches pairing with them are explicitly out of scope -- those need work-form routing design (csl26-zmxt), not select:first.

## Update (2026-09-10)

This PR ended up demonstrating two distinct outcomes, not one:

1. `chicago-notes-18th.yaml`'s broadcast identifier: a genuine `select: first` migration (the shared-tail trap, documented above already applies here too -- both original branches rendered `number`).
2. `chicago-shortened-notes-bibliography-core.yaml`'s author+title citation gate (`render-when: field-present/absent: author`): closes `csl26-x79y`. This one needed **no primitive at all** -- the field-absent branch's content (bare title) is a strict subset of the field-present branch's content (contributor + title), so deleting the render-when pair and merging into one unconditional group is correct on its own: the group's existing emptiness/delimiter-join semantics already degrade to bare title when `contributor: author`'s own substitution (editor, then translator) also comes up empty. This is a THIRD trap/non-trap category worth checking for on every remaining candidate before reaching for select:first: **is the absent-branch content already a strict subset of the present-branch content?** If so, the render-when pair can just be deleted, no select:first needed. Verified as a real bug fix (not just a refactor): chicago-shortened-notes-bibliography's exact-parity count moved 92->93/473 (chicago-18-base family aggregate 560->561/1629), zero regressions in the 35-style full corpus.

**New, NOT yet verified candidate found while investigating:** `chicago-author-date-18th.yaml:876-877,911-912` has a second `field-present/absent: author` pair, in the `interview:` type-variant. This looked superficially like the same bug but is NOT a strict-subset case -- the field-absent branch drops several fields the field-present branch has (archive-location, archive-collection, references, publisher, parent-monograph, event-place, raw-medium/dimensions detail) rather than being a subset of it. Needs its own truth-table verification before assuming either the select:first fix or the plain-merge fix applies; do not treat it as already covered by this bean's two examples above.
