---
# csl26-2hbd
title: Characterize gb-t-7714-2025-note repeated-note behavior for note-position-expectations.yaml
status: todo
type: task
priority: normal
tags:
    - style
    - fidelity
    - gb-t
    - reporting
created_at: 2026-07-23T00:24:24Z
updated_at: 2026-07-23T00:24:24Z
---

The note-position audit (scripts/report-data/note-position-expectations.yaml)
reports configuration-gap for gb-t-7714-2025-note — it has no entry in that
file, so the audit can't assess its repeated-note (ibid/short-form) behavior.
A guard was added in an earlier wave so report-core no longer crashes on the
missing entry, but the entry itself was never authored.

Surfaced while tuning gb-t-7714-2025-note to full fidelity (csl26-ap2b,
completed 2026-07-22); deferred there as presentational/audit-metadata, not
a fidelity defect, since it doesn't affect the 203-item corpus adjusted
score. Filing for real now — csl26-ap2b's own summary claimed this was
"filed as follow-up scope" without an actual bean existing, which was
wrong.

- [ ] Characterize gb-t-7714-2025-note's repeated-note (ibid/short-form)
      rendering behavior against the GB/T 7714—2025 standard's own
      conventions for consecutive same-source notes
- [ ] Add a styles.gb-t-7714-2025-note entry to
      scripts/report-data/note-position-expectations.yaml matching that
      behavior
- [ ] Confirm the note-position audit no longer reports configuration-gap
      for this style
