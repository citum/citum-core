---
# csl26-4ada
title: Add givenname-disambiguation-rule field to Disambiguation
status: completed
type: feature
priority: normal
created_at: 2026-06-02T13:49:12Z
updated_at: 2026-06-02T17:08:17Z
---

Add `givenname_rule: GivennameRule` field to `Disambiguation`. Engine should honor the scoping: `primary-name` and `primary-name-with-initials` expand only the first author; all other values expand all positions (current behavior). Initials vs full name stays driven by contributor config. All 5 CSL values modeled for round-trip fidelity. `by-cite` per-cite minimal-subset is a documented divergence.

**Specs:** `docs/specs/DISAMBIGUATION.md` §2.1, `docs/specs/CROSS_ENTRY_FIDELITY.md`

## Todo

- [x] Add `GivennameRule` enum + `givenname_rule` field to `citum-schema-style` Disambiguation
- [x] Update all `Disambiguation { .. }` literal sites (~24) to include the field
- [x] Parse `givenname-disambiguation-rule` in `csl-legacy` model + parser
- [x] Map the attribute in `citum-migrate` options extractor
- [x] Add `primary_only: bool` to `DisambiguationFlags` + `expand_given_names_primary_only` to ProcHints
- [x] Restrict given-name expansion in `format_name_list` when primary-only
- [x] New engine test: `primary-name` scoping vs `by-cite`/`all-names`
- [x] Migrate round-trip test for `givenname-disambiguation-rule="primary-name"`
- [x] Regen `docs/schemas/`

## Summary of Changes

Added `GivennameRule` enum (5 CSL values, `by-cite` default) to
`citum-schema-style` and wired it end-to-end:

- **Schema**: `Disambiguation.givenname_rule: GivennameRule`; `GivennameRule`
  re-exported from `citum_schema::options`; schema JSON regenerated.
- **csl-legacy**: parses `givenname-disambiguation-rule` attr on `<citation>`.
- **citum-migrate**: maps the attribute to `GivennameRule` (default `ByCite`).
- **Engine**: `DisambiguationFlags.primary_givenname_only` + `ProcHints.expand_given_names_primary_only`;
  `format_name_list` skips expansion for `index > 0` when primary-only is set.
- **Tests**: engine flag test (primary-name vs by-cite), migrate round-trip ×2,
  1474 tests pass.
