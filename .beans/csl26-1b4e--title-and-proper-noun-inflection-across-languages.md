---
# csl26-1b4e
title: Title and proper-noun inflection across languages
status: draft
type: feature
priority: normal
tags:
    - multilingual
created_at: 2026-05-16T11:28:29Z
updated_at: 2026-05-16T12:48:56Z
---

## Goal

Address the broader cross-language ask in CSL upstream issue [#6369](https://github.com/citation-style-language/styles/issues/6369): grammatical inflection of **titles and proper nouns** in citations. `csl26-v6ok` solved this for date components; titles and names are a much larger problem and need design before code.

## Status

**Draft** — needs a design pass. Do not start implementation until the design note is written and approved.

## The problem

In inflecting languages (Basque, Finnish, Hungarian, Russian, …) a proper noun or book title changes form depending on its grammatical role in the surrounding sentence. Examples:

- Basque: a title can take genitive `-(r)en`, locative `-an`, etc. Basque is also agglutinative — multiple case markers can combine on one word — and uses an ergative–absolutive alignment that differs fundamentally from the nominative–accusative systems of the others.
- Finnish: rich case system (~15 cases including inessive, elative, illative, adessive, ablative, allative, …) applied to author surnames in narrative citations.
- Hungarian: similarly large case inventory; titles take case suffixes when embedded in running prose.
- Russian: surname declension by case (six cases including genitive, dative, instrumental).

Citum currently treats titles and names as opaque strings, so styles cannot ask for an inflected form even when one would be linguistically required.

## Prior art

Substantial prior art **does not exist**. A literature/tooling scan turned up:

- **CSL 1.0 / CSL-JSON**: no concept of grammatical case on titles or names; treats them as opaque strings. The upstream issue [citation-style-language/styles#6369](https://github.com/citation-style-language/styles/issues/6369) is exactly the report that this gap exists.
- **citeproc-js / citeproc-rs / citation.js**: inherit CSL's opaque-string model.
- **biblatex / biber**: ships rich author/title metadata but no per-case variants either; the closest thing is `nameaddon` and language-specific `\DeclareCaseCommand`s for sorting, not rendering case.
- **Zotero / better-bibtex**: no case-variant input fields.
- **Pandoc citeproc**: same as citeproc-js.
- **CLDR / ICU**: provides case-sensitive number/date formatters (e.g. ICU's `:case=genitive` annotation on `MessageFormat 2`) but no name/title declension data.

The closest in-repo precedent is **`MaybeGendered<T>`** in `crates/citum-schema-style/src/locale/types.rs`, which solves the analogous problem for *locale terms* (role labels by grammatical gender). The shape generalizes, but the domain is much smaller — locale terms are a closed authored set; titles and names are open input data.

This bean is therefore green-field. The design note should justify the choices on first principles rather than appeal to convention.

## Design questions

1. **Input model.** Do contributors and titles carry per-case variants in the input data (similar to `MaybeGendered<T>`)? Or does the engine perform morphological derivation? Citum has consistently chosen "explicit over magic" — favours input-side variants.
2. **Schema surface.** Likely a new `InflectedString` or `TitleForms` type with a base + case map. Needs to survive serde round-trip and the existing forward-compatibility contract.
3. **Style language.** How does a CSL-derived or Citum-native template request "the genitive form of this title"? A new template attribute? Implicit from surrounding context (locative wrapper, possessive macro)?
4. **Locale binding.** Different languages need different case sets — and the *kinds* of cases differ structurally (Basque's ergative–absolutive vs Finnish's nominative–accusative; Finnish's 15+ vs Russian's 6). Use a locale-declared list of recognized cases rather than a hardcoded universal vocabulary.
5. **Fallback behavior.** What should the renderer do when a style requests a genitive (or any other case) but only the nominative is provided? Options: silent fallback to the supplied form, fallback with a render-time warning, hard error. Probably language- or locale-policy-driven (Finnish text with an uninflected name is "wrong" in a way that English text with an unannotated nominative isn't).
6. **Migration impact.** Most styles need no change because most input data is in nominative; inflection becomes opt-in. Confirm with a concrete style + locale combination.

## Todo

- [x] Write initial design note: [`docs/specs/TITLE_NAME_INFLECTION.md`](../../docs/specs/TITLE_NAME_INFLECTION.md) (v0.1, Draft — bundled into the `csl26-v6ok` PR)
- [ ] Native-speaker review of the worked examples in the spec (Finnish, Russian, Basque entries; resolve the 🚧 markers)
- [ ] Resolve the open follow-up questions (§"Open questions for follow-up research" in the spec) — each gets an answer or its own child bean
- [ ] Record decisions for Q1–Q6 with one-paragraph rationale each
- [ ] Author a sample Finnish or Russian style against the proposed schema; render a realistic bibliography correctly
- [ ] Get sign-off from the user before implementation
- [ ] Decompose into implementation beans

## Related

- Spec: [`docs/specs/TITLE_NAME_INFLECTION.md`](../../docs/specs/TITLE_NAME_INFLECTION.md)
- Sibling deferred work from `csl26-v6ok` (dates were the easy subset)
- CSL upstream #6369
- `docs/specs/GENDERED_LOCALE_TERMS.md` is the closest precedent (`MaybeGendered<T>` shape)
