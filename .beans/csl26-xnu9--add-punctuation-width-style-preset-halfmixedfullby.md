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
updated_at: 2026-07-20T12:58:57Z
---

Follow-up from PR #1073 review (GB/T 7714 semantic-punctuation migration, docs/specs/PUNCTUATION_REALIZATION.md increment 3). Domain research (GB/T 7714-2015 §7.2, citeproc-js oracle, biblatex-gb7714) confirms the punctuation-width design space is a closed set of four presets — half | mixed | full | bylan — that biblatex-gb7714 already names via its 'gbpunctwidth' option:

- half: all structural punctuation ASCII, strict standard reading
- full: all full-width, including area/terminal period
- mixed: full-width except period and brackets (matches the citeproc-js oracle)
- bylan: CJK-script items full-width, Western-script items half-width (current GB/T migration default, via realization-default: cjk + per-item Latin evidence override)

Add a style-level 'punctuation-width' option under options.multilingual that expands to the existing realization-default + per-script realization primitives (no new marks needed) so a style can declare one of the four presets declaratively instead of hand-assembling the override table. Update PUNCTUATION_REALIZATION.md Future/§8 to reference this. If the deferred width-policy decision (see PR #1073 discussion) lands on 'mixed' instead of the current 'bylan' default, this preset is also the natural home for that: 'mixed' requires an 'unconditional CJK' realization mode that realization-default alone cannot express today (per-item Latin evidence currently always overrides it).

## Follow-up scope: locale-supplied spacing

The width presets above (half/mixed/full/bylan) cover glyph selection and width. A related, distinct gap: today a realization override can hardcode NBSP/narrow-NBSP spacing for one style (e.g. a French `colon` override as ` : ` with U+00A0), but there is no mechanism for a locale to supply that spacing automatically across all marks — this is called out in PUNCTUATION_REALIZATION.md v1.4 section 4 and section 2 as deferred, blocked on per-item locale loading (PER_ITEM_TERM_LOCALE.md). Once that lands, add locale-supplied realization spacing (French NBSP before `; : ? !`, its France/Quebec variant, similar conventions elsewhere) as the natural next increment on this same table, slotting between style override and engine default per the section 4 resolution order. Quote-glyph family (guillemets/low-high quotes/CJK brackets) is out of scope here — already solved per-locale via grammar-options and deliberately kept a separate table (section 2).
