---
# csl26-ns2i
title: Implement note-start conformance for Chicago and OSCOLA
status: completed
type: feature
priority: medium
created_at: 2026-03-11T17:05:00Z
updated_at: 2026-03-11T17:05:00Z
---

Follow-up to `docs/specs/NOTE_START_REPEATED_NOTE_POLICY.md`.

Implement the settled `note-start` follow-up on top of the existing repeated-
note audit and conformance split.

Definition of done:

- Add the minimum conformance-layer support for `note-start` as an orthogonal
  style-declared rendering dimension, not a new `Position`.
- Start with Chicago and OSCOLA families only.
- Do not widen into `prose-integral`.
- Do not widen into MHRA, New Hart's, Thomson Reuters, or `oscola-no-ibid`.
- Keep regression and conformance layers separate.
- Reuse the existing repeated-note fixture unless a concrete blocker forces a
  change.
- Remove `note-start` from `unresolved` only for `chicago-full-note`,
  `chicago-shortened-note`, and `oscola`.
- Add the minimum tests and report/audit updates required by the spec.
- Keep `short-form.md` out of scope.
- Verify with:
  - `cargo fmt`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo nextest run`
  - `node scripts/audit-note-positions.js --json`
  - `node scripts/report-core.js > /tmp/core-report.json && node scripts/check-core-quality.js --report /tmp/core-report.json --baseline scripts/report-data/core-quality-baseline.json`
  - `./scripts/check-docs-beans-hygiene.sh`
