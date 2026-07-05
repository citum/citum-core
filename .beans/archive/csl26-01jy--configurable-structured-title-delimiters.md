---
# csl26-01jy
title: Configurable structured title delimiters
status: completed
type: feature
priority: normal
created_at: 2026-07-05T17:08:53Z
updated_at: 2026-07-05T17:23:17Z
---

Related: https://github.com/citum/citum-core/issues/1010
Specs: docs/specs/TITLE_TEXT_CASE.md, docs/specs/LOCALE_MESSAGES.md

- [x] Amend structured title delimiter spec
- [x] Add style-level title delimiter options
- [x] Add locale grammar defaults for structured title delimiters
- [x] Render structured titles with primary/subtitle delimiters
- [x] Add tests for default, custom, locale override, and short form
- [x] Regenerate schemas and pass pre-commit

## Summary of Changes

Implemented locale-backed structured title delimiters for GitHub issue #1010.
Added locale `grammar-options.title-subtitle-delimiter` for the
main-title-to-subtitle-group boundary and `grammar-options.subtitle-delimiter`
for subtitle-to-subtitle boundaries, while keeping style-level
`primary-delimiter` and `subtitle-delimiter` as explicit overrides. The
fallback defaults are `: ` for main-to-subtitle and `; ` for subtitle lists,
with behavior documented in `docs/specs/TITLE_TEXT_CASE.md` and
`docs/specs/LOCALE_MESSAGES.md`. Regenerated `docs/schemas/style.json` and
added renderer/schema tests.

Validation:
- `just schema-gen`
- `cargo test -p citum-schema-style test_title_locale_overrides_deserialization`
- `cargo test -p citum-engine structured_title`
- `just pre-commit` (1779 tests passed)

Follow-up punctuation suppression is tracked in `csl26-zfqr`.
