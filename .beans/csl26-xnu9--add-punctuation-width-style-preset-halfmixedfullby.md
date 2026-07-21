---
# csl26-xnu9
title: Add punctuation-width style preset (half|mixed|full|bylan)
status: draft
type: task
priority: normal
tags:
    - multilingual
    - punctuation
    - style
created_at: 2026-07-20T12:12:23Z
updated_at: 2026-07-21T11:38:41Z
parent: csl26-0ugp
---

Follow-up from PR #1073 review (GB/T 7714 semantic-punctuation migration, docs/specs/PUNCTUATION_REALIZATION.md increment 3). Domain research (GB/T 7714-2015 §7.2, citeproc-js oracle, biblatex-gb7714) confirms the punctuation-width design space is a closed set of four presets — half | mixed | full | bylan — that biblatex-gb7714 already names via its 'gbpunctwidth' option:

- half: all structural punctuation ASCII, strict standard reading
- full: all full-width, including area/terminal period
- mixed: full-width except period, slashes, dashes, and square brackets (matches the citeproc-js oracle; see "PR #1073 comment follow-up" below for full detail)
- bylan: CJK/square-script items (Chinese, Japanese, Korean) full-width, alphabetic-script items (Latin, Cyrillic) half-width (current GB/T migration default, via realization-default: cjk + per-item evidence override; see "PR #1073 comment follow-up" below)

Add a style-level 'punctuation-width' option under options.multilingual that expands to the existing realization-default + per-script realization primitives (no new marks needed) so a style can declare one of the four presets declaratively instead of hand-assembling the override table. Update PUNCTUATION_REALIZATION.md Future/§8 to reference this. If the deferred width-policy decision (see PR #1073 discussion) lands on 'mixed' instead of the current 'bylan' default, this preset is also the natural home for that: 'mixed' requires an 'unconditional CJK' realization mode that realization-default alone cannot express today (per-item Latin evidence currently always overrides it).

## Follow-up scope: locale-supplied spacing

The width presets above (half/mixed/full/bylan) cover glyph selection and width. A related, distinct gap: today a realization override can hardcode NBSP/narrow-NBSP spacing for one style (e.g. a French `colon` override as ` : ` with U+00A0), but there is no mechanism for a locale to supply that spacing automatically across all marks — this is called out in PUNCTUATION_REALIZATION.md v1.4 section 4 and section 2 as deferred, blocked on per-item locale loading (PER_ITEM_TERM_LOCALE.md). Once that lands, add locale-supplied realization spacing (French NBSP before `; : ? !`, its France/Quebec variant, similar conventions elsewhere) as the natural next increment on this same table, slotting between style override and engine default per the section 4 resolution order. Quote-glyph family (guillemets/low-high quotes/CJK brackets) is out of scope here — already solved per-locale via grammar-options and deliberately kept a separate table (section 2).

## PR #1073 comment follow-up (2026-07-21)

Domain-expert review from `@YDX-2147483647`
([comment `5032704432`](https://github.com/citum/citum-core/pull/1073#issuecomment-5032704432))
corrects and extends this bean's preset definitions. Docs amended: this bean,
`docs/specs/PUNCTUATION_REALIZATION.md` (v1.6), and `DIVERGENCE_REGISTER.md`
div-014.

### Corrected preset definitions

- **`mixed` was defined too narrowly** ("full-width except period and
  brackets"). Correct: full-width except **period, slashes, dashes, and
  square brackets**. `mixed` = bibtex's `GB` (2025 default) = biblatex's
  `mixed` = the Zotero bilingual-CSL styles' default = **the citeproc-js
  oracle's actual behavior**.
- **`bylan` classifies by script family, not "CJK vs. Western language."**
  Full-width applies to all square-character/CJK scripts — Chinese,
  **Japanese, Korean** — not just Chinese; half-width applies to alphabetic
  scripts (Latin, Cyrillic/Russian). Verified: the engine's `script_class`
  (`crates/citum-engine/src/values/mod.rs:415-423`) already maps
  `Jpan`/`Kore`/`Hang`/`Bopo` to `ScriptClass::Cjk` alongside `Hani`/`Hans`/
  `Hant`, and the embedded `ja-JP` locale
  (`crates/citum-schema-style/embedded/locales/ja-JP.yaml`) already authors
  full-width colon/comma/semicolon and CJK corner-bracket quotes. No code or
  locale fix needed — `bylan` is already correct for Japanese today.

### Cross-implementation equivalence

| bibtex `bibpunct` | biblatex `gbpunctwidth` | Notes |
|---|---|---|
| `half` (2015/2005 default) | `half` (default) | All ASCII/Narrow |
| `GB` (2025 default) | `mixed` | = Zotero bilingual-CSL default = citeproc-js oracle |
| — | `full` | All full-width |
| `bylanguage` | `bylan` | Script-conditional; Citum's current GB/T default |

### Default is contested — needs a decision, not assumed here

biblatex default = `half`; bibtex 2025 default = `GB` (=`mixed`); Zotero CSL
bilingual default = `mixed`; citeproc-js oracle = `mixed`; the commenter
reports no community consensus and personally prefers `half`. Citum
currently ships `bylan`. **Recommendation to confirm with the user:** target
`mixed` for citeproc-js oracle parity (Citum's stated compatibility target
for CSL-derived styles), documenting `bylan` as a named alternative rather
than the default. This is an open decision, not made in this doc-only pass.

### Mechanism gap

`half`, `full`, and `mixed` are **script-independent** fixed tables — width
does not vary by item script. `bylan` is the only **script-conditional**
preset, and the only one `realization-default` + per-item-evidence
(PUNCTUATION_REALIZATION.md §5) expresses today, since that mechanism only
ever overrides *toward* CJK for an opted-in style. Implementing
`half`/`full`/`mixed` requires a new "unconditional / script-independent"
realization mode.

### Terminology

Ecosystem preset names (`half`/`full`/`mixed`/`bylan`) are kept as-is, but
"half-width" is technically imprecise: it denotes ASCII / UAX #11 *Narrow
(Na)*, and "full-width" denotes *Fullwidth (F)*. The presets select
**codepoints**, not rendered glyph widths — actual glyph width is a font
property and the standard's own examples mix codepoint intent with
font-rendered appearance (per the commenter's `／ToUnicode` analysis).

### Document-level configurability

Out of scope for this bean's remaining work: per-document selection of a
punctuation-width preset already has a home in the existing
`DocumentOptionsOverride` (`crates/citum-engine/src/processor/document/types.rs`,
frontmatter `options:` block) — no new override layer is needed, just a
field on that existing struct when this bean is implemented.
