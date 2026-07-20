---
# csl26-umay
title: Wire calendar-date note into real GB/T corpus + fidelity
status: in-progress
type: task
priority: high
tags:
    - fidelity
    - gb-t
    - dates
    - rendering
created_at: 2026-07-20T17:31:47Z
updated_at: 2026-07-20T17:58:03Z
blocked_by:
    - csl26-epu6
---

Follow-up to PR 1068 review feedback: roll out note-wrap to gb-t-7714-2025-note/numeric styles, add a Rust regression test against the real pinned tests/fixtures/test-items-library/gb-t-7714-2025.json corpus (not synthetic data), empirically check gb-t-7714-2025-numeric's fidelity gate, register div-015 in DIVERGENCE_REGISTER.md, update spec acceptance criteria, and add a README pre-1.0 stability disclaimer.

## Todo

- [x] Step 1: add bibliography.options.dates + note-wrap to gb-t-7714-2025-note.yaml
- [x] Step 1: add bibliography.options.dates + note-wrap to gb-t-7714-2025-numeric.yaml
- [x] Step 2: empirical fidelity check done — gb-t-7714-2025-numeric is NOT in core-quality-baseline.json's gating style list (check-core-quality.js only hard-fails styles listed there), so no div-015 masking JS needed. Feature covers 5 real records: 7.5.4.1:1/:2, 8.2.2:2, 8.12.3:1, 8.12.3:3 (found via the real render + fidelity report, not just the 2 originally scoped).
- [x] Step 3: Rust regression test against the real pinned gb-t-7714-2025.json fixture — 9 rstest cases (5 records x applicable styles), all pass
- [ ] Step 4: register div-015 in DIVERGENCE_REGISTER.md
- [x] Step 5: filed csl26-svfg — style-inheritance whole-block-replace merge limitation for nested option blocks (general, not GB/T-scoped)
- [x] Step 6: update CALENDAR_DATE_ANNOTATIONS.md acceptance criteria + changelog v1.6
- [x] Step 7: README pre-1.0 stability disclaimer
- [x] Verify: just pre-commit green (fmt/clippy/2103 tests). render-refs demo evidence captured for all 3 styles x 5 records.
