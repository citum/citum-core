# Locale Messages Specification

**Status:** Draft
**Version:** 1.1
**Date:** 2026-03-18
**Supersedes:** (none)
**Related:** bean `csl26-xd7e`

## Purpose

Replace Citum's flat key-to-static-string locale model with a parameterized
message system based on ICU Message Format 1 (MF1). This separates language
realization (words, inflection, punctuation, date and number formats) from
style structure (field order, conditions, what to omit), and enables
composition of `StylePreset × LocalePreset × LocaleOverride` instead of
duplicating styles per language.

## Scope

**In scope:**
- Schema additions to `RawLocale` and `Locale` for `messages`, `dateFormats`,
  `numberFormats`, `grammarOptions`, `legacyTermAliases`.
- `LocaleOverride` struct and merge semantics.
- `MessageEvaluator` trait and its ICU-backed implementation.
- Migration compatibility layer: dual-path lookup and `localeSchemaVersion`
  gating.
- CLI lint tooling: `citum locale lint` and `citum style lint --locale`.
- Updated `en-US.yaml` and `de-DE.yaml` with v2 messages for high-impact terms.
- `scripts/migrate-locale-v1-to-v2.js` for automated v1 → v2 conversion.

**Out of scope:**
- Full Fluent (`.ftl`) syntax support.
- MF2 message evaluation (designed for, but not implemented here — see §1.5).
- MF2 custom formatter registration API (see §1.5).
- Gender agreement for contributor name declension (tracked separately).
- Locale discovery or registry beyond the existing file-based `locales/`
  directory.
- Inline style-level locale overrides embedded directly inside style YAML.

---

## Design

### 1. Message Syntax: ICU Message Format 1

Citum adopts **ICU Message Format 1 (MF1)** as the canonical message syntax,
with the conceptual model designed for forward compatibility with MF2 (see
§1.5). The first implementation targets a minimal MF1-compatible subset behind
a thin Rust abstraction (`MessageEvaluator` trait) so the underlying evaluation
engine can be swapped without changing locale files or call sites.

Rationale for MF1 over Fluent:

- **YAML compatibility.** MF1 strings are ordinary YAML scalars. Fluent's
  `.ftl` multi-line identifier syntax does not compose cleanly inside YAML
  map values.
- **Rust ecosystem.** The `icu4x` family provides `icu_plurals` with CLDR
  plural rules, zero-copy, and WASM compatibility. No external runtime is
  required.
- **Mechanical migration.** Existing `singular`/`plural` pairs convert
  directly to `{count, plural, one{…} other{…}}`, enabling an automated
  migration script.
- **Interoperability.** MF1 is understood by Lokalise, Crowdin, and most
  major i18n platforms, simplifying future community locale contributions.

Supported MF1 constructs (v2 locales with `evaluation.message-syntax: mf1`):

| Construct | Syntax | Notes |
|-----------|--------|-------|
| Plain text | `"retrieved"` | No variables. |
| Variable interpolation | `"{names}"` | Named string substitution. |
| Plural | `{count, plural, one{p.} other{pp.}}` | CLDR category dispatch via `icu_plurals`. |
| Select | `{gender, select, masc{él} fem{ella} other{elle}}` | Arbitrary string-keyed dispatch. |
| Number | `{count, number}` | Locale-formatted integer. |

**Date formatting is not done inside MF1 messages.** The engine formats dates
using the `dateFormats` map and passes the result as a plain `{date}` variable.
This is a deliberate deferral of MF2-style custom formatter annotations (e.g.
`{$date :citum-date format=bib-default}`) — see §1.5.

---

### 1.4 Message Types

Messages in the `messages` map fall into two categories. The `MessageEvaluator`
dispatches based on the active `evaluation.message-syntax`.

**Static messages** — plain text strings with no `{…}` delimiters.

```yaml
term.and: "and"
role.editor.verb: "edited by"
date.open-ended: "present"
```

Evaluated immediately in all schema versions. No evaluator required. The
`general_term()` lookup path handles these in Phase 0.

**Parameterized messages** — ICU MF1 (or future MF2) syntax containing
variable references or selector constructs.

```yaml
term.page-label: "{count, plural, one {p.} other {pp.}}"
pattern.page-range: "{start}–{end}"
```

Stored in the `messages` map in all schema versions, but only evaluated
when `MessageEvaluator` is wired in (Phase 4). Before Phase 4, the engine
silently falls back to the legacy term-map path for any message whose body
contains `{`. This fallback is intentional — static terms already cover
the rendering surface that Phase 0–3 exercises.

The distinction is runtime-only: both types use identical YAML representation
(a string value under a message ID key). The engine classifies a message as
parameterized if and only if its body contains a `{` character.

---

### 1.5 MF2 Forward-Compatibility Design

ICU Message Format 2 (MF2) is the successor to MF1, currently at Unicode
Candidate Recommendation status. MF1 and MF2 are syntactically incompatible,
so migration requires changes to locale files. Citum's design minimises the
blast radius of that future migration.

#### Abstraction boundary

The `MessageEvaluator` trait (§7.3) is the single seam between the rest of
Citum and any message format implementation. Call sites in `citum-engine` use
only the trait; they never depend on ICU, MF1 syntax, or any format-specific
AST. This means:

- Adding `IcuMf2MessageEvaluator` (MF2 engine) is additive — no call-site
  changes.
- Switching a locale from MF1 to MF2 is a two-step: update the locale file's
  `evaluation.message-syntax: mf2` and rewrite the message strings to MF2
  syntax.

#### `evaluation.message-syntax` dispatch

The `evaluation` block on each `LocalePreset` (§3) declares which syntax the
`messages` map uses. The engine selects the matching `MessageEvaluator`
implementation at locale-load time:

| `message-syntax` | Evaluator | Status |
|------------------|-----------|--------|
| `static` (default) | None — plain string return | Phase 0 |
| `mf1` | `IcuMf1MessageEvaluator` | Phase 4 |
| `mf2` | `IcuMf2MessageEvaluator` | Future |

A locale file declaring `message-syntax: mf1` signals intent. Until Phase 4
is complete, the engine falls back to static-only evaluation for parameterized
messages rather than erroring.

#### `MessageArgs` as MF2 named variables

`MessageArgs` (§7.3) maps directly to MF2's named-variable model: each field
(`count`, `gender`, `names`, …) corresponds to a `$variable` in an MF2
message. No structural change to `MessageArgs` is anticipated when migrating
to MF2.

MF2 multi-selector patterns (matching on multiple variables simultaneously)
are accommodated by `MessageArgs` having multiple fields — the added
expressiveness is in the message syntax, not in the Rust argument type.

#### Custom formatters (MF2 concept)

MF2 supports custom function annotations: `{$date :citum-date format=bib-default}`.
In MF1 (and in Citum's current design) this is handled by pre-formatting:
the engine computes the date string using `dateFormats["bib-default"]` and
passes it as a plain `{date}` variable.

This is intentionally conservative. Pre-formatting keeps the formatting logic
in typed Rust code with full EDTF awareness, and avoids a two-layer parsing
problem (CLDR date skeletons embedded inside ICU messages). The `MessageEvaluator`
trait is structurally equivalent to a custom function registry at the Rust level —
each implementation is a formatter. When MF2 is adopted, `:citum-date` and
`:citum-names` can be registered as custom functions in `IcuMf2MessageEvaluator`
without changing any call sites or the `MessageArgs` shape.

The `dateFormats` map (§3) is the stable API regardless of when custom
formatter annotations are adopted: both the pre-formatting path and a future
`:citum-date` formatter consume the same symbolic name → CLDR pattern mapping.

---

### 2. Message ID Namespace

All message IDs are dot-namespaced strings:

| Prefix | Meaning | Example |
|--------|---------|---------|
| `term.` | Localized labels | `term.page-label` |
| `role.<role>.<form>` | Contributor role phrases | `role.editor.label`, `role.editor.verb` |
| `pattern.` | Compositional phrase templates | `pattern.page-range`, `pattern.retrieved-from` |
| `date.` | Date-specific terms not in `dateFormats` | `date.open-ended` |

Legacy CSL-style term keys (`page`, `et_al`, `no_date`, …) remain accessible
via the `legacyTermAliases` map on `LocalePreset`, which redirects old keys to
new message IDs.

---

### 3. LocalePreset Schema

A `LocalePreset` is a structured YAML file in `locales/`. It supersedes the v1
flat term file but carries the same terms in the `messages` block, plus
`dateFormats`, `numberFormats`, `grammarOptions`, and `legacyTermAliases`.

**YAML fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `locale-schema-version` | `string` | yes (v2) | `"2"` selects the new parser path. Controls file format, not evaluation. |
| `locale` | `string` | yes | BCP 47 locale tag (`en-US`). |
| `language` | `string` | yes | ISO 639-1 language code. |
| `region` | `string` | yes | ISO 3166-1 region code. |
| `fallback` | `string\|null` | no | Locale ID to try when a message ID is missing. |
| `version` | `string` | no | Semver of this locale file. |
| `evaluation` | `object` | no | Runtime evaluation options. See §3.4. |
| `messages` | `map<string, string>` | yes | Message ID → message body (static or MF1 syntax). |
| `date-formats` | `map<string, string>` | yes | Symbolic name → CLDR date pattern. |
| `number-formats` | `object` | yes | `decimal-separator`, `thousands-separator`, `minimum-digits`. |
| `grammar-options` | `object` | yes | See §3.3. |
| `legacy-term-aliases` | `map<string, string>` | yes | Old key → new message ID. |

#### 3.4 `evaluation` Block

Controls runtime message evaluation. All fields are optional; defaults reflect
the Phase 0 behaviour.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `message-syntax` | `string` | `"static"` | Message format in use: `static`, `mf1`, or `mf2`. |

`message-syntax: static` — all messages are plain text; parameterized syntax
is not evaluated (silently skipped). Safe default for v2 files that have not
yet been audited for evaluator readiness.

`message-syntax: mf1` — ICU MF1 evaluation is active (requires Phase 4
`IcuMf1MessageEvaluator`). Before Phase 4, the engine treats this identically
to `static` for graceful degradation.

`message-syntax: mf2` — reserved for future MF2 engine. Treated as `static`
until `IcuMf2MessageEvaluator` is registered.

The `evaluation` block may grow with additional fields (e.g. custom function
declarations, evaluator hints) without breaking existing locale files.

#### 3.1 `en-US` Example

```yaml
locale-schema-version: "2"
locale: en-US
language: en
region: US
fallback: null
version: "2.0.0"

evaluation:
  message-syntax: mf1

messages:
  # Locator labels
  term.page-label:      "{count, plural, one {p.} other {pp.}}"
  term.page-label-long: "{count, plural, one {page} other {pages}}"
  term.chapter-label:   "{count, plural, one {chap.} other {chaps.}}"
  term.volume-label:    "{count, plural, one {vol.} other {vols.}}"
  term.section-label:   "{count, plural, one {sec.} other {secs.}}"
  term.figure-label:    "{count, plural, one {fig.} other {figs.}}"

  # Conjunctions and connectors
  term.and:        "and"
  term.and-symbol: "&"
  term.et-al:      "et al."
  term.and-others: "and others"

  # Date and access terms
  term.accessed:    "accessed"
  term.retrieved:   "retrieved"
  term.no-date:     "n.d."
  term.no-date-long: "no date"
  term.forthcoming: "forthcoming"
  term.circa:       "ca."
  term.circa-long:  "circa"

  # Role labels
  role.editor.label:       "{count, plural, one {ed.} other {eds.}}"
  role.editor.label-long:  "{count, plural, one {editor} other {editors}}"
  role.editor.verb:        "edited by"
  role.translator.label:   "trans."
  role.translator.label-long: "{count, plural, one {translator} other {translators}}"
  role.translator.verb:    "translated by"

  # Compositional patterns
  pattern.page-range:      "{start}\u2013{end}"
  pattern.retrieved-from:  "retrieved from {url}"
  pattern.available-at:    "available at {url}"
  pattern.n-authors-et-al: "{mainList}, et al."

  # Date terms
  date.open-ended: "present"

date-formats:
  numeric-short:  "M/d/yyyy"
  textual-long:   "MMMM yyyy"
  textual-full:   "MMMM d, yyyy"
  bib-default:    "MMMM d, yyyy"
  year-only:      "yyyy"
  iso:            "yyyy-MM-dd"

number-formats:
  decimal-separator:   "."
  thousands-separator: ","
  minimum-digits:      1

grammar-options:
  punctuation-in-quote: true
  nbsp-before-colon:    false
  open-quote:           "\u201C"
  close-quote:          "\u201D"
  open-inner-quote:     "\u2018"
  close-inner-quote:    "\u2019"
  serial-comma:         true
  page-range-delimiter: "\u2013"

legacy-term-aliases:
  page:          term.page-label
  pages:         term.page-label
  et_al:         term.et-al
  and:           term.and
  accessed:      term.accessed
  retrieved:     term.retrieved
  no_date:       term.no-date
  "no date":     term.no-date
  editor:        role.editor.label
  translator:    role.translator.label
```

#### 3.2 `de-DE` Example (contrast)

```yaml
locale-schema-version: "2"
locale: de-DE
language: de
region: DE
fallback: null
version: "2.0.0"

evaluation:
  message-syntax: mf1

messages:
  term.page-label:      "{count, plural, one {S.} other {S.}}"
  term.page-label-long: "{count, plural, one {Seite} other {Seiten}}"
  term.and:             "und"
  term.et-al:           "u.\u00A0a."
  term.no-date:         "o.\u00A0J."
  term.accessed:        "Zugriff am"
  term.retrieved:       "abgerufen von"

  role.editor.label:    "{count, plural, one {Hrsg.} other {Hrsg.}}"
  role.editor.label-long: "{count, plural, one {Herausgeber} other {Herausgeber}}"
  role.editor.verb:     "herausgegeben von"

  pattern.retrieved-from: "abgerufen von {url}"
  date.open-ended:      "heute"

date-formats:
  numeric-short:  "d.M.yyyy"
  textual-long:   "MMMM yyyy"
  textual-full:   "d. MMMM yyyy"
  bib-default:    "d. MMMM yyyy"
  year-only:      "yyyy"
  iso:            "yyyy-MM-dd"

number-formats:
  decimal-separator:   ","
  thousands-separator: "."
  minimum-digits:      1

grammar-options:
  punctuation-in-quote: false
  nbsp-before-colon:    false
  open-quote:           "\u201E"
  close-quote:          "\u201C"
  open-inner-quote:     "\u201A"
  close-inner-quote:    "\u2018"
  serial-comma:         false
  page-range-delimiter: "\u2013"
```

The same message IDs (`term.page-label`, `role.editor.verb`) carry
language-specific realizations without any change to the style YAML.

#### 3.3 `grammar-options` Fields

| Field | Type | Description |
|-------|------|-------------|
| `punctuation-in-quote` | `bool` | Period/comma goes inside closing quote mark. |
| `nbsp-before-colon` | `bool` | Non-breaking space before `:` and `?` (French typographic style). |
| `open-quote` / `close-quote` | `string` | Outer quotation marks. |
| `open-inner-quote` / `close-inner-quote` | `string` | Nested quotation marks. |
| `serial-comma` | `bool` | Oxford comma before final list conjunction. |
| `page-range-delimiter` | `string` | Default separator between page range endpoints. |

---

### 4. LocaleOverride

A `LocaleOverride` is a partial `LocalePreset` that patches a named base
locale for a specific style or document. It lives in `locales/overrides/`.

**YAML fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | `string` | Unique identifier for this override. |
| `baseLocale` | `string` | Locale ID of the base `LocalePreset`. |
| `messages` | `map` | Partial — only keys to override. |
| `dateFormats` | `map` | Partial — only keys to override. |
| `grammarOptions` | `map` | Partial — only keys to override. |

**Merge semantics:** shallow key-level replacement. For each map
(`messages`, `dateFormats`, `grammarOptions`), keys present in the override
replace matching keys in the base. Keys absent from the override retain their
base values. There is no deep-merge or list-append behavior.

**Example — Chicago English:**

```yaml
# locales/overrides/en-US-chicago.yaml
id: en-US-chicago
baseLocale: en-US

messages:
  term.page-label: "{count, plural, one {p.} other {pp.}}"
  term.no-date:    "n.d."

dateFormats:
  bib-default: "MMMM d, yyyy"

grammarOptions:
  serialComma: true
```

**Example — Turabian German:**

A style like "Turabian (German)" is represented as:
- Base `LocalePreset`: `de-DE`
- Override `de-DE-turabian`: overrides `bib-default` date format and a few
  label terms specific to Turabian conventions.

```yaml
# locales/overrides/de-DE-turabian.yaml
id: de-DE-turabian
baseLocale: de-DE

messages:
  role.editor.verb: "hg. von"

dateFormats:
  bib-default: "d. MMMM yyyy"
```

---

### 5. StylePreset Interaction

Styles reference **message IDs**, not raw strings.

#### 5.1 Term References

Old style YAML used bare CSL term keys:

```yaml
# v1 (deprecated)
term: page
term: et_al
term: editor
```

New style YAML uses message IDs:

```yaml
# v2
term: term.page-label
term: term.et-al
term: role.editor.label
```

The `legacyTermAliases` map in `LocalePreset` means the engine accepts both
forms; however, style authors should migrate to message IDs over time. The
`citum style lint` command will warn on legacy key usage after a grace period.

#### 5.2 Date Format Presets

Styles reference named date formats by symbolic key, never by pattern:

```yaml
options:
  dates:
    defaultFormat: bib-default
    issuedFormat:  textual-long
```

The engine resolves `bib-default` → `dateFormats["bib-default"]` in the active
`LocalePreset` (after applying any `LocaleOverride`), then passes the pattern
to the EDTF date formatter.

#### 5.3 Variable Declarations (for lint)

A style may declare expected variables at each message call site via an `args`
block. This is optional at runtime but required for `citum style lint` to
validate variable completeness:

```yaml
- term: term.page-label
  args:
    count: locator-count   # binds "count" to the engine's locator count value
```

If `args` is absent, the engine passes the full `MessageArgs` struct and the
message uses what it needs.

---

### 6. Engine Runtime Composition

Rendering pipeline per citation:

1. Resolve the active `LocalePreset` from `options.defaultLocale` in the style
   or from the request-level locale parameter.
2. If a `LocaleOverride` is specified (`options.localeOverride`), apply it via
   `Locale::apply_override()` to produce a merged view. The original
   `LocalePreset` is not mutated.
3. For each template component that calls a message ID:
   - Build `MessageArgs` from engine state (`count`, `names`, `gender`, etc.).
   - Call `MessageEvaluator::eval(id, &args)`.
   - For plural constructs: call `icu_plurals::PluralRules::category(count)`
     for the active locale to obtain the CLDR category string (`"one"`,
     `"other"`, etc.), then select the matching arm of the MF1 message.
   - For date variables: pre-format the date using the resolved `dateFormats`
     pattern, then substitute as a plain string.
   - Return the resolved `String`.

---

### 7. Rust Design

#### 7.1 `RawLocale` Additions (`locale/raw.rs`)

```rust
pub struct RawLocale {
    // existing fields unchanged …

    /// Schema version. Absent means v1 (legacy). "2" activates new paths.
    pub locale_schema_version: Option<String>,

    /// ICU MF1 messages keyed by message ID.
    #[serde(default)]
    pub messages: HashMap<String, String>,

    /// Named date format presets (symbolic name → CLDR pattern).
    #[serde(default)]
    pub date_formats: HashMap<String, String>,

    /// Locale-level number formatting options.
    pub number_formats: Option<RawNumberFormats>,

    /// Grammar toggles that vary by language.
    pub grammar_options: Option<RawGrammarOptions>,

    /// Backwards-compatibility aliases: old term key → new message ID.
    #[serde(default)]
    pub legacy_term_aliases: HashMap<String, String>,
}

pub struct RawNumberFormats {
    pub decimal_separator: String,
    pub thousands_separator: String,
    pub minimum_digits: u8,
}

pub struct RawGrammarOptions {
    pub punctuation_in_quote: Option<bool>,
    pub nbsp_before_colon: Option<bool>,
    pub open_quote: Option<String>,
    pub close_quote: Option<String>,
    pub open_inner_quote: Option<String>,
    pub close_inner_quote: Option<String>,
    pub serial_comma: Option<bool>,
    pub page_range_delimiter: Option<String>,
}
```

#### 7.2 `Locale` Additions (`locale/mod.rs`)

```rust
pub struct Locale {
    // existing fields unchanged …

    /// Parsed and compiled MF1 messages. Populated only for v2 locales.
    pub messages: HashMap<String, CompiledMessage>,

    /// Named date format presets.
    pub date_formats: HashMap<String, String>,

    /// Number formatting options.
    pub number_formats: NumberFormats,

    /// Grammar options.
    pub grammar_options: GrammarOptions,

    /// Legacy alias map for backwards-compatible term lookup.
    pub legacy_term_aliases: HashMap<String, String>,
}
```

`Locale::general_term(key)` lookup order:
1. Check `self.messages` for the key directly.
2. Check `self.legacy_term_aliases` to resolve key → message ID, then check
   `self.messages`.
3. Fall back to the existing legacy `terms` map (`HashMap<String, String>`).

`Locale::apply_override(override: &LocaleOverride) -> Locale` returns a new
`Locale` with shallow-merged maps. The original is not mutated.

#### 7.3 `MessageEvaluator` (`citum-engine/src/values/message.rs`)

```rust
/// Evaluates an ICU MF1 message for a given locale and argument set.
pub trait MessageEvaluator {
    fn eval(&self, id: &str, args: &MessageArgs) -> Result<String, MessageError>;
}

/// Runtime inputs for message evaluation.
pub struct MessageArgs {
    /// Count for plural dispatch.
    pub count: Option<u64>,
    /// Pre-formatted string value (e.g. a name list or URL).
    pub value: Option<String>,
    /// Grammatical gender for select dispatch.
    pub gender: Option<GrammaticalGender>,
    /// Pre-formatted name list string.
    pub names: Option<String>,
    /// Start of a range (e.g. page range start).
    pub start: Option<String>,
    /// End of a range.
    pub end: Option<String>,
    /// URL string.
    pub url: Option<String>,
    /// Pre-formatted date string.
    pub date: Option<String>,
    /// Main contributor list for "et al." patterns.
    pub main_list: Option<String>,
}

pub enum GrammaticalGender {
    Masculine,
    Feminine,
    Neuter,
    Other,
}

pub enum MessageError {
    /// No message found for this ID in the active locale.
    MissingMessage(String),
    /// A variable referenced in the message body was not provided.
    MissingVariable { message_id: String, variable: String },
    /// The message body failed to parse as valid MF1.
    ParseError { message_id: String, detail: String },
}
```

The `IcuMessageEvaluator` struct implements `MessageEvaluator`:
- Holds a reference to `Arc<Locale>` (already shared for concurrent rendering).
- Parses all `messages` at locale-load time into a `HashMap<String, CompiledMessage>`.
- For plural: delegates category computation to
  `icu_plurals::PluralRules` initialized with the locale's BCP 47 tag.
- For select: direct string-keyed dispatch, no CLDR dependency.
- Available behind Cargo feature `icu-messages` (enabled by default in
  `citum-engine`).

---

### 8. Migration Strategy

#### Phase 0 — Compatibility layer (no breaking changes)

- Add `locale_schema_version` (optional, defaults to `"1"` if absent) to
  `RawLocale`.
- Add `messages`, `date_formats`, `grammar_options`, `legacy_term_aliases` to
  `RawLocale` with `#[serde(default)]`. These fields are populated only when
  the YAML contains them.
- Keep all existing `terms`, `roles`, `locators` parsing unchanged.
- Update `Locale::general_term()`, `locator_term()`, `role_term()` to check
  `messages` first, fall back to legacy maps. Net effect: zero regression on
  existing v1 locale files and all oracle tests.

#### Phase 1 — Convert high-impact terms

Convert these term keys to ICU messages in `en-US.yaml` and `de-DE.yaml`:

- All locator labels: `page`, `chapter`, `volume`, `section`, `figure`, `note`.
- Role labels: `editor`, `translator`, `director`, `compiler`.
- Connectors: `et_al`, `and`, `no_date`, `accessed`, `retrieved`.

Provide `scripts/migrate-locale-v1-to-v2.js`:

- Reads a v1 locale YAML.
- Converts every `singular`/`plural` pair to
  `{count, plural, one{…} other{…}}`.
- Emits a v2 YAML with `localeSchemaVersion: "2"` and a populated
  `legacyTermAliases` block.

#### Phase 2 — `dateFormats` and `grammarOptions`

- Add `dateFormats` map to locale files, mapping the existing `DatePreset`
  enum names (`Long`, `Short`, `Numeric`, `Iso`) plus new symbolic names
  (`bib-default`, `textual-long`) to CLDR pattern strings.
- Move `punctuation_in_quote` and related config from hard-coded Rust structs
  into `grammarOptions` in locale YAML.

#### Phase 3 — `LocaleOverride` mechanism

- Define `LocaleOverride` struct and `Locale::apply_override()`.
- Create `locales/overrides/` directory with `en-US-chicago.yaml` as the
  first example.
- Add `options.localeOverride` field to style YAML schema.

#### Phase 4 — Validation tooling

- `citum locale lint <file>`: parse locale YAML, validate MF1 syntax in all
  messages, report undeclared variables, check `legacyTermAliases` targets
  exist.
- `citum style lint <style.yaml> --locale <locale.yaml>`: validate all message
  IDs referenced in the style exist in the locale, and that declared `args`
  variables match the message's variable set.

---

### 9. Performance Notes

- **Parse once.** ICU message strings are parsed at `LocalePreset` load time
  into `CompiledMessage` trees stored in a `HashMap`. Per-citation evaluation
  operates on compiled trees only.
- **Shared locale.** `Locale` (and its compiled message map) is wrapped in
  `Arc<Locale>` and shared across concurrent rendering threads. No clone per
  citation.
- **CLDR plural rules.** `icu_plurals::PluralRules` is initialized once per
  `(locale_id, PluralRuleType::Cardinal)` pair and held on the evaluator.
  Not reconstructed per citation.
- **CBOR path.** The existing CBOR serialization path for `Locale` should be
  extended to include compiled messages for zero-parse cold starts in embedded
  or WASM scenarios.

---

### 10. Error Handling and Fallback

| Condition | Behavior |
|-----------|----------|
| Message ID not found in active locale | Check `legacyTermAliases`; if still missing, try `fallback` locale; if still missing, return bare ID as plain text and emit `warn!`. |
| ICU plural categories incomplete | Use `"other"` as universal fallback (consistent with CLDR recommendation). |
| MF1 parse error in locale YAML | Hard error at locale-load time; engine falls back to `en-US`. Do not fail silently at render time. |
| Missing variable in `MessageArgs` | Substitute empty string; emit `warn!` with message ID and variable name. |
| `LocaleOverride.baseLocale` not found | Emit error; treat override as standalone locale rather than crashing. |

---

### 11. Versioning

Two version fields are introduced:

- `localeSchemaVersion: "2"` on locale YAML — controls the parser path.
- Style schema versioning (`STYLE_SCHEMA_VERSION`) is separate and unchanged
  by this spec.

Engine behavior by `localeSchemaVersion`:

| Value | Engine behavior |
|-------|-----------------|
| absent or `"1"` | Legacy path: `terms` + `roles` + `locators` maps only. |
| `"2"` | New path: `messages` + `dateFormats` + `grammarOptions`; legacy maps also loaded for alias lookup. |

---

## Implementation Notes

- `icu_plurals` is the correct crate (part of `icu4x`), not `icu_messageformat_parser`.
  The latter parses MF1 ASTs; we need a minimal MF1 evaluator plus a CLDR
  plural-rules engine. Evaluate whether `icu_messageformat_parser` + `icu_plurals`
  together cover the full evaluation path, or whether a small custom MF1
  interpreter over `icu_plurals` is sufficient for Citum's constrained message
  vocabulary.
- The existing `DatePreset` enum in `presets.rs` (`Long`, `Short`, `Numeric`,
  `Iso`) should be supplemented — not replaced — by the locale-bound
  `dateFormats` map. Styles using the old enum names continue to work via
  `legacyTermAliases`-style resolution until a future deprecation pass.
- `de-DE.yaml` currently encodes non-breaking spaces as `&#160;` HTML entities.
  The v2 migration script should normalize these to `\u00A0` Unicode escapes
  or direct UTF-8 characters for consistency.

---

## Acceptance Criteria

- [ ] `en-US.yaml` with `localeSchemaVersion: "2"` parses without error.
- [ ] `de-DE.yaml` with `localeSchemaVersion: "2"` parses without error.
- [ ] `MessageEvaluator::eval("term.page-label", {count: 1})` returns `"p."` for `en-US`.
- [ ] `MessageEvaluator::eval("term.page-label", {count: 2})` returns `"pp."` for `en-US`.
- [ ] `MessageEvaluator::eval("term.page-label", {count: 1})` returns `"S."` for `de-DE`.
- [ ] A v1 locale file (no `localeSchemaVersion`) loads via legacy path with zero
      regression on all existing citation oracle tests.
- [ ] `LocaleOverride` merge replaces only the specified keys; unspecified keys
      retain their base locale values.
- [ ] `dateFormats["bib-default"]` in `en-US` resolves to a concrete CLDR pattern
      used by the EDTF date renderer.
- [ ] `citum locale lint` reports a hard error for malformed MF1 syntax in a
      message value.
- [ ] `citum locale lint` reports a warning when a message ID referenced in a
      style is absent from the locale file.
- [ ] Rendering benchmark shows no per-citation regression versus the v1 baseline
      (parse-once guarantee verified by profiling).

---

## Changelog

- v1.1 (2026-03-18): Add §1.4 message taxonomy (static vs parameterized),
  §1.5 MF2 forward-compatibility design (custom formatters, trait boundary,
  migration path), `evaluation` block with `message-syntax` field on
  `LocalePreset`. Fix all YAML example keys to kebab-case.
- v1.0 (2026-03-18): Initial spec draft.
