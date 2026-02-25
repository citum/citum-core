---
# csl26-vdrn
title: 'InputReference deny_unknown_fields wart: pre-stabilization mitigation'
status: todo
type: bug
priority: normal
created_at: 2026-02-25T12:21:13Z
updated_at: 2026-02-25T12:21:13Z
---

## Problem

`InputReference` variants cannot use `#[serde(deny_unknown_fields)]` due to a serde limitation: internally-tagged enum dispatch replays the tag field into the inner struct's deserializer, making `deny_unknown_fields` incompatible. The result: a caller who passes a reference with a mistyped field (e.g., `publischer` instead of `publisher`) gets no error — the field is silently ignored, the output is wrong, and there is no diagnostic.

This is the one gap in our "strict validation" design principle. Documented in DESIGN_PRINCIPLES.md §3.

## Why it matters before spec stabilization

Once the schema is declared stable, callers (Zotero, Pandoc, etc.) will rely on the current behavior as the contract. Mitigation is much cheaper before adoption than after.

## Options

1. **Post-deserialize validation pass** — after deserializing an `InputReference`, check for `None` on fields expected for that reference type (e.g., `publisher` for `book`). Surface a structured warning or error. Low complexity; fits the `csln validate` subcommand pattern.

2. **Custom `Deserialize` impl** — replicate `deny_unknown_fields` behavior via a manual deserializer on the enum. High complexity; tight coupling to serde internals.

3. **Accept + expose via `csln validate`** — document it clearly, add an explicit validation subcommand users can run to catch mistyped fields. Least invasive; fits "processor stays dumb, caller validates explicitly" ethos.

**Recommendation:** Option 3 for immediate stabilization; Option 1 as follow-up for richer diagnostics.

## References

- DESIGN_PRINCIPLES.md §3 (existing documentation of the trade-off)
- ARCHITECTURAL_SOUNDNESS_2026-02-25.md
