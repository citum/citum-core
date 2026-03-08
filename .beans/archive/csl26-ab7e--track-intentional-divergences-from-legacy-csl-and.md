---
# csl26-ab7e
title: Track intentional divergences from legacy CSL and citeproc
status: completed
type: task
priority: normal
created_at: 2026-03-08T00:33:37Z
updated_at: 2026-03-08T01:59:39Z
---

Some recent style improvements intentionally prefer publisher rules, biblatex
prior art, or Citum design principles over legacy CSL/citeproc quirks. Those
decisions are valid, but they need a durable record so future compatibility
triage does not "fix" them back toward legacy behavior.

Create a lightweight divergence register that notes the affected style, the
behavioral difference, the authority basis that won, and any expected impact on
oracle or compatibility reporting. Start with the divergences surfaced by the
recent numeric-compound and alphabetical sorting work, then extend the pattern
to future style-wave edits.

## Summary of Changes

Created `docs/adjudication/DIVERGENCE_REGISTER.md` with five initial divergence
entries covering compound numeric sets, biblatex-primary chemistry styles,
explicit default bibliography sorting by processing family, missing-name title
fallback, and citation-list explicit-only policy. Added extension protocol for
future style-wave authors. Added reference to register in architecture README.
