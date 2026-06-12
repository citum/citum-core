---
# csl26-hxhx
title: 'migrate: remove XML layout compiler once it stops winning'
status: todo
type: task
priority: normal
created_at: 2026-06-13T10:17:39Z
updated_at: 2026-06-13T10:30:42Z
blocked_by:
    - csl26-h0rt
---

The Phase 2 synthesis loop (bean csl26-8txa) made the CSL XML layout compiler **non-authoritative**: it is no longer compiled as the template authority, but it is retained in-tree as one *seed candidate* in the synthesis search space during the transition. This is by design per `docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md` (§Integration): "The XML layout compiler remains in-tree only as a candidate seed until the seeded scorecard shows it no longer wins selections, after which it can be removed."

This bean tracks that removal — the actual code reduction. It is the deletion of the XML→declarative-template *layout compilation* path (NOT the CSL XML parsing in `csl-legacy`, which stays: the loop still reads XML for declarative attributes/options).

## Gate (must hold before removal)

Run the seeded random-100 scorecard with selection debug and confirm the `xml` seed candidate wins **0 (or a negligible count of)** citation and bibliography selections:

```
CITUM_MIGRATE_DEBUG_CITATION_SELECTION=1 CITUM_MIGRATE_DEBUG_BIB_SELECTION=1 \
  <run migrate over the seed-20260610 sample> | grep "selected .* xml"
```

If XML still wins a non-trivial share, those styles regress on removal — do not remove yet; instead add mutation/patch families that recover what XML was providing, then re-measure.

### Baseline measurement (2026-06-13, seed-20260610 sample, post-csl26-8txa)

The `xml` seed **still wins a substantial share**, so the gate does NOT hold yet:

- citation selections: **24 / 99** styles select the `xml` candidate (~24%)
- bibliography selections: **17 / 98** styles select the `xml` candidate (~17%)

Removing the compiler today would regress roughly **1 in 5 styles**. The gate's "0 or negligible" condition is far from met — this bean is blocked on synthesis (incl. csl26-h0rt type-variant operators) covering what XML uniquely provides, then a re-measurement showing the win-share near zero.

### Entanglement caveat (not a clean seed-drop)

`xml_fallback`'s output is referenced **~31 times** in `crates/citum-migrate/src/main.rs`. Beyond feeding the losing whole-template `xml` seed, it also supplies **type-variant templates** (`merge_inferred_type_templates` in `bib_postprocess.rs`) and **note-position overrides** that are merged into the *inferred/synthesized* path. So even once the whole-template seed stops winning, its *partial* contributions may remain load-bearing — removal must confirm nothing else depends on them (this is why csl26-h0rt is a prerequisite, not just a nice-to-have).

### Deletable surface (scope of the reduction, ~2,700 LOC)

- `crates/citum-migrate/src/template_compiler/` — ~2,386 lines
- `crates/citum-migrate/src/compilation.rs` (`compile_from_xml` path) — ~322 lines
- plus the `use_xml` / `source_xml` / `xml_passes` selection plumbing in `measured_citation.rs` and `synthesis/`.

## Scope (once gate holds)

- Remove the XML layout compiler module(s) in `crates/citum-migrate/src/` (the layout-tree → `TemplateComponent` compilation; `compilation.rs` / `template_compiler/` paths feeding the `xml` seed).
- Drop the `xml`/`source-xml` seed from `synthesis/citation.rs` and `synthesis/bibliography.rs` and simplify `pick_seed`/`MeasuredBibliographySelection` (`use_xml`, `xml_passes`, `source_xml` candidate) accordingly.
- Keep XML *parsing* for declarative attributes/options (et-al thresholds, initialize-with, sort keys, disambiguation).
- Re-run the full gate + portfolio quality gate; require no regression.

## Expected impact

This is the net-negative-LOC change the synthesis work sets up. Phase 2 itself *added* code (the loop + operators) and kept XML as a seed, so the diff was net-positive. Removal lands the reduction: deletes the procedural XML layout compiler and its selection plumbing once the data shows it is dead weight. Fidelity should be unchanged (gate proves XML no longer wins); the win is maintainability — one declarative synthesis path instead of two parallel template sources.

## References

- Spec §Integration: `docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md`
- Seeds assembled in `crates/citum-migrate/src/synthesis/{citation,bibliography}.rs`
- XML seed plumbing: `MeasuredBibliographySelection.use_xml`, `CandidateStyle::source_xml` in `crates/citum-migrate/src/measured_citation.rs`
