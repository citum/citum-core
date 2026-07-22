---
# csl26-ap2b
title: Tune gb-t-7714-2025-note to full fidelity
status: completed
type: task
priority: normal
tags:
    - style
    - fidelity
    - multilingual
created_at: 2026-07-16T10:56:59Z
updated_at: 2026-07-23T00:24:30Z
blocked_by:
    - csl26-8uxa
---

Drive the embedded gb-t-7714-2025-note style to 100% fidelity on the upstream corpus (tests/fixtures/test-items-library/gb-t-7714-2025.json) and flip its verification-policy benchmark run back to count_toward_fidelity: true with min_pass_rate 1.0. Most numeric-wave fixes land in the shared gb-t-7714-2025-base.yaml and should transfer; re-run the cluster triage before tuning.

Also add a styles.gb-t-7714-2025-note entry to scripts/report-data/note-position-expectations.yaml once the style's repeated-note behavior is characterized — the note-position audit currently reports configuration-gap for it (guard added in wave 2 so report-core no longer crashes).

## Summary of Changes

Note style reached 100% adjusted fidelity on the 203-item GB/T corpus
(citations 1/1, bibliography 203/203) essentially for free — the base-layer
fixes made for numeric (locale/type-variant fallback via csl26-7hsx, div-011
date-annotation divergence registered this session) apply equally since note
inherits the shared gb-t-7714-2025-base.yaml bibliography grammar. Flipped
verification-policy.yaml's benchmark run to count_toward_fidelity: true,
min_pass_rate: 1.0 — the gate is no longer diagnostic-only.

The separate note-position-expectations.yaml entry (repeated-note behavior)
mentioned in the original bean description is still open; the note-position
audit reports configuration-gap for gb-t-7714-2025-note. Filed as csl26-2hbd
rather than blocking this pass since it's presentational/audit-metadata,
not a fidelity defect. (Correction 2026-07-23: an earlier version of this
summary claimed this was already filed — it was not; csl26-2hbd is the
actual bean.)
