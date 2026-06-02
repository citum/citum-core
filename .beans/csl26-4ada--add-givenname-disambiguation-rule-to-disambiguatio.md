---
# csl26-4ada
title: Add givenname-disambiguation-rule field to Disambiguation
status: in-progress
type: feature
priority: normal
created_at: 2026-06-02T13:49:12Z
updated_at: 2026-06-02T16:42:31Z
---

Add `givenname_rule: GivennameRule` field to `Disambiguation`. Engine should honor the scoping: `primary-name` and `primary-name-with-initials` expand only the first author; all other values expand all positions (current behavior). Initials vs full name stays driven by contributor config. All 5 CSL values modeled for round-trip fidelity. `by-cite` per-cite minimal-subset is a documented divergence.

**Specs:** `docs/specs/DISAMBIGUATION.md` §2.1, `docs/specs/CROSS_ENTRY_FIDELITY.md`

## Todo

- [ ] Add `GivennameRule` enum + `givenname_rule` field to `citum-schema-style` Disambiguation
- [ ] Update all `Disambiguation { .. }` literal sites (~24) to include the field
- [ ] Parse `givenname-disambiguation-rule` in `csl-legacy` model + parser
- [ ] Map the attribute in `citum-migrate` options extractor
- [ ] Add `primary_only: bool` to `DisambiguationFlags` + `expand_given_names_primary_only` to ProcHints
- [ ] Restrict given-name expansion in `format_name_list` when primary-only
- [ ] New engine test: `primary-name` scoping vs `by-cite`/`all-names`
- [ ] Migrate round-trip test for `givenname-disambiguation-rule="primary-name"`
- [ ] Regen `docs/schemas/`
