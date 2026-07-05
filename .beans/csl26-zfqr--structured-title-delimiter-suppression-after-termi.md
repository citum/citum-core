---
# csl26-zfqr
title: Structured title delimiter suppression after terminal punctuation
status: todo
type: bug
priority: normal
tags:
    - punctuation
    - rendering
created_at: 2026-07-05T17:18:48Z
updated_at: 2026-07-05T17:18:48Z
---

Follow-up from GitHub issue #1010 / csl26-01jy.

When a structured title main part ends with significant terminal punctuation,
rendering should avoid blindly inserting the configured primary delimiter.
Example desired behavior: `Main title? And a subtitle`, not
`Main title?: And a subtitle`.

This likely belongs with broader punctuation-boundary work (`csl26-ztxq`) and
needs a spec decision for which terminal marks suppress or replace configured
structured-title delimiters across locales.

- [ ] Amend the title or punctuation spec with terminal-punctuation delimiter suppression rules
- [ ] Add structured-title tests for `?`, `!`, and other agreed terminal punctuation
- [ ] Implement delimiter suppression without breaking configurable `primary-delimiter` / `subtitle-delimiter` behavior
