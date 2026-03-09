# Gendered Locale Term Forms Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-03-09
**Author:** citum team
**Related:** [CSL schema #460](https://github.com/citation-style-language/schema/issues/460), [CSL locales PR #421](https://github.com/citation-style-language/locales/pull/421), bean `csl26-y3kj`

## Purpose

CSL locale terms currently support gender only at the `<term>` element level. Arabic ordinals and Romance-language contributor role terms require gender to be specifiable on individual `<single>` and `<multiple>` child forms within the same term. This spec extends the locale data model and APIs to support per-form gender while maintaining backward compatibility with existing styles.

## Scope

**In scope:**
- New `TermGender` enum (`Masculine | Feminine | Neuter | Common`)
- Extended csl-legacy model to parse gender on `<single>` and `<multiple>` elements
- New `GenderedForms` and `GenderedSingularPlural` types in citum-schema
- Updated term lookup APIs to accept optional gender context
- Engine support for passing gender hints through rendering pipeline
- YAML deserialization support in citum-schema locales

**Out of scope:**
- Automatic gender inference from reference data (requires linguistic models, separate feature)
- Template attribute syntax for gender selection (deferred to implementation refinement)
- Exhaustive coverage of all inflected languages (start with Arabic + Romance families)

## Design

### 1. New TermGender Enumeration

```rust
/// Gender marker for inflected locale terms.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TermGender {
    Masculine,
    Feminine,
    Neuter,
    Common,  // Used when gender is not applicable or unmarked
}
```

### 2. CSL Legacy Model Extension

**File:** `crates/csl-legacy/src/model.rs`

Current structure (simplified):
```rust
pub struct Term {
    pub name: String,
    pub gender: Option<String>,
    pub single: Option<String>,
    pub multiple: Option<String>,
}
```

New structure:
```rust
pub struct Term {
    pub name: String,
    pub gender: Option<String>,
    pub single: Option<TermForm>,        // Either plain string or gendered map
    pub multiple: Option<TermForm>,
}

/// A term form can be a single ungendered string or a gender-keyed mapping.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum TermForm {
    Plain(String),
    Gendered {
        #[serde(default)]
        masculine: Option<String>,
        #[serde(default)]
        feminine: Option<String>,
        #[serde(default)]
        neuter: Option<String>,
        #[serde(default)]
        common: Option<String>,
    },
}

impl TermForm {
    /// Resolve to a single string given optional gender context.
    pub fn resolve(&self, gender: Option<TermGender>) -> Option<String> {
        match self {
            TermForm::Plain(s) => Some(s.clone()),
            TermForm::Gendered { masculine, feminine, neuter, common } => {
                match gender {
                    Some(TermGender::Masculine) => masculine.clone(),
                    Some(TermGender::Feminine) => feminine.clone(),
                    Some(TermGender::Neuter) => neuter.clone(),
                    Some(TermGender::Common) | None => common.clone().or_else(|| masculine.clone()),
                }
            }
        }
    }
}
```

XML parsing example (CSL format):
```xml
<term name="ordinal-01">
  <single gender="masculine">١٫</single>
  <single gender="feminine">١.</single>
</term>
```

Deserialized as:
```rust
Term {
    name: "ordinal-01".into(),
    gender: None,
    single: Some(TermForm::Gendered {
        masculine: Some("١٫".into()),
        feminine: Some("١.".into()),
        neuter: None,
        common: None,
    }),
    multiple: None,
}
```

### 3. Citum Schema Locale Types

**File:** `crates/citum-schema/src/locale/raw.rs`

Extend `RawTermValue` enum:
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawTermValue {
    Simple(String),
    SingularPlural(SingularPlural),
    GenderedForms(GenderedForms),     // NEW
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenderedForms {
    pub masculine: Option<String>,
    pub feminine: Option<String>,
    pub neuter: Option<String>,
    pub common: Option<String>,
}
```

**File:** `crates/citum-schema/src/locale/types.rs`

New type for template consumption:
```rust
/// Locale term form with optional gender variants.
#[derive(Clone, Debug)]
pub struct GenderedSingularPlural {
    pub singular: Option<String>,  // Can be gendered
    pub plural: Option<String>,    // Can be gendered
}

impl GenderedSingularPlural {
    /// Resolve singular form for given gender context.
    pub fn singular(&self, gender: Option<TermGender>) -> Option<String> {
        self.singular.clone()  // For v1, deferred to implementation
    }

    /// Resolve plural form for given gender context.
    pub fn plural(&self, gender: Option<TermGender>) -> Option<String> {
        self.plural.clone()
    }
}
```

### 4. Locale API Changes

**File:** `crates/citum-schema/src/locale/mod.rs`

Extend the public API:
```rust
impl Locale {
    /// Get a general term (e.g., "page", "no date") with optional gender.
    pub fn general_term(
        &self,
        name: &str,
        gender: Option<TermGender>,
    ) -> Option<String> {
        // Implementation deferred; basic version ignores gender in v1
        self.general_term(name)
    }

    /// Get a locator term (e.g., "book", "section") with optional gender.
    pub fn locator_term(
        &self,
        name: &str,
        gender: Option<TermGender>,
    ) -> Option<String> {
        self.locator_term(name)
    }

    /// Get a contributor role term (e.g., "editor", "translator") with optional gender.
    pub fn role_term(
        &self,
        name: &str,
        gender: Option<TermGender>,
    ) -> Option<String> {
        self.role_term(name)
    }
}
```

### 5. Engine Term Rendering

**File:** `crates/citum-engine/src/values/term.rs`

Pass gender context through render options:
```rust
pub struct TermRenderContext {
    pub gender: Option<TermGender>,
    pub form: Option<TermForm>,  // singular, long, etc.
}

impl TermValue {
    pub fn render(
        &self,
        locale: &Locale,
        context: &TermRenderContext,
    ) -> String {
        // Resolve gender-aware term lookup
        let term_name = self.name;
        locale
            .general_term(term_name, context.gender)
            .unwrap_or_else(|| term_name.to_string())
    }
}
```

Optionally extend `RenderOptions` (v1.1 deferral):
```rust
pub struct RenderOptions {
    // ... existing fields
    pub term_gender_hint: Option<TermGender>,  // Global default; overridden by context
}
```

### 6. YAML Locale Serialization

Citum locale YAML with gendered ordinals (future example):
```yaml
info:
  id: apa-7th-arabic
  title: APA 7th Edition (Arabic)
  default-locale: ar

terms:
  ordinal:
    ordinal-01:
      masculine: "١٫"
      feminine: "١."
  roles:
    editor:
      singular:
        masculine: "محرر"
        feminine: "محررة"
      plural:
        masculine: "محررون"
        feminine: "محررات"
```

## Implementation Notes

1. **Parser Migration:** The csl-legacy parser must handle both old format (single `<term gender="m">` wrapper) and new format (`<single gender="m">`) simultaneously.

2. **Backward Compatibility:** Existing YAML locales without gender continue to work; the gender parameter defaults to `None`, which falls back to plain form resolution.

3. **Gender Resolution Fallback:** When a specific gender is requested but not available, fall back to `common` or the first available form.

4. **Template Author Guidance:** Open question — how should template authors specify gender context? Options:
   - Hardcoded in template logic (e.g., for ordinals, always use Arabic context rules)
   - Reference data attribute (e.g., `contributor.gender` field)
   - Explicit template variable binding

5. **Testing:** Add snapshot tests in `tests/snapshots/locale/` for Arabic ordinals and Romance role terms.

## Acceptance Criteria

- [ ] `TermGender` enum defined in `crates/citum-schema/src/locale/mod.rs`
- [ ] `TermForm` and `GenderedForms` types added to csl-legacy and citum-schema
- [ ] CSL XML parser updated to handle gender on `<single>`/`<multiple>` elements
- [ ] YAML deserialization supports gendered term forms
- [ ] `Locale::general_term`, `locator_term`, `role_term` accept `Option<TermGender>`
- [ ] Engine term rendering pipeline passes gender context
- [ ] All pre-commit checks pass (cargo fmt, clippy, tests)
- [ ] Snapshot tests for Arabic ordinals and Romance role terms
- [ ] Status updated to `Active` in same commit as first implementation

## Changelog

- v1.0 (2026-03-09): Initial specification for CSL #460 gendered locale forms.
