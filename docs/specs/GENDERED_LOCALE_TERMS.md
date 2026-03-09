# Gendered Locale Term Forms

**Status:** Draft
**Version:** 1.0
**Date:** 2026-03-09
**Bean:** `csl26-y3kj`

## Problem

Citum's locale model represents every term string as a plain `String`. This works for English
and most uninflected languages, but breaks for inflected languages where the same semantic term
takes different surface forms depending on grammatical gender.

Two concrete cases:

**Contributor role terms (Romance languages).** French "editor" is "éditeur" (masculine) or
"éditrice" (feminine). A locale file author currently has no way to encode both forms; they
must pick one and accept incorrect output for the other.

**Ordinals (Arabic, Romance languages).** Arabic ordinals inflect for gender: the masculine
first ordinal is "الأول" while the feminine is "الأولى". No single string can represent both.

## Prior Art

**biblatex** handles gender ad hoc: separate localization keys per gendered variant (e.g.
`idemsm`, `idempf` for the *idem* family), and separate ordinal macros (`\mkbibmascord`,
`\mkbibfemord`, `\mkbibneutord`) that each language module implements independently. There is
no systematic gender dimension on the term data model — style authors must know which keys to
call and manage gender-selection logic themselves. Explicit, but tedious and fragile.

**CSL 1.0** puts a `gender` attribute on the `<term>` element. An open issue (schema #460)
proposes extending this to `<single>` and `<multiple>` child elements, allowing different
genders per number form. The approach works within CSL's XML constraints, but gender remains
an attribute-level annotation rather than a first-class type.

Citum can improve on both: a typed `MaybeGendered<T>` approach makes gender an optional,
orthogonal dimension on any term string, with no separate keys or explicit dispatch logic.

## Design

### Core type: `MaybeGendered<T>`

```rust
/// A value that is either uniform across grammatical genders, or gender-specific.
///
/// `Plain` covers the common case (most English and uninflected language terms).
/// `Gendered` is used only where a language requires it; only the applicable
/// variants need to be populated.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaybeGendered<T> {
    /// The same value regardless of grammatical gender.
    Plain(T),
    /// Separate values per grammatical gender.
    Gendered {
        masculine: Option<T>,
        feminine: Option<T>,
        neuter: Option<T>,
        /// Used when gender is unknown, inapplicable, or unmarked.
        common: Option<T>,
    },
}
```

`serde(untagged)` means existing plain-string locale files deserialize without any changes:
`"editor"` becomes `Plain("editor")`, and `{ masculine: "éditeur", feminine: "éditrice" }`
becomes the `Gendered` variant. A `resolve(gender: Option<TermGender>) -> Option<&T>` method
handles the fallback chain: requested gender → `common` → first available variant.

### `TermGender` enum

```rust
/// Grammatical gender for term resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TermGender {
    Masculine,
    Feminine,
    Neuter,
    /// Gender-unmarked or inapplicable.
    Common,
}
```

### Changes to existing types

`SimpleTerm` in `crates/citum-schema/src/locale/types.rs` becomes:

```rust
pub struct SimpleTerm {
    pub long:  MaybeGendered<String>,
    pub short: MaybeGendered<String>,
}
```

`SingularPlural` becomes:

```rust
pub struct SingularPlural {
    pub singular: MaybeGendered<String>,
    pub plural:   MaybeGendered<String>,
}
```

`ContributorTerm`, `LocatorTerm`, and `DateTerms` are unchanged — they compose the above
types and gain gender support transitively.

### YAML representation

Existing locales require no changes:

```yaml
roles:
  editor:
    long:
      singular: "editor"
      plural: "editors"
```

A French locale adds gender variants only where the language requires them:

```yaml
roles:
  editor:
    long:
      singular:
        masculine: "éditeur"
        feminine:  "éditrice"
      plural:
        masculine: "éditeurs"
        feminine:  "éditrices"
```

An Arabic locale for ordinals (once ordinal term support lands):

```yaml
terms:
  ordinal-01:
    singular:
      masculine: "الأول"
      feminine:  "الأولى"
```

### Engine changes

`Locale::role_term`, `locator_term`, and `general_term` gain an optional
`gender: Option<TermGender>` parameter. Callers that do not need gendered output pass `None`
and receive existing behavior: `Plain` values resolve unconditionally; `Gendered` values fall
back to `common` or the first populated variant.

Three sources supply gender, resolved in precedence order:

**1. Template-level override** — explicit `gender` attribute on any template component that
renders a term or number. Highest precedence; beats everything else.

```yaml
- number: volume
  form: ordinal
  gender: masculine        # explicit; overrides locale lookup
```

**2. Reference data** — a `gender` field on a contributor in the input reference. Used when
rendering contributor role labels (e.g. "éditeur" vs "éditrice"); the engine reads the gender
of the first (or only) contributor in scope.

```yaml
editor:
  - family: "Dupont"
    given: "Marie"
    gender: feminine
```

**3. Locale term gender** — a `gender` property on the term entry itself, declaring the
grammatical gender of that noun. Used for ordinals that must agree with a noun (e.g. "1re
édition" because "édition" is feminine).

```yaml
# In locale/fr-FR.yaml
terms:
  edition:
    singular: "édition"
    plural: "éditions"
    gender: feminine
```

When a template says `number: edition, form: ordinal`, the engine looks up the gender of the
"edition" term automatically — no per-use annotation needed.

If none of the three sources resolves, the engine falls back to `common` or the first populated
variant in the `MaybeGendered` value.

The implementation details of this resolution chain (how `gender` is threaded through render
context, where `gender` lives in the template schema) are left to the implementation phase.

## Out of scope

- Automatic gender inference from reference data.
- Grammatical case or full declension tables.
- Specific language locale files — the type change is the deliverable; individual locales ship
  separately.

## Backwards compatibility

The YAML data model change is additive: a new untagged enum variant that existing plain-string
values satisfy. The Rust API change (new `Option<TermGender>` parameter on term lookup methods)
is breaking, acceptable before 1.0.

## Acceptance criteria

- [ ] `MaybeGendered<T>` and `TermGender` defined in `crates/citum-schema/src/locale/types.rs`
- [ ] `SimpleTerm.long` / `.short` changed to `MaybeGendered<String>`
- [ ] `SingularPlural.singular` / `.plural` changed to `MaybeGendered<String>`
- [ ] `Locale::role_term`, `locator_term`, `general_term` accept `Option<TermGender>`
- [ ] All existing locale tests pass (plain-string values round-trip correctly)
- [ ] New snapshot tests: French gendered editor term, Arabic gendered ordinal
- [ ] `RawTermValue` extended with a `Gendered` variant for YAML deserialization
- [ ] Status set to `Active` in the same commit as the first implementation
