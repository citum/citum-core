---
# csl26-c2um
title: 'migrate: titles double-quoted in migrated templates'
status: completed
type: bug
priority: normal
tags:
    - migrate
    - fidelity
created_at: 2026-06-14T11:20:25Z
updated_at: 2026-06-14T17:07:45Z
parent: csl26-vmcr
---

Several author-date/note styles render titles with doubled quotation marks, e.g. journal-of-advertising-research and early-medieval-europe produce '““Title””'. The converter wraps a title component in quotes that the engine/locale also applies (or wraps twice). Converter-level template defect. Repro: node scripts/oracle.js styles-legacy/journal-of-advertising-research.csl --json --force-migrate

## Summary of Changes

Root cause was a **double-application of quotes across two rendering layers**, not a single converter wrap:

1. **Engine (primary fix):** `render_component_with_format_and_renderer` applied both the `quote: true` flag (from the global `titles.*.quote` config layer) **and** a `wrap: { punctuation: quotes }` (from the template layer) to the same component, yielding `““Title””`. Guarded so the `quote` flag is skipped when the component is already wrapped in quotes. `crates/citum-engine/src/render/component.rs`.
2. **Converter (hygiene):** `convert_formatting` emitted both a `quote` field and a `wrap: quotes` for a CSL `quotes="true"` node. The `wrap` path is now the single owner; the redundant `quote` field is left unset. `crates/citum-migrate/src/template_compiler/formatting.rs`.

**Evidence:**
- `journal-of-advertising-research`: `““The Future…””` → `“The Future…”` (bib 36/38; residual diffs are csl26-ya9b).
- `early-medieval-europe`: doubled-quote note entries 15 → 0.
- Sentinels apa / chicago-author-date / nature unchanged; portfolio gate 154 styles fidelity 1.0.

**Tests:** `given_quote_flag_and_quote_wrap_when_render_then_single_pair_of_quotes` and `given_quote_flag_and_non_quote_wrap_when_render_then_both_applied` (engine); `given_quotes_true_when_convert_formatting_then_only_wrap_owns_quotes` (migrate).
