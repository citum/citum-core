# Title Text-Case Semantics Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-03-11
**Supersedes:** none
**Related:** `csl26-wv5o`, `csl26-suz3`, `docs/architecture/DJOT_RICH_TEXT.md`

## Purpose

Defines how Citum should model and apply title-like text-case transformations
for bibliography and citation output. The goal is to make title rendering
style-correct, language-aware, and non-destructive while preserving the bounded
Djot title-markup work already landed in `csl26-suz3`.

## Scope

**In scope:**
- Title-like fields whose rendered casing varies by style or context
- Sentence-case and title-case semantics for English-focused bibliographic rules
- Case-protection semantics equivalent to CSL `.nocase` and BibTeX braces
- Interaction between casing, field language, mixed-language spans, and
  rich-text markup
- Style-owned preset behavior for casing variants such as APA-like and
  NLM-like sentence case
- Explicit normalization workflows that run outside normal rendering

**Out of scope:**
- Reopening the bounded Djot title-markup work from `csl26-suz3`
- Full NLP-based detection of proper nouns or title boundaries
- General non-title casing behavior for arbitrary fields
- Final schema naming for every new field or enum variant
- Shipping implementation in this spec

## Design

### Core Principles

1. Title casing must be non-destructive by default.
2. Sentence case is the safest canonical storage assumption for portable
   bibliographic data, following CSL experience.
3. Style-specific casing rules belong primarily in authored presets, not in
   hard-coded engine branches.
4. Casing transforms must operate on structured rich text, not flat strings.
5. Language metadata must constrain casing behavior; if Citum lacks a defined
   algorithm for a language, it must prefer `as-is` behavior over guessing.

### Title-Like Field Model

Citum should treat title-like fields as structured content with explicit title
parts and explicit rich-text spans.

Normative rules:

- Rendering must use Citum's structured title model, not parse full title
  strings to recover main-title or subtitle boundaries.
- Multiple subtitles are first-class data, not an edge case.
- Parsing punctuation such as `:` or em dash is acceptable only for legacy
  import, migration, or fallback display of unstructured external data. It is
  not the normative rendering model.

Structured title content includes:

- field-level language metadata
- explicit main-title and subtitle structure
- optional span-level language overrides
- generic case-protection metadata on spans
- optional semantic roles for spans, such as embedded title, acronym, formula,
  translation, or quoted title

The existing rich-text direction from `csl26-suz3` is the correct foundation.
Case transforms must run over the rich-text tree so they can skip protected
spans while preserving surrounding italics, quotes, links, and affixes.

### Case Patterns and Variants

Citum must model text-case transforms as explicit variants rather than a single
global "sentence case" or "title case" mode.

Required initial patterns:

- `title`
- `sentence`
- `capitalize-first`
- `lowercase`
- `uppercase`
- `as-is`

Required initial sentence-case variants:

- `sentence-apa`
- `sentence-nlm`

The distinction is normative. At minimum:

- `sentence-apa` capitalizes the first word and the first word after subtitle
  boundaries used by APA-like styles.
- `sentence-nlm` capitalizes the first word only, except where the source text
  or protected spans preserve additional capitals.

Title-case variants may later need additional style presets, but that is not a
blocker for this spec because the strongest current disagreement is in sentence
case, not headline-style title case.

### Subtitle Boundaries

Subtitle behavior is style-sensitive and must not be collapsed into one generic
sentence-case rule.

For rendering purposes, Citum must support subtitle-boundary-aware casing rules.
When the title is structured, these rules apply across explicit subtitle parts,
not across a reparsed flat string. For legacy fallback only, the engine may
recognize at least these punctuation boundaries:

- `:`
- em dash
- question mark
- exclamation point

Semicolon does not create a subtitle boundary by default.

### Case Protection

Citum must define an internal case-protection concept that all case transforms
respect. This internal concept must be able to ingest:

- Djot-authored protected spans
- CSL `.nocase` spans
- BibTeX or biblatex brace protection

Case protection and semantic annotation are distinct concerns:

- **Generic protected spans** answer "do not recase this content."
- **Semantic spans** answer "what kind of content is this."

Every semantic span may imply protection, but the two concepts must not be
collapsed into one flag. For example, an embedded title span may carry semantic
meaning even when it is not fully locked, while a plain protected acronym may
need no richer semantic classification.

Protected spans are mandatory for preserving content whose lettercase carries
semantic meaning, including:

- acronyms and initialisms
- mixed-case brand or product names
- gene, protein, and chemical notation
- taxonomic names when authored with case-sensitive structure
- embedded titles or quoted material that should not be recased by the outer
  transform

Transforms must never modify a protected span. Semantic span roles may inform
future behavior, but generic protection is the minimum contract every casing
algorithm must honor.

### Automatic Protection Heuristics

Citum may add a small safety-net layer of automatic protection heuristics for
obvious patterns such as all-caps acronyms, formulas, or mixed-case tokens with
internal uppercase.

These heuristics are advisory, not authoritative:

- they may protect a token
- they must not rewrite a token
- they must not replace explicit authored protection

Heuristics exist to reduce accidental damage, not to infer full semantics.

### Language Behavior

Field-level language is required input to the casing engine.

Normative rules:

- English title/sentence transforms are in scope for engine-defined behavior.
- Mixed-language titles must allow span-level language overrides.
- For languages without defined casing semantics, Citum must default to
  `as-is` behavior rather than applying English transforms.
- Translated titles and transliterated titles must be treated as separate
  language-bearing content, whether stored as separate fields or separate spans.

This spec does not require immediate per-language casing algorithms beyond the
English-focused variants above.

### Input Casing and Normalization

Citum should assume sentence-case-oriented source data for portable bibliographic
workflows, while still preserving explicit authored capitals and protected spans.

Normative rules:

- Rendering may assume that unprotected title-like data is intended to behave as
  sentence-case-oriented source text unless a style or field explicitly says
  otherwise.
- Rendering must not destroy explicit authored capitals that survive within that
  sentence-case-oriented model.
- Title-case input is not the default portability assumption.
- Any database-wide or field-level normalization workflow must still be explicit
  and opt-in.

Normalization, if implemented later, belongs in a dedicated migration or data
cleanup tool, not in the default rendering path.

### Style and Preset Ownership

The engine provides generic case-transform primitives. Styles and presets own
which transform applies to which field in which context.

This includes:

- whether a field renders in title case, sentence case, or `as-is`
- which sentence-case variant applies
- which punctuation marks count as subtitle boundaries
- context-specific differences between bibliography, note, and in-text title
  rendering

This keeps the engine reusable while allowing style families to encode
Chicago-, APA-, MLA-, SBL-, IEEE-, or NLM-like behavior through data.

### Schema Direction

This spec intentionally fixes behavior before exact schema names. The following
capabilities are required, even if the final YAML or Rust surface differs:

- a style-level way to select case pattern and variant per field or field class
- a way to carry field language and optional span language
- a way to mark spans as case-protected independently of semantic span role
- a way to carry semantic span roles where they are useful
- a way to model explicit main-title and subtitle structure, including multiple
  subtitles
- a way to declare when a field intentionally deviates from the sentence-case
  default assumption

The future implementation spec may refine concrete enum names and YAML keys,
but it must preserve these semantics.

### Open Questions

The source memos agree on the non-destructive, rich-text, language-aware
direction. The remaining disagreements or unresolved choices are:

1. **How much automatic protection to infer**
   The research supports light heuristics for obvious acronyms and formulas, but
   the acceptable heuristic scope is still unsettled.

2. **How far to go beyond English**
   The memos agree that non-English behavior cannot safely reuse English rules,
   but they do not settle which non-English bibliographic casing systems should
   be first-class in the initial implementation.

3. **How much behavior semantic spans should unlock**
   This spec separates generic protection from semantic span roles, but the
   first implementation still needs to decide which semantic roles have active
   rendering consequences beyond simple case protection.

## Implementation Notes

- Build on the current Djot rich-text path instead of introducing a second
  string-only casing pipeline.
- Keep the first implementation narrow: title-like fields, English-focused
  sentence-case variants, explicit case protection, and style-owned presets.
- Do not make ICU or generic Unicode titlecasing the primary rule engine; it is
  only a possible low-level primitive for character conversion.
- Any follow-up implementation spec should define a conformance matrix covering:
  APA-like sentence case, NLM-like sentence case, title-case skip-word rules,
  protected scientific tokens, mixed-language spans, semantic vs generic
  protection, and title/subtitle
  boundaries.

## Acceptance Criteria

- [ ] Citum has a draft specification that treats title text-case behavior as a
      style-owned, non-destructive, rich-text-aware system
- [ ] The specification distinguishes at least `sentence-apa` and
      `sentence-nlm` as separate semantic variants
- [ ] The specification requires an internal case-protection mechanism
      compatible with Djot spans, CSL `.nocase`, and BibTeX-style protection
- [ ] The specification states that structured title parts, including multiple
      subtitles, are the normative rendering model
- [ ] The specification distinguishes generic protected spans from richer
      semantic span roles
- [ ] The specification adopts sentence case as the default portability
      assumption for title-like source data
- [ ] The specification requires field-level language awareness and span-level
      overrides for mixed-language titles
- [ ] The specification records unresolved choices in an explicit open-questions
      section rather than burying them in implementation details

## Changelog

- v1.0 (2026-03-11): Initial draft synthesized from `perplexity.md` and
  `gem.md`.
