# Locale Messages Specification

**Status:** Active
**Version:** 1.3
**Date:** 2026-03-22
**Supersedes:** (none)
**Related:** bean `csl26-qrpo` (ICU4X upgrade)

## Purpose

Replace Citum's flat key-to-static-string locale model with a parameterized
message system based on Unicode MessageFormat 2 (MF2). This separates language
realization (words, inflection, punctuation, date and number formats) from
style structure (field order, conditions, what to omit), and enables
composition of `StylePreset × LocalePreset × LocaleOverride` instead of
duplicating styles per language.

## Scope

**In scope:**
- Schema additions to `RawLocale` and `Locale` for `messages`, `dateFormats`,
  `numberFormats`, `grammarOptions`, `legacyTermAliases`.
- `LocaleOverride` struct and merge semantics.
- `MessageEvaluator` trait and `Mf2MessageEvaluator` implementation.
- Migration compatibility layer: dual-path lookup and `localeSchemaVersion`
  gating.
- CLI lint tooling: `citum locale lint` and `citum style lint --locale`.
- Updated `en-US.yaml` and `de-DE.yaml` with v2 messages for high-impact terms.
- `scripts/migrate-locale-v1-to-v2.js` for automated v1 → v2 conversion.

**Out of scope:**
- Full Fluent (`.ftl`) syntax support.
- MF2 custom function annotations (`:citum-date`, `:citum-names`) — see §1.5.
- Full CLDR plural rules beyond `one`/`*` — see §1.3.
- Gender agreement for contributor name declension (tracked separately).
- Locale discovery or registry beyond the existing file-based `locales/`
  directory.
- Inline style-level locale overrides embedded directly inside style YAML.

---

## Design

### 1. Message Syntax: Unicode MessageFormat 2

Citum adopts **Unicode MessageFormat 2 (MF2)** as the canonical message syntax.
The implementation targets a minimal MF2 subset behind the `MessageEvaluator`
trait so the underlying evaluation engine can be swapped without changing locale
files or call sites. See §1.5 for the ICU4X migration path.

Rationale for MF2 over Fluent or MF1:

- **Finalized standard.** MF2 is a ratified Unicode Technical Standard (CLDR
  TR #35 §12). Unlike MF1, it will not change in backwards-incompatible ways.
- **YAML compatibility.** MF2 multi-line `.match` blocks serialize cleanly as
  YAML block scalars (`|`). Static strings require no delimiter changes.
- **ICU4X alignment.** ICU4X's `icu_message_format` targets MF2 natively.
  Adopting MF2 now means locale files require no changes when we upgrade to
  the ICU4X evaluator (see §1.5 and bean `csl26-qrpo`).
- **Interoperability.** MF2 is the future standard for Lokalise, Crowdin, and
  other i18n platforms.

**Supported MF2 subset** (v2 locales with `evaluation.message-syntax: mf2`):

| Construct | Syntax | Notes |
|-----------|--------|-------|
| Plain text | `"retrieved"` | No variables. |
| Variable substitution | `{$names}` | Named string substitution. |
| Plural | `.match {$count :plural}`<br>`when one {p.} when * {pp.}` | Two-value dispatch: `one` and `*` (wildcard). Full CLDR categories deferred to ICU4X. |
| Select | `.match {$gender :select}`<br>`when masculine {él} when * {elle}` | Arbitrary string-keyed dispatch with wildcard fallback. |
| Multi-selector role labels | `.match {$gender :select} {$count :plural}`<br>`when feminine one {editora} when * * {equipo editorial}` | Active scoped support for gender-aware role labels. Variant key count must match selector count. |

**Out of scope in current evaluator:**
- Full CLDR plural categories (`zero`, `two`, `few`, `many`) — only `one`/`*` supported.
- MF2 custom function annotations (`:citum-date`, `:citum-names`) — see §1.5.
- MF2 markup elements (`{#b}…{/b}`).
- General-purpose multi-selector `.match` beyond Citum's gender-aware role-label
  subset.

**Date formatting is not done inside MF2 messages.** The engine formats dates
using the `dateFormats` map and passes the result as a plain `{$date}` variable.
This is a deliberate deferral of custom formatter annotations (e.g.
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

**Parameterized messages** — MF2 syntax containing variable references or
selector constructs.

```yaml
term.page-label: |
  .match {$count :plural}
  when one {p.}
  when * {pp.}
pattern.page-range: "{$start}–{$end}"
```

Stored in the `messages` map and evaluated by `Mf2MessageEvaluator` when
`message-syntax: mf2` is set. Before the evaluator is wired in, the engine
silently falls back to the legacy term-map path for any message whose body
contains `{`. This fallback is intentional — static terms already cover
the rendering surface that Phase 0–3 exercises.

The distinction is runtime-only: both types use identical YAML representation
(a string value under a message ID key). The engine classifies a message as
parameterized if and only if its body contains a `{` character.

---

### 1.5 ICU4X Migration Path

MF2 is now active in Citum, but the current evaluator is a custom dependency-free
implementation covering Citum's narrow message vocabulary. When ICU4X's
`icu_message_format` crate stabilizes, we swap to a fully standard-conformant
evaluator — no locale files or call sites change. See bean `csl26-qrpo` and
[unicode-org/icu4x#3028](https://github.com/unicode-org/icu4x/issues/3028).

#### Abstraction boundary

The `MessageEvaluator` trait (§7.3) is the single seam between the rest of
Citum and any message format implementation. Call sites in `citum-engine` use
only the trait; they never depend on any format-specific AST. This means:

- Adding `IcuMf2MessageEvaluator` (ICU4X-backed) is additive — no call-site
  changes.
- The locale files already use MF2 syntax — they are unchanged by the swap.

#### `evaluation.message-syntax` dispatch

The `evaluation` block on each `LocalePreset` (§3) declares which syntax the
`messages` map uses. The engine selects the matching `MessageEvaluator`
implementation at locale-load time:

| `message-syntax` | Evaluator | Status |
|------------------|-----------|--------|
| `static` (default) | None — plain string return | Active |
| `mf2` | `Mf2MessageEvaluator` (custom) | **Active** |
| `mf2` (future) | `IcuMf2MessageEvaluator` (ICU4X) | Planned — bean `csl26-qrpo` |

#### `MessageArgs` as MF2 named variables

`MessageArgs` (§7.3) maps directly to MF2's named-variable model: each field
(`count`, `gender`, `names`, …) corresponds to a `$variable` in an MF2
message. No structural change to `MessageArgs` is needed when swapping to the
ICU4X evaluator.

MF2 multi-selector patterns for gender-aware role labels are accommodated by
`MessageArgs` having both `$gender` and `$count` fields — the added
expressiveness is in the message syntax, not in the Rust argument type. The
custom evaluator supports the scoped pattern used by locales:
`.match {$gender :select} {$count :plural}` with variants such as
`when feminine one {...}`, `when feminine * {...}`, and `when * * {...}`.

#### Custom formatters (deferred)

MF2 supports custom function annotations: `{$date :citum-date format=bib-default}`.
Currently Citum pre-formats dates: the engine computes the date string using
`dateFormats["bib-default"]` and passes it as a plain `{$date}` variable.

Pre-formatting keeps the formatting logic in typed Rust code with full EDTF
awareness and avoids a two-layer parsing problem. When the ICU4X evaluator is
adopted, `:citum-date` and `:citum-names` can be registered as custom functions
without changing any call sites or the `MessageArgs` shape.

The `dateFormats` map (§3) is the stable API regardless: both the pre-formatting
path and a future `:citum-date` formatter consume the same symbolic name → CLDR
pattern mapping.

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
| `messages` | `map<string, string>` | yes | Message ID → message body (static or MF2 syntax). |
| `date-formats` | `map<string, string>` | yes | Symbolic name → CLDR date pattern. |
| `number-formats` | `object` | yes | `decimal-separator`, `thousands-separator`, `minimum-digits`. |
| `grammar-options` | `object` | yes | See §3.3. |
| `legacy-term-aliases` | `map<string, string>` | yes | Old key → new message ID. |

#### 3.4 `evaluation` Block

Controls runtime message evaluation. All fields are optional; defaults reflect
the Phase 0 behaviour.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `message-syntax` | `string` | `"static"` | Message format in use: `static` or `mf2`. |

`message-syntax: static` — all messages are plain text; parameterized syntax
is not evaluated (silently skipped). Safe default for v2 files that have not
yet been audited for evaluator readiness.

`message-syntax: mf2` — MF2 evaluation is active via `Mf2MessageEvaluator`.
Supports variable substitution (`{$var}`), plural dispatch (`.match {$count :plural}`),
and select dispatch (`.match {$var :select}`). See §1 for the supported subset.

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
  message-syntax: mf2

messages:
  # Locator labels
  term.page-label: |
    .match {$count :plural}
    when one {p.}
    when * {pp.}
  term.page-label-long: |
    .match {$count :plural}
    when one {page}
    when * {pages}
  term.chapter-label: |
    .match {$count :plural}
    when one {chap.}
    when * {chaps.}
  term.volume-label: |
    .match {$count :plural}
    when one {vol.}
    when * {vols.}
  term.section-label: |
    .match {$count :plural}
    when one {sec.}
    when * {secs.}
  term.figure-label: |
    .match {$count :plural}
    when one {fig.}
    when * {figs.}

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
  role.editor.label: |
    .match {$count :plural}
    when one {ed.}
    when * {eds.}
  role.editor.label-long: |
    .match {$count :plural}
    when one {editor}
    when * {editors}
  role.editor.verb:        "edited by"
  role.translator.label:   "trans."
  role.translator.label-long: |
    .match {$count :plural}
    when one {translator}
    when * {translators}
  role.translator.verb:    "translated by"

  # Compositional patterns
  pattern.page-range:      "{$start}\u2013{$end}"
  pattern.retrieved-from:  "retrieved from {$url}"
  pattern.available-at:    "available at {$url}"
  pattern.n-authors-et-al: "{$main_list}, et al."

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
  message-syntax: mf2

messages:
  term.page-label: |
    .match {$count :plural}
    when one {S.}
    when * {S.}
  term.page-label-long: |
    .match {$count :plural}
    when one {Seite}
    when * {Seiten}
  term.and:             "und"
  term.et-al:           "u.\u00A0a."
  term.no-date:         "o.\u00A0J."
  term.accessed:        "Zugriff am"
  term.retrieved:       "abgerufen von"

  role.editor.label: |
    .match {$count :plural}
    when one {Hrsg.}
    when * {Hrsg.}
  role.editor.label-long: |
    .match {$count :plural}
    when one {Herausgeber}
    when * {Herausgeber}
  role.editor.verb:     "herausgegeben von"

  pattern.retrieved-from: "abgerufen von {$url}"
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
  term.page-label: |
    .match {$count :plural}
    when one {p.}
    when * {pp.}
  term.no-date:    "n.d."

dateFormats:
  bib-default: "MMMM d, yyyy"

grammarOptions:
  serialComma: true
```

**Example — Chicago German:**

A style like "Chicago Notes (German)" is represented as:
- Base `LocalePreset`: `de-DE`
- Override `de-DE-chicago`: encodes German-specific deviations like
  `delimiter-precedes-last: never` and specific verb forms.

```yaml
# locales/overrides/de-DE-chicago.yaml
id: de-DE-chicago
base-locale: de-DE

grammar-options:
  delimiter-precedes-last: never
  serial-comma: false

messages:
  role.editor.verb: "hg. von"
```

A style document references this override by its ID:

```yaml
# styles/chicago-author-date.yaml
options:
  default-locale: de-DE
  locale-override: de-DE-chicago
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
   - For plural constructs: match against `"one"` for count == 1, `"*"` otherwise.
     (Full CLDR plural categories deferred to ICU4X upgrade — see §1.5.)
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

    /// Parsed MF2 messages. Populated only for v2 locales.
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

#### 7.3 `MessageEvaluator` (`crates/citum-schema-style/src/locale/message.rs`)

The trait is the seam for ICU4X migration. Call sites in the engine depend only on
this trait; they never import format-specific code.

```rust
/// Arguments passed to message evaluation.
pub struct MessageArgs<'a> {
    pub count: Option<u64>,
    pub value: Option<&'a str>,
    pub gender: Option<&'a str>,
    pub names: Option<&'a str>,
    pub start: Option<&'a str>,
    pub end: Option<&'a str>,
    pub url: Option<&'a str>,
    pub date: Option<&'a str>,
    pub main_list: Option<&'a str>,
}

/// Evaluates a parameterized message string with runtime arguments.
///
/// Returns Some(result) on success, None on parse error or missing variables.
pub trait MessageEvaluator: Send + Sync {
    fn evaluate(&self, message: &str, args: &MessageArgs<'_>) -> Option<String>;
}
```

The `Mf2MessageEvaluator` struct implements `MessageEvaluator` with a custom
dependency-free evaluator:
- For plain text: returned as-is.
- For `{$var}` patterns: direct substitution from `MessageArgs` fields.
- For `.match {$var :plural}`: two-value dispatch (`one` vs `*`).
- For `.match {$var :select}`: string-keyed dispatch with wildcard fallback.
- Returns `None` on parse error or missing required variable (caller provides fallback).

The `Locale` struct holds `Arc<dyn MessageEvaluator>`, initialized at load time
based on `evaluation.message_syntax`:
- `Static` or unset → `Mf2MessageEvaluator` (parameterized evaluation skipped for
  messages lacking `{` — fast path)
- `Mf2` → `Mf2MessageEvaluator` (**current active evaluator**)
- Future: `IcuMf2MessageEvaluator` when ICU4X `icu_message_format` stabilizes
  (bean `csl26-qrpo`) — swap in one struct, no other changes

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

Convert these term keys to MF2 messages in `en-US.yaml` and `de-DE.yaml`:

- All locator labels: `page`, `chapter`, `volume`, `section`, `figure`, `note`.
- Role labels: `editor`, `translator`, `director`, `compiler`.
- Connectors: `et_al`, `and`, `no_date`, `accessed`, `retrieved`.

Provide `scripts/migrate-locale-v1-to-v2.js`:

- Reads a v1 locale YAML.
- Converts every `singular`/`plural` pair to MF2 block scalar:
  ```
  .match {$count :plural}
  when one {…}
  when * {…}
  ```
- Emits a v2 YAML with `localeSchemaVersion: "2"`, `evaluation.message-syntax: mf2`,
  and a populated `legacyTermAliases` block.

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

- `citum locale lint <file>`: parse locale YAML, validate MF2 syntax in all
  messages, report undeclared variables, check `legacyTermAliases` targets
  exist.
- `citum style lint <style.yaml> --locale <locale.yaml>`: validate all message
  IDs referenced in the style exist in the locale, and that declared `args`
  variables match the message's variable set.

---

### 9. Performance Notes

- **Evaluate from raw string.** The current `Mf2MessageEvaluator` evaluates
  directly from the raw message string on each call. There is no compile or
  parse-once step. For Citum's narrow, mostly-static message vocabulary this is
  acceptable. A parse-once optimisation (pre-compiling MF2 ASTs at locale-load
  time) is deferred to the ICU4X upgrade (bean `csl26-qrpo`), which would
  replace the evaluator entirely.
- **Shared locale.** `Locale` (and its message map) is wrapped in `Arc<Locale>`
  and shared across concurrent rendering threads. No clone per citation.
- **Simple plural dispatch.** The current `one`/`*` dispatch is a single integer
  comparison — zero allocation. Full CLDR rules (ICU4X) will be initialized once
  per `(locale_id, PluralRuleType::Cardinal)` pair at that time.
- **CBOR path.** The existing CBOR serialization path for `Locale` should be
  extended to include compiled messages for zero-parse cold starts in embedded
  or WASM scenarios.

---

### 10. Error Handling and Fallback

| Condition | Behavior |
|-----------|----------|
| Message ID not found in active locale | Check `legacyTermAliases`; if still missing, try `fallback` locale; if still missing, return bare ID as plain text and emit `warn!`. |
| ICU plural categories incomplete | Use `"other"` as universal fallback (consistent with CLDR recommendation). |
| MF2 parse / evaluation error | `MessageEvaluator::evaluate()` returns `None`. The call site (engine) provides the fallback — typically the legacy term-map value or the bare message ID. No panic, no hard error at render time. |
| Missing variable in `MessageArgs` | `evaluate()` returns `None`; the call site falls back to the legacy path or bare ID. |
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

- The current `Mf2MessageEvaluator` is a custom dependency-free implementation
  intentionally scoped to Citum's narrow message vocabulary. It does not attempt
  to implement the full MF2 spec. The `MessageEvaluator` trait ensures that
  upgrading to ICU4X's `icu_message_format` is a one-struct swap when it
  stabilizes (see §1.5 and bean `csl26-qrpo`).
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
- [ ] `citum locale lint` reports a hard error for malformed MF2 syntax in a
      message value.
- [ ] `citum locale lint` reports a warning when a message ID referenced in a
      style is absent from the locale file.
- [ ] Rendering benchmark shows no per-citation regression versus the v1 baseline
      (parse-once guarantee verified by profiling).

---

## Changelog

- v1.3 (2026-03-22): **Pivot to MF2.** Replace MF1 as the canonical message
  syntax with Unicode MessageFormat 2 (finalized standard). Implement custom
  dependency-free `Mf2MessageEvaluator` (no external crate — GPL-3.0 crates
  excluded). Supported subset: `{$var}` substitution, `.match {$count :plural}`
  (one/`*`), `.match {$var :select}` (arbitrary keys). Update all locale YAML
  examples to MF2 block-scalar syntax. Rewrite §1, §1.5 (ICU4X migration path),
  §3.4, §7.3. Bean csl26-jrr6 complete (archived). ICU4X swap tracked in
  bean csl26-qrpo / [unicode-org/icu4x#3028](https://github.com/unicode-org/icu4x/issues/3028).
- v1.2 (2026-03-22): **Status: Active**. Add `MessageEvaluator` trait and
  `message.rs` submodule. Update §7.3 with trait signature and load-time
  selection logic.
- v1.1 (2026-03-18): Add §1.4 message taxonomy (static vs parameterized),
  §1.5 MF2 forward-compatibility design (custom formatters, trait boundary,
  migration path), `evaluation` block with `message-syntax` field on
  `LocalePreset`. Fix all YAML example keys to kebab-case.
- v1.0 (2026-03-18): Initial spec draft.
