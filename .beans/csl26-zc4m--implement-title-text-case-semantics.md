---
# csl26-zc4m
title: Implement title text-case semantics
status: todo
type: feature
priority: normal
created_at: 2026-03-11T21:30:00Z
updated_at: 2026-03-13T16:24:22Z
---

Follow-up to archived research bean `csl26-wv5o`.

Implement the first execution slice of the title text-case spec in
`docs/specs/TITLE_TEXT_CASE.md`.

## Spec

See `docs/specs/TITLE_TEXT_CASE.md`

## Scope

First implementation slice only:

- honor structured title parts (`main` + `sub`) as the normative rendering
  model; do not parse flat title strings during normal rendering
- add an internal case-protection concept for title rich text that can back
  Djot `.nocase`-style spans
- implement sentence-case variants for at least APA-like and NLM-like behavior
- wire style-owned field casing selection into title rendering
- preserve multilingual behavior through field language and span-level language
  overrides, with `as-is` fallback where no transform is defined

## Non-goals

- broad NLP-style proper noun inference
- full non-English casing transforms in the first slice
- solving every semantic span role beyond what is needed for case protection
- migration-wide normalization tooling in the same change

## Technical Direction

- Build on the rich-text path landed under `csl26-suz3`; do not add a second
  string-only title casing pipeline.
- Use structured titles directly from the existing `Title` model and
  `StructuredTitle { main, sub }`.
- Keep authored YAML lightweight; internal semantics may be richer than the
  authored surface.
- Treat sentence case as the default portability assumption, but preserve
  explicit authored capitals and protected spans.
- Evaluate the Rust `titlecase` crate only as low-level prior art or a helper
  for English headline-style logic; do not assume it can satisfy Citum's
  structured-title, `.nocase`, or sentence-variant requirements by itself.

## Todos

- [ ] Define the engine-facing case-protection representation for title spans
- [ ] Add style/schema support for selecting title text-case variants
- [ ] Implement structured-title-aware sentence casing for `sentence-apa`
- [ ] Implement structured-title-aware sentence casing for `sentence-nlm`
- [ ] Thread language metadata through the title case-transform path
- [ ] Add tests covering multiple subtitles, protected scientific tokens, and
      mixed-language title spans
- [ ] Document any intentionally deferred non-English transform behavior
- [ ] Evaluate whether the Rust `titlecase` crate is useful as a helper without
      making it the architectural center of the feature

## 2026-03-13
- crates/citum-engine/src/processor/rendering.rs: DRY (3× multilingual extraction → helper), concision (3 dead methods removed, leading_group_affix via rendering()), idioms (is_some_and, match over unwrap), perf (avoid clone in punctuation_in_quote). 2364→2208 lines.
- crates/citum-engine/src/processor/rendering.rs: extracted render_integral_explicit_group and render_special_type_items from 306-line render_grouped_citation_with_format; hoisted duplicate group_ids binding; applied then_some idiom
