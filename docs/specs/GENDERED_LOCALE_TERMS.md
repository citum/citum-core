**Status:** Draft  
**Version:** 1.2  
**Date:** 2026-04-15  
**Bean:** `csl26-y3kj`

## Problem

Citum's locale model currently represents every term string as a plain `String`. This works for English and most uninflected languages, but breaks for inflected languages where the same semantic term takes different surface forms depending on grammatical gender.

Two concrete cases:

**Contributor role terms (Romance languages).** French “editor” is “éditeur” (masculine) or “éditrice” (feminine). A locale file author currently has no way to encode both forms; they must pick one and accept incorrect output for the other.

**Ordinals (Arabic, Romance languages).** Arabic ordinals inflect for gender: the masculine first ordinal is “الأول” while the feminine is “الأولى”. No single string can represent both.

Citum needs a way to:

- Store gendered variants of term values where required.
- Attach lexical gender metadata to noun-like terms for agreement.
- Select the right variant at render time, based on an explicit agreement context.

## Prior Art

**biblatex** handles gender ad hoc: separate localization keys per gendered variant (e.g. `idemsm`, `idempf` for the *idem* family), and separate ordinal macros (`\\mkbibmascord`, `\\mkbibfemord`, `\\mkbibneutord`) that each language module implements independently. There is no systematic gender dimension on the term data model — style authors must know which keys to call and manage gender-selection logic themselves. Explicit, but tedious and fragile.

**CSL 1.0** puts a `gender` attribute on the `<term>` element. An open issue (schema #460) proposes extending this to `<single>` and `<multiple>` child elements, allowing different genders per number form. The approach works within CSL’s XML constraints, but gender remains an attribute-level annotation rather than a first-class type.

Modern localization systems that address grammatical gender generally:

- Encode gendered variants explicitly in the resource.
- Treat “requested grammatical gender” as a contextual selector, often tied to user preferences or agreement rules.
- Keep plural rules and gender selection orthogonal.

Citum can improve on both CSL and biblatex with a typed approach that makes gender:

- A **first-class, optional dimension** on any term string.
- A **separate lexical property** of noun-like terms used for agreement.
- A **contextual parameter** in the engine when resolving term forms.

## Design Overview

Citum introduces three related but distinct pieces:

1. **Gendered term values**: `MaybeGendered<T>` carries either a single value or a set of gender-specific variants.
2. **Grammatical gender category**: `GrammaticalGender` enumerates the stable set of genders the engine recognizes for agreement.
3. **Lexical gender metadata**: optional `gender: GrammaticalGender` on noun-like locale terms to drive agreement, especially for ordinals.

The engine passes an optional **requested agreement gender** (`requested_gender: Option<GrammaticalGender>`) into term resolution. Callers that do not need gendered output pass `None` and receive existing behavior.

## Core type: `MaybeGendered<T>`

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
        feminine:  Option<T>,
        neuter:    Option<T>,

        /// Used when a language has a grammatically valid common-gender form,
        /// or as a gender-unspecified default value.
        common:    Option<T>,
    },
}
```

`serde(untagged)` means existing plain-string locale files deserialize without any changes: `"editor"` becomes `Plain("editor")`, and `{ masculine: "éditeur", feminine: "éditrice" }` becomes the `Gendered` variant.

### Resolution methods

`MaybeGendered<T>` gains two resolution helpers:

```rust
impl<T> MaybeGendered<T> {
    /// Resolve without fallback beyond the explicitly matching slot.
    ///
    /// - `Plain` always resolves.
    /// - For `Gendered`, only the requested slot is considered;
    ///   `common` is not used implicitly here.
    pub fn resolve_strict(&self, requested: Option<GrammaticalGender>) -> Option<&T> {
        // exact behavior defined in implementation
    }

    /// Resolve with documented fallback behavior.
    ///
    /// - If `Plain`, always returns the inner value.
    /// - If `Gendered`:
    ///   1. Try the requested slot (if any).
    ///   2. Then `common` (if populated).
    ///   3. Then the first populated slot in canonical order:
    ///      masculine → feminine → neuter.
    pub fn resolve_with_fallback(
        &self,
        requested: Option<GrammaticalGender>,
    ) -> Option<&T> {
        // implementation follows the rules above
    }
}
```

`resolve_strict` is reserved for validation and testing use cases (e.g., asserting that a locale provides
a specific gender slot). Production rendering always uses `resolve_with_fallback`. Acceptance criteria
below require only `resolve_with_fallback` for the production path; `resolve_strict` does not need
a production call site to be considered complete.
```

The **canonical fallback order** for the specific slots is:

1. `masculine`
2. `feminine`
3. `neuter`

This order is arbitrary but fixed and must be documented so behavior is deterministic and testable.

### Grammatical gender enum

```rust
/// Grammatical gender for agreement and term resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GrammaticalGender {
    Masculine,
    Feminine,
    Neuter,
    /// Common-gender form, when a language has a grammatically valid
    /// unmarked or shared form (e.g., some Romance inclusive forms).
    Common,
}
```

Notes:

- `GrammaticalGender` is the category type used both for lexical gender metadata and for requested agreement context.
- `Common` is a first-class category for languages that truly have common-gender forms; it is also used as a **semantically neutral requested value** when a caller knows the language prefers common-gender output.

Callers that have *no agreement information* should pass `None`, not `Some(GrammaticalGender::Common)`.

## Changes to existing types

`SimpleTerm` in `crates/citum-schema-style/src/locale/types.rs` becomes:

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

`ContributorTerm`, `LocatorTerm`, and `DateTerms` are unchanged — they compose the above types and gain gender support transitively.

**Verb-form gender scope.** `ContributorTerm.verb` (e.g., "edited by") is a `SimpleTerm` and thus technically inherits `MaybeGendered<String>` capacity. However, verb-form gender agreement is out of scope for this change — the intended use is inflected role-label nouns (e.g., "éditeur/éditrice"), not verb phrases. Locale authors MUST NOT populate gendered variants on verb-form term entries in this release; any such entries will be ignored by the engine.

No term *must* become gendered; `Plain` is the default representation and is sufficient for most locales.

**`Default` impl and existing initializers.** The `SimpleTerm` and `SingularPlural` field-type changes are source-breaking. All existing initializers — including `Terms::en_us()`, any `Default` impl, and any test fixtures that construct these structs directly — must wrap plain string literals in `MaybeGendered::Plain(...)`. No existing locale file needs to change, but Rust construction sites must be updated in the same commit as the type change.

## Lexical gender metadata on terms

In addition to gendered values, noun-like locale entries gain optional lexical gender metadata:

```rust
pub struct NounTerm {
    pub value: SingularPlural,
    pub gender: Option<GrammaticalGender>,
}
```

Schema-wise, this is surfaced as a `gender` property on the term entry:

```yaml
# In locale/fr-FR.yaml
terms:
  edition:
    singular: "édition"
    plural: "éditions"
    gender: feminine
```

Constraints:

- `gender` is only meaningful on **noun-like terms** that serve as agreement targets (e.g., “edition”, “volume”, possibly some locators).
- `gender` MUST NOT be used on terms where grammatical gender depends on the referent (e.g., contributor roles “editor/éditrice”); in those cases, `MaybeGendered<String>` carries variants and the agreement context comes from reference data.

The concrete Rust types composing these fields (`NounTerm` vs other term structs) are an implementation detail; the spec guarantees only:

- For noun-like terms, `gender: GrammaticalGender?` is available for agreement logic.
- For all terms, values are expressed via `MaybeGendered<String>` as defined above.

## YAML representation

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

A French locale declaring the lexical gender of a noun:

```yaml
terms:
  edition:
    singular: "édition"
    plural: "éditions"
    gender: feminine
```

The YAML-to-Rust mapping for term values is:

- Scalar string → `MaybeGendered::Plain`.
- Mapping with `masculine`/`feminine`/`neuter`/`common` keys → `MaybeGendered::Gendered`.
- Mixed or malformed shapes are rejected by validation.

## Engine changes

### Lookup signatures

`Locale::role_term`, `locator_term`, and `general_term` gain an optional `requested_gender: Option<GrammaticalGender>` parameter:

```rust
impl Locale {
    pub fn role_term(
        &self,
        role: Role,
        number: Number,
        requested_gender: Option<GrammaticalGender>,
    ) -> Option<&str> {
        // ...
    }

    pub fn locator_term(
        &self,
        locator: Locator,
        number: Number,
        requested_gender: Option<GrammaticalGender>,
    ) -> Option<&str> {
        // ...
    }

    pub fn general_term(
        &self,
        key: TermKey,
        number: Number,
        requested_gender: Option<GrammaticalGender>,
    ) -> Option<&str> {
        // ...
    }
}
```

Callers that do not need gendered output pass `None` and receive existing behavior:

- `Plain` values resolve unconditionally.
- `Gendered` values resolve via `resolve_with_fallback(None)`, which uses only the fallback step (canonical order and `common`), without any requested slot.

### Agreement resolution: where requested gender comes from

The engine derives `requested_gender` from three sources, in strictly defined precedence order:

1. **Template-level override** — explicit `gender` attribute on any template component that renders a term or number. Highest precedence; overrides everything else.

   ```yaml
   - number: volume
     form: ordinal
     gender: masculine       # explicit; overrides locale lookup
   ```

   This maps directly to `Some(GrammaticalGender::Masculine)` in the render context.

2. **Reference data** — a `gender` field on a contributor in the input reference, used when rendering contributor role labels (e.g., “éditeur” vs “éditrice”). The engine derives `requested_gender` from contributors **only when exactly one contributor is in scope** for the rendered label.

   YAML:

   ```yaml
   editor:
     - family: "Dupont"
       given: "Marie"
       gender: feminine
   ```

   Engine behavior:

   - If exactly one contributor is relevant and has a `gender`, use that `GrammaticalGender`.
   - If there are multiple contributors: collect only those that have an *explicit* `gender` field (contributors with no `gender` field are skipped, not treated as a mismatch). If all collected genders are identical and at least one contributor has a gender, use that shared gender.
   - If no contributor has an explicit `gender`, or the collected genders differ, do not derive a `requested_gender` from reference data (i.e., pass `None` and fall back to locale defaults).

   This avoids silently gendering plural role labels from the first name only.

3. **Locale lexical gender** — a `gender` property on the term entry itself, declaring the grammatical gender of that noun. Used for ordinals and other agreement cases that must match a noun (e.g., “1re édition” because *édition* is feminine).

   YAML (as above):

   ```yaml
   terms:
     edition:
       singular: "édition"
       plural: "éditions"
       gender: feminine
   ```

   When a template says `number: edition, form: ordinal`, the engine:

   - Looks up the “edition” term.
   - Reads its `gender` (if any).
   - Uses it as `requested_gender` when resolving the ordinal form.

If none of the three sources produces a `requested_gender`, the engine passes `None` and relies entirely on the fallback behavior in `MaybeGendered::resolve_with_fallback`.

### Resolution behavior

The engine always uses `resolve_with_fallback` when resolving term values for rendering:

```rust
let value: Option<&str> = term_value
    .resolve_with_fallback(requested_gender)
    .map(|s| s.as_str());
```

This ensures:

- Template-level override feeds directly into the gender slot selection.
- When an ordinal is tied to a noun with lexical gender, that gender drives agreement automatically.
- When no agreement information is known, `Plain` values still work, and `Gendered` values fall back in a deterministic order.

### Out-of-scope semantics

The following remain out of scope for this change:

- Automatic gender inference from reference data beyond explicit `gender` fields (e.g., inferring from first names).
- Grammatical case, animacy, person, or full declension tables.
- Higher-level message-format constructs that perform gender agreement across whole sentences (ICU MessageFormat-style selects).

The gender feature is intentionally narrow: it only affects locale **term** values and agreement with those terms.

## Raw term value and deserialization

`RawTermValue` (the internal AST used during YAML deserialization) gains a variant that directly mirrors `MaybeGendered<String>`:

```rust
pub enum RawTermValue {
    Scalar(String),
    Gendered {
        masculine: Option<String>,
        feminine:  Option<String>,
        neuter:    Option<String>,
        common:    Option<String>,
    },
    // … other existing shapes, if any
}
```

**Mapping from existing `RawTermValue` variants to `MaybeGendered<String>`.** The existing `RawTermValue` in `raw.rs` currently has three variants:

| Existing variant | Converted to |
|------------------|--------------|
| `Simple(String)` / `Scalar(String)` | `MaybeGendered::Plain(String)` |
| `SingularPlural { singular, plural }` | Conversion not applicable here — these map to `SingularPlural` at the struct level, where each field then becomes `MaybeGendered<String>`. See "Changes to existing types" above. |
| `Forms(HashMap<String, RawTermValue>)` | **Retired or narrowed.** The `Forms` variant currently accepts arbitrary key → value maps. A YAML object with only `masculine`/`feminine`/`neuter`/`common` keys is now deserialized as the `Gendered` variant instead of `Forms`. `Forms` is removed or reserved for future structured shapes that do not fit `Gendered`; it MUST NOT silently absorb gendered-variant maps. |
| *(new)* `Gendered { masculine, feminine, neuter, common }` | `MaybeGendered::Gendered { … }` |

The `Gendered` variant is a dedicated discriminant (rather than relying on `Forms`) because:

- `Forms` has no schema constraint on its keys and produces opaque fallback behavior.
- A dedicated variant enables precise validation errors ("unexpected key in gendered term") and exhaustive pattern matching.

Deserialization layers:

1. YAML scalar or mapping → `RawTermValue`.
2. `RawTermValue` → `MaybeGendered<String>`.

Rationale:

- `RawTermValue` continues to serve as a syntax-layer representation; it can reject malformed shapes with clear diagnostics before committing to the typed model.
- `MaybeGendered<String>` is the stable, public representation that style authors and engine code use after validation.

**Note for implementers.** `serde(untagged)` on `MaybeGendered<T>` produces opaque deserialization errors by default. Add a custom `Deserialize` impl or a wrapper that produces a clear diagnostic (e.g., "expected a string or a map with masculine/feminine/neuter/common keys") when an unexpected shape is encountered.

This two-step approach keeps error reporting on YAML inputs precise while still providing a single typed model for use in the rest of the engine.

## Backwards compatibility

The YAML data model change is additive:

- Plain-string values remain valid and map to `MaybeGendered::Plain`.
- Locale files that do not use gendered variants or lexical `gender` metadata behave exactly as before.

The Rust API change — new `requested_gender: Option<GrammaticalGender>` parameter on term lookup methods — is breaking but acceptable before 1.0.

Existing locale tests MUST continue to pass unchanged:

- Plain-string values round-trip correctly through YAML → Rust → YAML.
- Lookups with `None` for `requested_gender` produce the same output as before.

## Acceptance criteria

- [ ] `MaybeGendered<T>` defined in `crates/citum-schema-style/src/locale/types.rs` with `resolve_strict` and `resolve_with_fallback` as specified.
- [ ] `GrammaticalGender` defined and used consistently for both lexical metadata and requested agreement context.
- [ ] `SimpleTerm.long` / `.short` changed to `MaybeGendered<String>`.
- [ ] `SingularPlural.singular` / `.plural` changed to `MaybeGendered<String>`.
- [ ] Noun-like terms support optional lexical `gender: GrammaticalGender` in the locale schema and corresponding Rust types.
- [ ] `Locale::role_term`, `locator_term`, `general_term` accept `requested_gender: Option<GrammaticalGender>`.
- [ ] All existing locale tests pass (plain-string values round-trip correctly; behavior with `None` requested gender matches prior output).
- [ ] New snapshot tests:
      - French contributor role with gendered variants (e.g., “éditeur/éditrice”).
      - Arabic gendered ordinal agreeing with a feminine noun.
- [ ] New unit tests for:
      - `resolve_with_fallback` behavior with each slot populated individually.
      - Deterministic fallback when `requested_gender` is `None`.
      - Reference-data behavior for one vs multiple contributors with matching and mixed genders.
      - Ordinal agreement derived from lexical noun gender.
- [ ] `RawTermValue` extended with a `Gendered`-shaped variant and conversion to `MaybeGendered<String>`.
- [ ] This spec’s `Status` set to `Active` in the same commit as the first implementation.

## Suggested external references

For future implementers and spec readers, these resources provide additional context on grammatical gender in software localization:

- [Android 14 Grammatical Inflection API](https://developer.android.com/about/versions/14/features/grammatical-inflection) — user-centric grammatical gender selection in resource lookup.
- [Localazy: Beyond interpolation — multiple plurals, genders and building lists](https://localazy.com/blog/beyond-interpolation-multiple-plurals-genders-and-building-lists) — practical patterns for plural and gender handling in message resources.
- [Phrase: Linguistics for developers — real-world i18n challenges](https://phrase.com/blog/posts/internationalization-beyond-code-a-developers-guide-to-real-world-language-challenges/) — discussion of gendered constructions across languages.
- [Shopify Engineering: i18n best practices for front-end developers](https://shopify.engineering/internationalization-i18n-best-practices-front-end-developers) — examples of noun gender and agreement in UI copy.

## Changelog

### v1.2

- Clarified `resolve_strict` scope: reserved for validation and testing only; production rendering always uses `resolve_with_fallback`. Removed `resolve_strict` from production acceptance criteria.

- Explicitly bridged existing `RawTermValue` variants to `MaybeGendered<String>` with a conversion table: `Simple`/`Scalar` → `Plain`; existing `Forms` variant retired/narrowed (no longer absorbs gendered maps); new `Gendered` variant → `Gendered`. Explained why a dedicated discriminant is required over `Forms`. Added a note about `serde(untagged)` error opacity and the need for a custom diagnostic.

- Added explicit note that `Terms::en_us()`, all `Default` impls, and Rust construction sites must wrap plain string literals in `MaybeGendered::Plain(...)` in the same commit as the type change.

- Scoped verb-form gender: `ContributorTerm.verb` technically inherits `MaybeGendered` capacity but gendered verb forms are out of scope for this release; locale authors must not populate gendered variants on verb-form entries.

- Tightened multi-contributor gender rule: contributors without an explicit `gender` field are *skipped* (not treated as a mismatch); only contributors with an explicit field participate in the unanimity check.

- Removed duplicate bracketed inline link citations throughout the document; external references are now listed only in the "Suggested external references" section.

### v1.1

- Clarified the role of `MaybeGendered<T>` by specifying two resolution helpers, `resolve_strict` and `resolve_with_fallback`, and defining a deterministic fallback order for gendered slots (requested → `common` → masculine → feminine → neuter).

- Renamed and stabilized the gender category type as `GrammaticalGender`, and documented its use both as lexical gender metadata on noun-like terms and as the requested agreement context in engine calls.

- Introduced explicit lexical `gender: GrammaticalGender` metadata for noun-like locale terms (e.g., `edition`), and defined how ordinal rendering derives `requested_gender` from this metadata for gender agreement.

- Tightened the agreement source precedence to a clear chain: template-level override → reference-data contributor gender (with defined behavior for single vs multiple contributors) → locale lexical gender, falling back to locale defaults when none apply.

- Specified which term kinds may safely carry lexical gender metadata vs those that should only use gendered variants (e.g., contributor roles), to avoid overloading `gender` on terms whose form depends on the referent.

- Clarified the role of `RawTermValue` as a syntax-layer AST and documented its `Gendered`-shaped variant and conversion into `MaybeGendered<String>` to keep YAML validation and runtime representation aligned.

