---
# csl26-nts1
title: Spec note-start repeated-note policy
status: completed
type: feature
priority: medium
created_at: 2026-03-11T00:00:00Z
updated_at: 2026-03-11T16:30:00Z
---

Follow-up to `docs/specs/NOTE_SHORTENING_POLICY.md`.

We now have a settled repeated-note family model for lexical markers and
shortened-note fallback, and this follow-up settles the narrow `note-start`
question for Chicago and OSCOLA families only.

Definition of done:

- [x] Draft a narrow spec under `docs/specs/` for note-start repeated-note
  policy.
- [x] Limit scope to `note-start` repeated-cite semantics, not prose/integral
  rules.
- [x] Start with Chicago and OSCOLA families before widening to MHRA, New
  Hart's, and legal-note families.
- [x] Settle `note-start` as a style-declared rendering dimension rather than a
  processor-managed position state.
- [x] Specify the minimum future fixture/audit follow-up: keep regression and
  conformance separate, reuse the existing repeated-note fixture, and remove
  `note-start` from `unresolved` only for Chicago and OSCOLA conformance
  families once that audit field exists.
- [x] Keep paraphrases only; do not commit copyrighted source excerpts.
