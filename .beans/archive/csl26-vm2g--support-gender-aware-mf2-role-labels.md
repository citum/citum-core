---
# csl26-vm2g
title: Support gender-aware MF2 role labels
status: completed
type: feature
priority: normal
tags:
    - schema
    - locale
created_at: 2026-04-25T00:00:00Z
updated_at: 2026-04-30T20:30:00Z
parent: csl26-li63
---

Add the MF2 evaluator and call-site support needed to migrate gendered
contributor role labels for gendered locales from `roles:` to `messages:`
without losing the `MaybeGendered<T>` behavior that is already live in legacy
term maps.

## Todos

- [x] Pass both `$count` and `$gender` into `MessageArgs` from
      `resolved_role_term`, `resolved_role_term_neutral`,
      `resolved_locator_term`, and `resolved_general_term` where applicable.
- [x] Map `GrammaticalGender` to stable MF2 selector keys:
      `masculine`, `feminine`, `neuter`, `common`.
- [x] Extend the custom evaluator and CLI MF2 linting to support
      multi-selector `.match` while preserving existing one-selector messages.
- [x] Migrate a first confirmed gendered locale's `role.editor.*` and
      `role.translator.*` matrices using existing `roles:` strings as the
      source of truth.
- [x] Add French and Arabic migrations only after locale content and tests are
      confirmed for those languages.
- [x] Keep `roles:` as fallback until every migrated role/form has tests and
      fallback deprecation is intentional.

## Verification

- [x] Unit tests for multi-selector matching, wildcard fallback, missing gender
      fallback, and existing count-only messages.
- [x] Locale tests for feminine singular, feminine plural, masculine plural, and
      mixed/common role labels through MF2.
- [x] Engine tests proving `roles:` fallback still works when an MF2 message is
      absent or cannot evaluate.

## Notes

Do not depend on ICU4X MF2 support for this task. The ICU4X implementation is
tracked separately and can replace the evaluator later through the existing
`MessageEvaluator` trait.

## Summary of Changes

All tasks completed as part of csl26-vm2g implementation (~2026-04-26, commits 5994096c and 52d6c8ba):
- MF2 evaluator extended for multi-selector .match with $count and $gender
- GrammaticalGender mapped to stable MF2 selector keys
- French (fr-FR) and Arabic (ar-AR) role labels migrated
- Unit and engine tests added in crates/citum-engine/src/values/tests.rs
Bean had status 'done' (invalid) — corrected to completed.
