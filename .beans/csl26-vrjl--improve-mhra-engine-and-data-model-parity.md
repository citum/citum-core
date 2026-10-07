---
# csl26-vrjl
title: Improve MHRA engine and data-model parity
status: in-progress
type: task
priority: high
tags:
    - engine
    - schema
    - fidelity
    - dates
created_at: 2026-10-07T11:35:33Z
updated_at: 2026-10-08T12:44:35Z
parent: csl26-83iw
---

Investigate and implement general engine/data-model behavior behind MHRA exact-parity residuals after PR #1288. Cover structured issued/accessed dates, repeated-author substitution, contributor-role and genre semantics, and legal/archive conversion and rendering. Add focused regression tests before any MHRA YAML changes; avoid MHRA-specific engine strings; preserve citation fidelity at 33/34, bibliography fidelity at 43/46, and the measured SQI floor of 0.897 while improving exact parity from 27/79. Keep the parent bean csl26-83iw in-progress until the engine and any stacked style follow-up are complete.

## Checklist

- [x] Capture and classify remaining MHRA exact-parity residuals
- [x] Add focused engine/data-model regression tests
- [x] Implement general structured-date and substitution fixes
- [x] Investigate genre, contributor-role, legal, and archive mappings
- [x] Run schema generation and just pre-commit (2,828 tests passed)
- [x] Run the final MHRA all-features report and regression analysis (44/79 exact; citation 33/34, bibliography 43/46, SQI 0.897)
- [x] Open draft engine PR #1290 with metrics and limitations
- [x] Create stacked draft style PR #1291 for date forms, raw genre, contributor roles, and style-facing parity



## PR #1290 review resolution

Replaced partial-first post-render substring mutation with render-time contributor substitution. Added exact regressions for unrelated leading text collisions, select-first/render-when winner selection, and HTML escaping. Focused bibliography suite: 121/121 passed. Required Rust gate: 2,831/2,831 passed after clean formatting and clippy. Stacked MHRA report: citations 33/34, bibliography 43/46, exact parity 44/79, SQI 0.897. Diagnostic MHRA Zotero benchmark remains non-gating at 52/127. PRs #1290 and #1291 remain draft; review threads pending CI evidence.
