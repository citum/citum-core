---
# csl26-ahxh
title: 'migrate: note-class citation component order and affixes diverge'
status: todo
type: bug
priority: normal
tags:
    - migrate
    - fidelity
    - note-styles
created_at: 2026-06-14T11:21:02Z
updated_at: 2026-07-06T18:43:28Z
parent: csl26-vmcr
---

Note-class styles in the sub-90 tail render bibliographic-prose citations with wrong component order/affixes vs citeproc. Example early-medieval-europe: oracle "J. Smith et al, 'Title', Journal of Climate Analytics 12.3 (2021), pp. 201–" vs citum "Smith et al, 2021, 205, '“Title”', Journal of Climate Analytics, 12, 3, accessed". Affects anabases, bulletin-de-correspondance-hellenique, the-journal-of-transport-history, histoire-at-politique. Converter-level: note citation template synthesis reorders components and leaks separators. Repro: node scripts/oracle.js styles-legacy/early-medieval-europe.csl --json --force-migrate

## Root Cause (2026-07-06 review)

Reproduced: `node scripts/oracle.js styles-legacy/early-medieval-europe.csl --json --force-migrate` → citations 18/20 passed, with entries like:

- oracle: `T.S. Kuhn, 'The Structure of Scientific Revolutions', Chicago International Encyclopedia of Unified Science 2.2 (1962)`
- citum:  `Kuhn, 1962, "The Structure of Scientific Revolutions", International Encyclopedia of Unified Science, 2, 2, accessed`
- **match: true** — the per-item pass metric is bag-of-words Jaccard (`measured_citation.rs::token_jaccard`), which is order- and punctuation-blind, so this visibly wrong entry passes.

Mechanism (three compounding converter defects, all on the citation seed for note-class styles):

1. **Author-date-shaped seed order.** Note-class styles ride the same citation synthesis path as in-text styles (`synthesis/citation.rs`); the inferred seed places year after author (in-text convention) instead of the note layout's terminal `(year)`. Because the pass metric cannot see order, seed selection and mutation rounds neither penalize nor repair it (confirmed structurally futile to fix via scoring: docs/architecture/audits/2026-06-14_MIGRATE_ORDER_AWARE_FITNESS_NEGATIVE.md).
2. **Separator/affix loss.** `volume.issue` (`2.2`) degrades to list-delimited `2, 2` — the vol/issue group delimiter from the XML layout is not preserved into the note citation template (grouping passes exist in `passes/grouping.rs` but target bibliography shapes).
3. **Gating loss.** Bare `accessed` leaks where citeproc renders it only for web-only items — `fixups/gating.rs::gate_web_only_url_accessed` is not applied to citation templates. Name form is also wrong (`Kuhn` vs `T.S. Kuhn`): note first-references use the full/initialized form, but the citation contributor extraction emits the in-text short form.

## Fix Design

Fix the seed, not the scorer:

1. **Route note-class citation seeds through the XML layout compilation with order preserved.** For `Processing::Note` styles, make `compile_citation_note` (template_compiler/mod.rs) the primary seed and bias `pick_seed` to it unless the inferred candidate strictly beats it on passes — the inferrer's citation fragments are in-text-shaped and structurally wrong for notes.
2. **Apply the bibliography-side passes to note citation templates**: run `passes/grouping.rs` vol/issue grouping and `fixups/gating.rs` accessed/URL gating on the citation template when the style class is note (they currently run only on bibliography templates).
3. **Contributor form:** for note-class citation templates, extract the first-reference name form from the note layout's `<names>` attributes (initialize-with / name-as-sort-order) instead of defaulting to the in-text short form.
4. **Diagnostic only, not pass classification:** add an order-sensitive similarity column to the migrate debug/evidence output for note-class styles so regressions are visible, per the 2026-06-14 decision not to change the pass metric.

Verify against the affected cohort: early-medieval-europe, anabases, bulletin-de-correspondance-hellenique, the-journal-of-transport-history, histoire-at-politique (`--force-migrate` reruns), plus the seeded random-100 scorecard to confirm no headline regression.

Sizing: items 2-3 are Sonnet-executable; item 1 touches seed routing and should be reviewed against OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md first.
