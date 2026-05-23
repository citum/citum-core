---
# csl26-gfx5
title: Fix oscola.yaml bibliography name-form to match no-ibid variant
status: scrapped
type: task
priority: normal
created_at: 2026-05-23T13:02:26Z
updated_at: 2026-05-23T13:06:39Z
---

oscola.yaml bibliography options use name-form: full but should use name-form: initials + initialize-with: "" (matching oscola-no-ibid.yaml). This causes 3/34 bibliography failures (entries 32, 33, 34) in the oracle. Fix: copy the two fields from oscola-no-ibid bibliography options.

## Reasons for Scrapping

Duplicate of csl26-y9xu which completed the same work.
