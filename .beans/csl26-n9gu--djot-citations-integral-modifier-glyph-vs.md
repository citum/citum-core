---
# csl26-n9gu
title: 'Djot citations: integral modifier glyph + vs ='
status: todo
type: task
priority: low
tags:
    - citation
    - parser
    - forward-compat
created_at: 2026-09-12T13:42:45Z
updated_at: 2026-09-12T13:42:45Z
parent: csl26-s2zo
---

`parse_integral_modifier` parses `+` as the integral/narrative marker. jgm's comments on djot#32
drift from `+@foo` (his earlier proposal, which our implementation was built against) to `=@foo`
(his restated proposal, 2026-09-12) with no explicit call-out of the change.

Blocked on jgm confirming the switch is deliberate. Asked in the reply to djot#32.

## Scope

If confirmed, this is small and mechanical:

- One-character change in `parse_integral_modifier`
  (`crates/citum-engine/src/processor/document/djot/parsing.rs`).
- Update `test_parse_bracketed_integral_citation` and any fixtures using `[+@key]`.
- Update user-facing docs that show the integral citation form.
- Decide whether `+` stays accepted as a deprecated alias or is dropped outright. Prefer dropped —
  the syntax is not yet stable upstream and an alias entrenches a form the spec may not include.
