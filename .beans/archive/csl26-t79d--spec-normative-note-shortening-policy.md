---
# csl26-t79d
title: Spec normative note shortening policy
status: completed
type: feature
priority: medium
created_at: 2026-03-11T00:00:00Z
updated_at: 2026-03-11T00:00:00Z
---

Follow-up to the repeated-note rollout PR. The implementation contract now
lives in `docs/specs/NOTE_SHORTENING_POLICY.md`.

Execution targets:

- keep the existing repeated-note audit as the shipped-regression gate
- add a separate normative-conformance layer for settled note-style families
- separate processor invariants from style-declared behavior
- classify Chicago, MHRA, New Hart's, OSCOLA, OSCOLA-no-ibid, Thomson Reuters
  legal, and shipped shortened-note variants without quoting manuals
- tighten tests only for settled repeated-note invariants and settled family
  expectations
- keep note-start and prose/integral repeated-cite semantics unresolved unless
  supported by strong repo-backed evidence
