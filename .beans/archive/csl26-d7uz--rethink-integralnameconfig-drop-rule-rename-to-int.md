---
# csl26-d7uz
title: Rethink IntegralNameConfig — drop rule, rename to IntegralNameMemoryConfig
status: completed
type: feature
priority: high
created_at: 2026-05-23T07:45:46Z
updated_at: 2026-05-23T08:53:53Z
---

Pre-1.0 breaking schema redesign. Drop IntegralNameRule enum (block presence = on-switch). Rename Rust type IntegralNameConfig → IntegralNameMemoryConfig and YAML key integral-names → integral-name-memory. Rename IntegralNameForm → SubsequentNameForm. Restore MLA's name-memory block in the new shape. See /Users/brucedarcus/.claude/plans/on-a-new-pr-staged-swing.md and docs/specs/INTEGRAL_NAME_MEMORY.md.

## Tasks
- [x] Schema crate: drop IntegralNameRule + rule field; rename types
- [x] Engine crate: drop FullThenShort guards; rename imports
- [x] DocumentIntegralNameOverride: drop rule
- [x] Restore MLA integral-name-memory block (contexts: body-only, subsequent-form: short)
- [x] Update SHORT_NAME.md spec
- [x] Create INTEGRAL_NAME_MEMORY.md spec
- [x] Regenerate docs/schemas/{style,server}.json
- [x] Pre-commit gate green (fmt, clippy, nextest — all 1365 tests pass)
- [ ] Oracle: MLA fidelity not regressed (deferred — verify in PR)
- [ ] Quality-gate baseline check (deferred — verify in PR)

## Summary of Changes

Dropped `IntegralNameRule` enum (block presence is now the on-switch). Renamed `IntegralNameConfig` -> `IntegralNameMemoryConfig`, `IntegralNameForm` -> `SubsequentNameForm`. YAML key `integral-names` -> `integral-name-memory`. Restored MLA's narrative-name-memory block (removed in 2f3a9e8f) in the new shape: `contexts: body-only`, `subsequent-form: short`. New spec at docs/specs/INTEGRAL_NAME_MEMORY.md. SHORT_NAME.md updated. Schemas regenerated. Single `feat!:` change; pre-1.0, no compat shim.
