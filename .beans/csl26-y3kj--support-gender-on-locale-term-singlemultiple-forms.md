---
# csl26-y3kj
title: Add MaybeGendered<T> to locale term model
status: todo
type: feature
priority: low
created_at: 2026-03-09T22:28:26Z
updated_at: 2026-04-24T12:14:09Z
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
- [ ] Add `MaybeGendered<T>` and `TermGender` to citum-schema locale types
- [ ] Change `SimpleTerm.long/short` to `MaybeGendered<String>`
- [ ] Change `SingularPlural.singular/plural` to `MaybeGendered<String>`
- [ ] Add `Gendered` variant to `RawTermValue` for YAML deserialization
- [ ] Update `Locale::role_term`, `locator_term`, `general_term` to accept `Option<TermGender>`
- [ ] Pass gender context through engine term rendering
- [ ] Snapshot tests: French gendered editor, Arabic gendered ordinal
