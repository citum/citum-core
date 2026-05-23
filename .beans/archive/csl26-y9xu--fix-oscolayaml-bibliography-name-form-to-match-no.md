---
# csl26-y9xu
title: Fix oscola.yaml bibliography name-form to match no-ibid variant
status: completed
type: task
priority: normal
created_at: 2026-05-23T13:02:29Z
updated_at: 2026-05-23T13:06:10Z
---

oscola.yaml bibliography options use name-form: full but should use name-form: initials + initialize-with: '' (matching oscola-no-ibid.yaml). This causes 3/34 bibliography failures.

## Summary of Changes

Changed  from  to  and added  in . This matches the existing  configuration, which already passed 34/34.

**Fidelity:** bibliography 31/34 → 34/34 (100%). Citations remain 18/18. Core quality gate: 154 styles, all pass.
