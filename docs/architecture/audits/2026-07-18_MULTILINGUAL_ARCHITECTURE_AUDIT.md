# Multilingual Architecture Audit

**Date:** 2026-07-18
**Scope:** Whole-system design review of multilingual support — data model,
locale system, style schema, and renderer — from the perspective of scholars,
editors, and style authors working outside English.
**Related:** [`MULTILINGUAL.md`](../../specs/MULTILINGUAL.md),
[`MULTILINGUAL_NAMES.md`](../../specs/MULTILINGUAL_NAMES.md),
[`MULTILINGUAL_SORTING.md`](../../specs/MULTILINGUAL_SORTING.md),
[`PUNCTUATION_NORMALIZATION.md`](../../specs/PUNCTUATION_NORMALIZATION.md),
[`CALENDAR_DATE_ANNOTATIONS.md`](../../specs/CALENDAR_DATE_ANNOTATIONS.md);
epic `csl26-0ugp` (follow-up work, §7)

## 1. Overall assessment

The design direction is fundamentally sound. Three deep layers are principled
and ahead of prior art:

- **The parallel-metadata data model** —
  `original`/`transliterations`/`translations` with BCP 47 keys,
  `field-languages`, `sort-as`. Purely descriptive, backward compatible,
  correctly separated from rendering policy.
- **The MF2-based locale message layer** — real plural and gender selectors
  (the Arabic gendered role labels express something CSL 1.0 cannot), plus a
  genuinely useful `grammar-options` typography block (quote characters,
  serial comma, subtitle delimiters, punctuation-collision policy).
- **The sorting architecture** — UCA/ICU4X collation, the
  `sort-as` → transliteration → original key chain, and script partitioning.

None of these need rethinking.

The weakness is concentrated in one layer: **punctuation and typography
realization**. Delimiters, wraps, and quotes enter the output as literal
characters — authored in style YAML, baked into `GrammarOptions::default()`,
or hardcoded in renderer assembly — and multilingual correctness is then
recovered by late, string-level rewrite passes (`remap_to_latin_punctuation`
running at three separately maintained insertion points). That is the "too
local, too procedural, too late" pattern.

The second, quieter problem is **mechanism/data asymmetry**: the machinery is
sophisticated, but the locale data for the languages that motivated it is
skeletal, and the fallbacks are silently English.

## 2. Biggest architectural risks and blind spots

### (a) String rewriting as the multilingual mechanism

The half-width remap (`remap_to_latin_punctuation`,
`crates/citum-engine/src/render/component.rs`) operates on already-rendered
text, must be replicated at every point where punctuation enters output
(component render, citation-cluster wrap, citation-spec wrap —
`MULTILINGUAL.md` §3.2a documents "why three insertion points" as a
coordination burden), and only works in one direction. The missing
half-width → full-width direction is what blocks the calendar-note feature
(`csl26-0kqf`); that work is the first increment of `csl26-k2kp`. Each new
script that opts in multiplies this. It works
for GB/T 7714; it will not compose. The mixed-script compound-citation edge
(first item's language stands in for the whole cluster) is a symptom of the
same shape (bean `csl26-p05x`).

### (b) Boolean script classification

`is_latin_script_language` (`crates/citum-engine/src/values/mod.rs`) is a
hand-maintained latin/not-latin bool built from language lists. The
positive-evidence policy — absent or unrecognized evidence never triggers
behavior — is exactly right; the *output type* is wrong. The moment Cyrillic
or Arabic punctuation conventions need modeling, a bool forces a redesign.
The right target is one resolution to an ISO 15924 script code (explicit
subtag first, then a CLDR likely-subtags-style lookup), threaded as a
first-class value (bean `csl26-30ga`).

### (c) English defaults leak silently through thin locales

`zh-CN.yaml` is 2.5K (vs 21K for en-US) with **no `grammar-options` and no
`date-formats`** — so a Chinese-locale rendering inherits English curly
quotes, `": "` delimiters, and falls back to what `date_patterns.rs`
documents as "the engine's hardcoded English assembly" for dates (no
`2020年5月` pattern is expressible today). `ar-AR` likewise has no
`grammar-options`. There are **no ja, ko, ru, or Indic locales at all** —
notable since Katakana name handling is the motivating case of
`MULTILINGUAL_NAMES.md`. Nothing lints locale completeness, so a Chinese
scholar experiences a silent quality cliff the architecture gives no warning
about (beans `csl26-itri`, `csl26-tfi8`).

### (d) No bidi/directionality model

The engine contains no LRM/RLM/FSI handling. An Arabic bibliography entry is
almost always mixed-direction (Arabic title + Latin DOI + Western page
numbers); in plain-text output, weak-directional punctuation between runs
will visually scramble. An Arabic or Hebrew scholar hits this immediately; an
English-first design never sees it (bean `csl26-uzkj`).

### (e) Case mapping is not locale-tailored

`text_case.rs` uses Rust's `to_uppercase()`/`to_lowercase()` (Unicode default
mappings). Turkish `i`↔`İ` / `ı`↔`I` is the classic casualty. The sting is
that Citum's own embedded `tr-TR` locale file is among the most complete in
the tree (schema-v2 structure, full MF2 message coverage) — locale-data
investment offers no protection against an engine-level casing gap.
Title-casing is correctly gated to English; lowercase/uppercase/sentence
transforms are the exposure (bean `csl26-11wh`).

### (f) No number-system model

`number-formats` covers separators but not digit systems. Arabic-Indic
(٠١٢), Persian, and Devanagari digits are unrepresentable; ordinals beyond
the MF2 `number.ordinal` message have no locale story (bean `csl26-5q59`).

### (g) Per-item term locale is coupled to template structure

A Chicago-style user citing German sources in an English document wants
"hrsg. von" for the German item and "edited by" for the English one
(biblatex's `autolang`, CSL-M's per-item `default-locale`). Citum can only do
this via `citation.locales[]` branches — swapping the *entire template* to
change the *term language*. Structure and language are different axes and
should not be bundled (bean `csl26-838l`).

### (h) The verification proxy is itself English-centric

Byte-parity with citeproc-js inherits citeproc-js's own blind spots — the
specs already caught this twice (the GB/T full-width leak, the missing
calendar annotations). Every non-Latin script needs some standard-derived
expectations, because the oracle systematically cannot flag the class of bug
this audit is about.

## 3. Concerns the engine should model explicitly

1. **Language & script identity** — effective language *and* effective script
   (ISO 15924), per item and per field. Currently semi-explicit (bool +
   ad-hoc lookups).
2. **Text variants & selection policy** — done well (modes, patterns,
   transliteration matching).
3. **Punctuation & typography realization** — delimiter/wrap glyph choice,
   width, spacing (French NBSP), quote characters. The weakest layer.
4. **Punctuation dynamics** — collision, movement, suppression. Well designed
   in `PUNCTUATION_NORMALIZATION.md`; implementation still partly scattered.
5. **Terms & messages** — MF2 layer, sound.
6. **Dates & calendars** — EDTF + era labels + opaque `calendar-note`;
   patterns per locale (data missing for CJK).
7. **Numbers** — digit systems, ordinals, range formats. Unmodeled.
8. **Name conventions** — ordering/separators exist for CJK; the same
   data-driven mechanism should eventually speak to Hungarian family-first,
   Spanish compound surnames, Icelandic given-name filing.
9. **Collation & partitioning** — sound.
10. **Directionality** — unmodeled.
11. **Case transformation** — needs locale tailoring.

## 4. Where each concern belongs

| Concern | Layer | Note |
|---|---|---|
| Language/script tags, variants, `sort-as`, `calendar-note` | Data model | Already right; keep it purely descriptive |
| Terms, messages, date patterns, quote chars, spacing, collision policy, digit system | Locale data | `grammar-options` is the right home and precedent; digit system is the missing field |
| *Which view* to render, opt-ins (sorting mode, partitioning, note-wrap), per-script overrides | Style schema | Styles express *intent* wherever a semantic name exists; literal glyphs remain the escape hatch |
| Effective-language/script resolution, realization of semantic delimiters into glyphs, bidi isolation, collation execution | Renderer | Renderer *resolves and realizes*; it does not *decide policy* |

The test to apply to every future feature: **the data model describes, the
locale supplies conventions, the style expresses intent, the renderer
resolves.** The remap pass fails this test (the renderer holds a hardcoded
four-character policy table); the punctuation-collision work passes it
(locale defaults, style overrides, renderer executes) — and is the best
internal precedent to copy.

## 5. The general abstraction: semantic delimiters, script realization

The current `scripts.latin.punctuation` remap is intent expressed backwards:
the style authors full-width literals for every item, and a rewrite pass
repairs the Latin ones afterward. The general abstraction inverts this:

**Punctuation flows through assembly as typed tokens with roles**
(`FieldSeparator`, `Wrap(Parentheses)`, `SubfieldDelimiter`, …) **and is
realized into glyphs late, by a table keyed on (role, effective script,
locale)** — CLDR-informed defaults, locale-overridable, style-overridable.
Authority runs downward: a style guide's own punctuation rules, expressed as
style overrides, outrank locale conventions, which outrank the CLDR-informed
engine defaults — CLDR informs the defaults, it never overrules a style.
GB/T 7714 then declares "wrap issue numbers in parentheses; separate with
commas" *once*, and each item's script yields `（），` or `(), ` respectively.
`CALENDAR_DATE_ANNOTATIONS.md` already states the principle — "the width of
the delimiters is not authored; it follows the item's script" — this makes it
the architecture rather than a per-feature carve-out.

This subsumes three currently separate things: the remap (becomes a
compatibility shim for literal-authored styles), `csl26-kneq` (full-width
realization falls out in the direction the remap cannot do), and the
`PUNCTUATION_NORMALIZATION.md` phase-3 pipeline, giving one ordered stage:
*assemble tokens → normalize (collision/movement policies) → realize
(script/locale glyphs) → format/escape*.

It is also the strongest answer to "does multilingual support help
monolingual users": the three-site quote/punctuation logic is a fragility the
punctuation spec already complains about for *English* styles. Typed tokens
kill the duplication, make quote-style changes safe, shrink style YAML, and
turn punctuation tests into policy tests instead of string surgery.
Multilingual correctness and English maintainability are the same refactor.

Normative design: [`PUNCTUATION_REALIZATION.md`](../../specs/PUNCTUATION_REALIZATION.md)
(bean `csl26-k2kp`).

## 6. Prioritized recommendations

**Fix now (cheap, high leverage, no redesign):**

1. Locale completeness lint + zh-CN/ar-AR typography fill (`csl26-itri`).
2. Unify script resolution to ISO 15924 (`csl26-30ga`).
3. Fix locale-blind case mapping (`csl26-11wh`).

**Redesign soon (before more scripts opt in):**

4. Punctuation realization layer (`csl26-k2kp`) — do this *before* extending
   the remap to Cyrillic or Arabic; every script added to the current
   mechanism raises the migration cost.
5. Per-item term localization (`csl26-838l`).
6. Bidi/RTL output spec (`csl26-uzkj`).
7. Digit-system field in locale `number-formats` (`csl26-5q59`).

**Leave alone:**

- The parallel-metadata model, `sort-as`/sorting chain, partitioning, MF2
  messages, disambiguation-on-rendered-strings — all sound.
- The opaque `calendar-note` decision — calendar conversion and era tables
  stay out of scope for now, not forbidden in principle; opaque text is the
  right boundary until concrete demand says otherwise.
- `citation.locales[]` layout branches — keep for genuinely structural
  per-language differences; they just should not remain the *only* per-item
  locale mechanism.

## 7. Follow-up beans

Epic `csl26-0ugp` — *Multilingual architecture hardening* — parents:

| Bean | Title |
|---|---|
| `csl26-30ga` | Unify effective-script resolution (ISO 15924) |
| `csl26-k2kp` | Punctuation realization layer: semantic delimiters |
| `csl26-itri` | Locale completeness lint + zh-CN/ar-AR typography fill |
| `csl26-tfi8` | Add ja-JP, ko-KR, ru-RU embedded locales |
| `csl26-uzkj` | Spec: bidi/RTL handling in rendered output |
| `csl26-11wh` | Locale-tailored case mapping (Turkish dotted i) |
| `csl26-5q59` | Digit-system localization in number-formats |
| `csl26-838l` | Per-item term localization (autolang) |
| `csl26-p05x` | Investigate mixed-script compound citations under shared wrap |

`csl26-k2kp` absorbs the script-aware wrap rendering work (formerly tracked
as draft bean `csl26-kneq`) as its first increment; the calendar-annotation
feature (`csl26-0kqf`) depends on that increment alone, not on the full
layer.
