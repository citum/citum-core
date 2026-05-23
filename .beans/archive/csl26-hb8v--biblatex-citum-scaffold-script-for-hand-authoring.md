---
# csl26-hb8v
title: biblatex → Citum scaffold script for hand-authoring biblatex-benchmarked styles
status: completed
type: task
priority: low
created_at: 2026-05-23T18:16:57Z
updated_at: 2026-05-23T20:42:15Z
---

Followup from csl26-8kod. Investigates a low-cost path to accelerate authoring of biblatex-benchmarked Citum styles without taking on a full biblatex .bbx/.cbx → Citum converter.

## Why a full converter is impractical

- biblatex style files (.bbx/.cbx) are LaTeX macro code: Turing-complete, no clean AST, conditional control flow, and depend on biblatex's runtime field schema. A general converter is a multi-quarter project.
- The audience is narrow: most biblatex users already have working LaTeX. Citum's current 5 biblatex-benchmarked styles (numeric-comp, chem-acs, angewandte-chemie, chem-rsc, chem-biochem) are already hand-authored and benchmarked, so the marginal benefit is small.

## Why citum-migrate is not the right home

citum-migrate is structured around CSL 1.0 XML parsing (csl-legacy → Citum YAML). biblatex has no structural analogue to that pipeline — there is no XML-shaped style tree to walk. Trying to fit biblatex into the migrate crate would balloon its scope and dilute its CSL 1.0 focus.

## Proposed alternative — scaffold-only script

A Node script under `scripts/` (e.g. `scripts/scaffold-biblatex.js`) that consumes the rendered output already produced by `scripts/gen-biblatex-snapshot.js` (and existing snapshots in `tests/snapshots/biblatex/`) and emits a Citum YAML scaffold.

What it can plausibly infer from rendered snapshot text alone:
- bibliography sort order (compare rendered order against the fixture ref ordering).
- bibliography layout pattern (numeric prefix `(1)` vs alphabetic, em-dash separators, etc.).
- name-form / contributor-form (initials vs full names, `and` vs `&`, comma vs semicolon).
- title casing and quoting style.
- year placement heuristic (after authors vs at end).

What it cannot infer (and must be hand-completed):
- citation-side macros (biblatex snapshots only carry bibliography output).
- type-variants and per-type formatting rules.
- conditional / fallback logic.

The scaffold output should be a partially-complete YAML stub that the author finishes by hand — mirroring the existing workflow for the 5 chem styles, but with the boilerplate (sort, layout, name-form) pre-filled.

## Independent alternative (compared, not chosen)

Add biblatex conversion to citum-migrate. Rejected for the reasons above (LaTeX macro parsing complexity, dilutes crate scope, narrow audience).

## Acceptance criteria for the scaffold-only path

- [ ] Script reads a biblatex snapshot JSON + the fixture and emits a Citum YAML stub.
- [ ] Stub renders without errors via the Citum engine even if fidelity is low.
- [ ] Documentation in `docs/guides/` (or amendment to existing authoring guide) explains the scaffold → hand-finish workflow.
- [ ] Out of scope: any change to `crates/citum-migrate`.
