---
# csl26-j5xm
title: Design explicit type-suppression mechanism for embedded styles
status: todo
type: task
priority: low
created_at: 2026-06-24T20:53:53Z
updated_at: 2026-06-24T20:53:53Z
---

chicago-shortened-notes-bibliography-core.yaml currently suppresses personal-communication with an empty array (personal-communication: [] / personal_communication: []) — a workaround, not an intentional syntax. Two issues: (1) there is no explicit 'suppress this type' keyword in the style schema; (2) the hyphen/underscore duplication for the same type key. Design and implement an explicit suppression syntax (e.g. suppress: [personal-communication]) and fix the duplication. Deferred from PR #964 review (Group C comment).
