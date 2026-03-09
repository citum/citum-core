---
# csl26-y3kj
title: 'Support gender on locale term single/multiple forms (CSL #460)'
status: todo
type: feature
priority: low
created_at: 2026-03-09T22:28:26Z
updated_at: 2026-03-09T22:28:57Z
---

## Context

CSL schema issue: https://github.com/citation-style-language/schema/issues/460
CSL locales PR: https://github.com/citation-style-language/locales/pull/421

Arabic requires per-form gender because masculine and feminine singular ordinals differ:
```xml
<term name="ordinal-01">
  <single gender="masculine">١٫</single>
  <single gender="feminine">١.</single>
</term>
```

Romance languages also need this for contributor role terms (editor/translator).

## Spec

See `docs/specs/GENDERED_LOCALE_TERMS.md`

## Todos

- [x] Create spec doc (docs/specs/GENDERED_LOCALE_TERMS.md)
- [ ] Extend csl-legacy Term model (single/multiple → gender-aware enum)
- [ ] Extend csl-legacy parser to read gender on child nodes
- [ ] Add GenderedForms variant to citum-schema RawTermValue
- [ ] Add GenderedSingularPlural type to citum-schema locale types
- [ ] Update term lookup APIs to accept optional TermGender param
- [ ] Pass gender context through engine term rendering
- [ ] Update docs/schemas/locale.json
