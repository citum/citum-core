---
# csl26-y3kj
title: Add MaybeGendered<T> to locale term model
status: in-progress
type: feature
priority: low
created_at: 2026-03-09T22:28:26Z
updated_at: 2026-04-25T00:00:00Z
parent: csl26-li63
---

## Context

Citum's locale model uses plain `String` for all term values (`SimpleTerm.long/short`,
`SingularPlural.singular/plural`). Inflected languages need an optional gender dimension:

- Romance languages: French "editor" is "éditeur" (m) or "éditrice" (f)
- Arabic: ordinals inflect for gender — "الأول" (m) vs "الأولى" (f)

biblatex handles this ad hoc via separate localization keys and macros; there is no systematic
locale-level gender model. Citum can do better with a typed `MaybeGendered<T>` approach.

**Prior art:** biblatex (separate keys/macros per gender), CSL #460 (XML attribute extension).
**Citum approach:** replace `String` fields in `SimpleTerm` and `SingularPlural` with
`MaybeGendered<String>` — an untagged enum that plain-string locales satisfy automatically.

## Spec

See `docs/specs/GENDERED_LOCALE_TERMS.md`

## Todos

- [x] Create spec doc (docs/specs/GENDERED_LOCALE_TERMS.md)
- [x] Add `MaybeGendered<T>` and `GrammaticalGender` to citum-schema locale types
- [x] Change `SimpleTerm.long/short` to `MaybeGendered<String>`
- [x] Change `SingularPlural.singular/plural` to `MaybeGendered<String>`
- [x] Add gendered raw term parsing for YAML deserialization
- [x] Update `Locale::role_term`, `locator_term`, `general_term` to accept `Option<GrammaticalGender>`
- [x] Pass gender context through engine term rendering for legacy term-map lookup
- [ ] Snapshot tests: French gendered editor, Arabic gendered ordinal

## Notes

The `MaybeGendered<T>` term-map model is live. This bean no longer tracks
MessageFormat 2 role-label migration; that is split to `csl26-vm2g`, which must
add `$gender` plumbing and multi-selector `.match` support before gendered role
labels can move from `roles:` to `messages:`.
