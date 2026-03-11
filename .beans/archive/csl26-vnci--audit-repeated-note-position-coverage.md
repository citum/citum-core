---
# csl26-vnci
title: Audit repeated-note position coverage
status: completed
type: feature
priority: high
created_at: 2026-03-11T11:44:26Z
updated_at: 2026-03-11T11:57:14Z
---

Add committed repeated-note fixtures, a note-style expectation manifest, verification/report plumbing, and gap reporting for first/subsequent/ibid behavior across note styles.


Spec: `docs/specs/NOTE_POSITION_AUDIT.md`

## Summary of Changes

- Added dedicated repeated-note fixtures at `tests/fixtures/references-note-positions.json` and `tests/fixtures/citations-note-positions.json`.
- Added per-style expectation coverage in `scripts/report-data/note-position-expectations.yaml`.
- Added the repeat-note audit library, CLI, and focused tests.
- Integrated repeated-note audit results into `scripts/report-core.js` and registered the fixtures in verification/report metadata.
- Verified the audit now covers all 19 shipped `processing: note` styles and reports current configuration/rendering gaps explicitly.
