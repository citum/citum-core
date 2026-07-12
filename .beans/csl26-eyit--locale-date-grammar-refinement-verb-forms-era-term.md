---
# csl26-eyit
title: Locale date-grammar refinement (verb forms, era-term position)
status: todo
type: feature
priority: low
tags:
    - locale
    - multilingual
    - schema
    - dates
created_at: 2026-07-12T15:36:28Z
updated_at: 2026-07-12T18:15:00Z
parent: csl26-kcda
---

- Grammatical case variation for dates in narrative contexts (e.g. lv-LV
  locative vs nominative: "pēdējo reizi skatīts 2025. gada 25. maijā") —
  CSL schema#458. Verified on PR review: MF2's `:select` has no generic
  named-argument fallback (`determine_match_key`,
  crates/citum-schema-style/src/locale/message.rs:241-260, only resolves a
  hardcoded field list — count/value/gender/names/start/end/url/date/year/
  month/day/main_list) — a grammatical-case selector needs new engine
  wiring (a MessageArgs field), not just YAML authoring. There IS a
  closer-fit existing mechanism: suffix-in-pattern authoring
  (docs/guides/AUTHORING_LOCALES.md:111-150, used today for Basque
  genitive/absolutive month suffixes), which carries its own documented
  caveat: "if a locale ever needs more than one inflected form of the same
  month, file a follow-up before extending." Latvian's locative-vs-
  nominative case looks like exactly that caveat firing.
- Position parameter for era terms (AD/BC before vs after year, e.g.
  Chinese convention) — CSL schema#459. Confirmed NOT the same as
  style.json's DatePosition (which places the whole date element relative
  to author/title in a bibliography entry, not the era suffix relative to
  the year within the date itself). Also confirmed: era rendering today is
  hardcoded Rust (crates/citum-engine/src/values/date.rs, roughly
  `format!("{year} {era}")`), not routed through the MF2 evaluator the way
  month/day/year ordering already is.

- [ ] #458: follow the documented AUTHORING_LOCALES.md caveat — file the
      follow-up it asks for before extending suffix-pattern authoring to a
      second inflected form, rather than inventing a new mechanism
- [ ] #459: route era formatting through a new MF2 pattern (e.g.
      `pattern.date-year-era`), consistent with how month/day/year ordering
      is already MF2-routed, instead of a bespoke position field — needs
      engine wiring in citum-engine/src/values/date.rs either way
