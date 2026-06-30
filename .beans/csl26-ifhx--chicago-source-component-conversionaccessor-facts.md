---
# csl26-ifhx
title: Chicago source-component conversion/accessor facts
status: todo
type: feature
priority: high
created_at: 2026-06-30T14:29:54Z
updated_at: 2026-06-30T14:29:54Z
parent: csl26-40n4
blocked_by:
    - csl26-fr6f
---

Add missing source-component facts to Citum conversion/accessors (crates/citum-migrate + engine accessors): archival correspondence, recordings, performances, broadcasts, original publication dates, event dates, note-derived roles. These facts are shared needs across author-date, notes, shortened-notes, and T&F — the largest remaining fidelity lever per the audit.

## Todo
- [ ] Enumerate exact missing facts and target variant(s) from the audit (csl26-fr6f)
- [ ] Implement accessor/conversion support per fact
- [ ] Add Rust tests per docs/guides/test-coverage conventions
- [ ] Verify fidelity delta via report-core.js
