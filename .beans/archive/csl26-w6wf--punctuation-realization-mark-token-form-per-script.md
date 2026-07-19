---
# csl26-w6wf
title: 'Punctuation realization: mark token form + per-script overrides'
status: completed
type: feature
priority: normal
tags:
    - multilingual
    - punctuation
    - architecture
created_at: 2026-07-19T16:29:49Z
updated_at: 2026-07-19T17:18:20Z
parent: csl26-0ugp
---

Increment 2 of the punctuation realization layer (spec §8), building on the
script-aware `WrapPunctuation` realization landed in increment 1 (`csl26-k2kp`,
merged). Adds:

- The `{ mark: <name> }` token form for `delimiter`/`prefix`/`suffix` fields
  (mapping form only — `delimiter: comma` must stay the literal text "comma",
  not be misread as a token; see spec §3).
- The engine default realization table (`comma`, `colon`, `semicolon`,
  `period`, `parentheses`, `brackets` — spec §2), keyed on (mark, script class).
- The per-script `realization` override:
  `options.multilingual.scripts.<S>.realization`, living alongside the
  existing `delimiter`/`sort-separator`/`use-native-ordering`/`punctuation`
  block in `ScriptConfig` (spec §4). Resolution order: style override → engine
  default table.
- Schema regeneration (`just schema-gen`) for the new token form and override
  field.

## Acceptance criteria (from spec)

- [x] `delimiter: { mark: comma }` renders `，` for CJK items and `, ` for
      Latin items; `delimiter: "comma"` renders the literal text "comma".
- [x] A per-script `realization` override replaces the engine default for
      exactly the overridden marks.
- [x] Literal punctuation in `prefix`/`suffix`/`delimiter` is never rewritten
      by the realization layer.
- [x] Realization output passes through output-format escaping (HTML, LaTeX,
      Typst, plain, Djot) unchanged in meaning.
- [x] Generated schemas include the token form and per-script `realization`;
      all new public Rust items documented.

Spec: `docs/specs/PUNCTUATION_REALIZATION.md` §8 (increment 2).
Prerequisite `ScriptClass`/`realize_wrap` infrastructure already landed in
`crates/citum-engine/src/values/mod.rs` and `render/format.rs` via `csl26-k2kp`.


## Summary

Implemented explicit semantic punctuation mark tokens and per-script realization overrides across schema, rendering, citation/group assembly, and all output formats. Existing scalar style values remain literal; existing enum-like YAML spellings were migrated to explicit tokens or literals. Regenerated public schemas and validated with `just pre-commit` (2,073 tests passed).
