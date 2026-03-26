---
# csl26-8qs2
title: 'engine: grouped bibliography selector vs compound merge semantics'
status: todo
type: bug
priority: high
created_at: 2026-03-26T15:35:11Z
updated_at: 2026-03-26T15:35:11Z
parent: csl26-fk0w
---

Review pass found a grouped-bibliography correctness bug in the compound-entry path.

Grouped selectors currently run against already-merged compound rows, so selection can leak or drop compound members incorrectly. If only a non-leader subentry matches a selector, the whole merged row can disappear. If only the leader matches, non-matching siblings can still leak into the rendered row.

## Tasks
- [ ] Reproduce the failure with a focused regression test for grouped bibliography selection and compound entries
- [ ] Decide and document the intended rule: selectors apply to members before merge, or merged rows after merge
- [ ] Update grouped bibliography rendering to enforce that rule consistently
- [ ] Verify leader-only and non-leader-only match scenarios

Source: broad citum-engine review after PR #448.
