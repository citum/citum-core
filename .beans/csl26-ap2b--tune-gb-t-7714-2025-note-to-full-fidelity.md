---
# csl26-ap2b
title: Tune gb-t-7714-2025-note to full fidelity
status: todo
type: task
priority: normal
tags:
    - style
    - fidelity
    - multilingual
created_at: 2026-07-16T10:56:59Z
updated_at: 2026-07-16T16:32:37Z
blocked_by:
    - csl26-8uxa
---

Drive the embedded gb-t-7714-2025-note style to 100% fidelity on the upstream corpus (tests/fixtures/test-items-library/gb-t-7714-2025.json) and flip its verification-policy benchmark run back to count_toward_fidelity: true with min_pass_rate 1.0. Most numeric-wave fixes land in the shared gb-t-7714-2025-base.yaml and should transfer; re-run the cluster triage before tuning.

Also add a styles.gb-t-7714-2025-note entry to scripts/report-data/note-position-expectations.yaml once the style's repeated-note behavior is characterized — the note-position audit currently reports configuration-gap for it (guard added in wave 2 so report-core no longer crashes).
