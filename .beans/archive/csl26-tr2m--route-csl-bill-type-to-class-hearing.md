---
# csl26-tr2m
title: 'Route CSL bill type to class: hearing'
status: completed
type: feature
priority: normal
tags:
    - migrate
created_at: 2026-04-04T14:00:22Z
updated_at: 2026-04-30T19:18:49Z
---

Congressional hearing items in Zotero export as CSL type `bill` but should
convert to `class: hearing` in Citum (the schema has a dedicated `Hearing`
struct in legal.rs with `authority` and `session_number` fields).

Example: `6188419/VCP6HK7G` — "Homeland Security Act of 2002: Hearings on H.R. 5005"
currently converts to `class: monograph / type: document`.

## Tasks
- [ ] Determine a reliable signal for hearings in CSL-JSON (e.g. Zotero stores
  a `Medium` or extra field, or the title contains "Hearings on").
- [ ] Add routing in `from_bill_ref` (conversion.rs) to produce
  `InputReference::Hearing` when the signal is present.
- [ ] Add a focused integration test for the hearing route.
