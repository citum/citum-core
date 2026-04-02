# Rich Fidelity Inputs Specification

**Status:** Active
**Date:** 2026-04-01
**Related:** `.beans/csl26-x1y2--hook-up-generalized-relational-fixtures-to-fidelit.md`, `docs/specs/GENERALIZED_RELATIONAL_CONTAINER_MODEL.md`, `docs/specs/NUMBERING_SEMANTICS.md`

## Purpose
Define how the fidelity pipeline accepts richer benchmark inputs that exercise Citum's relational container and numbering model more fully than the legacy flat fixtures. The goal is to make future style enhancement work data-driven by supporting both native relational corpora and larger external bibliography corpora within the reporting pipeline.

## Scope
In scope: style-level `benchmark_runs` configuration, bibliography-only external benchmarks, native relational diagnostic runs, and official report output that distinguishes baseline fidelity from supplemental rich-input evidence. Out of scope: new public Rust schema changes, all-collection Zotero rollout, and replacing the existing `fixture_sets` family coverage model.

## Design
### Benchmark Runs

Styles may declare ordered supplemental `benchmark_runs` in addition to existing `fixture_sets`.

Each benchmark run defines:

- an id
- a human-readable label
- a runner kind
- one or more benchmark input files
- a verification scope
- whether the run contributes to fidelity totals or remains diagnostic-only

### Runner Kinds

This specification activates two runner kinds:

1. `citeproc-oracle`
   Uses an external comparator run and may operate in bibliography-only mode when only a reference corpus exists.
   Supported scopes in this wave: `bibliography` and `both`.

2. `native-smoke`
   Uses native Citum parsing and rendering to confirm that richer relational corpora remain usable inputs for style work even when no external oracle exists.
   Supported scope in this wave: `bibliography` only.
   `native-smoke` is always diagnostic-only in this wave.

### Rich Input Categories

The pipeline should treat richer fidelity inputs as a capability, not a one-off dataset hook:

- **Native relational corpora** validate that recursive containers, numbering, and related authoring patterns are preserved in benchmarkable inputs.
- **External bibliography corpora** validate style behavior against larger real-world datasets that expose gaps hidden by small flat fixtures.

### Initial Proof Points

The first implementation should prove the framework with:

- `examples/comprehensive.yaml`
- `examples/chicago-bib.yaml`
- `tests/fixtures/test-items-library/chicago-18th.json`

The Chicago Zotero corpus is the first external pilot, not the long-term limit of the feature.

## Implementation Notes
`benchmark_runs` extends the existing verification policy. It does not replace `fixture_sets`, fixture-family sufficiency, or current citeproc/biblatex authority selection.

Bibliography-only scoring runs must merge cleanly into bibliography totals without inventing citation fixtures. Diagnostic native runs must be visible in report output without changing fidelity math.

Rich benchmark runs are official supplemental evidence in this wave. They appear in `report-core`, `compat.html`, and related workflow summaries, but they do not yet redefine the headline portfolio gate or the meaning of current top-line fidelity claims.

## Acceptance Criteria
- [ ] Styles can declare ordered `benchmark_runs` without changing existing `fixture_sets` behavior.
- [ ] The report pipeline supports bibliography-only external benchmark runs.
- [ ] The report pipeline supports native relational diagnostic runs.
- [ ] Report output distinguishes scoring benchmark runs from diagnostic benchmark runs.
- [ ] Report output publishes compact, repo-relative rich benchmark summaries without leaking internal oracle payloads.
- [ ] Official status surfaces distinguish baseline gate metrics from supplemental rich-input evidence.
- [ ] `chicago-author-date` proves the framework with one external bibliography benchmark and two native relational diagnostics.

## Changelog
- 2026-04-01: Activated with `benchmark_runs`, bibliography-only external benchmarks, and native relational diagnostic runs.
