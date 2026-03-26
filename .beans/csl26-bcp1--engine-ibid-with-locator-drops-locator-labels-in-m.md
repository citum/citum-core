---
# csl26-bcp1
title: 'engine: ibid with locator drops locator labels in manual notes'
status: todo
type: bug
priority: high
created_at: 2026-03-26T15:35:11Z
updated_at: 2026-03-26T15:35:11Z
parent: csl26-fk0w
---

Review pass found a note-pipeline correctness bug in manual footnote flows.

When a manual note reaches the IbidWithLocator path, the reduced note text preserves only the locator value and drops the localized label. That turns values like `p. 23` into `23`.

## Tasks
- [ ] Add a regression test for a manual-note flow that reaches IbidWithLocator with a labeled locator
- [ ] Preserve rendered locator labels in reduced note text instead of only raw locator values
- [ ] Verify page and chapter-style locators still render correctly in manual notes

Source: broad citum-engine review after PR #448.
