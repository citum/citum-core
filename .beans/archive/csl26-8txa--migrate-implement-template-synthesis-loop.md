---
# csl26-8txa
title: 'migrate: implement template synthesis loop'
status: completed
type: feature
priority: normal
created_at: 2026-06-12T20:46:02Z
updated_at: 2026-06-13T09:43:12Z
blocking:
    - csl26-aynr
---

Implement the Phase 2 propose/render/score/mutate synthesis loop designed in docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md. Prerequisites (positional scenarios, held-out validation, candidate budget) shipped via bean csl26-aynr. Seeds: inferrer output + XML-compiled templates (transition only). Mutation operators: component order, affixes, label forms, group boundaries. Deterministic, bounded by CandidateBudget, held-out regression rejects. Integration: replaces the default migration path; headline gate is the seeded random-100 scorecard (seed 20260610).

## Implementation Plan (2026-06-12)

- [x] Baseline seeded random-100 scorecard captured on main (seed 20260610)
- [x] synthesis/operators.rs: four mutation families (order, affix, label form, group boundary)
- [x] synthesis/mod.rs: propose/render/score/mutate loop with held-out rejection gate
- [x] measured_citation.rs: pub(crate) reuse surface + selection struct synthesis metadata
- [x] lib.rs: register synthesis module
- [x] main.rs: route default migration path through synthesize_citation/synthesize_bibliography
- [x] spec status Draft -> Active (Phase 2 shipped)
- [x] cargo gate + targeted tests green
- [x] post-change scorecard: no regression vs baseline
- [x] portfolio quality gate green

Deferred follow-up: mutation operators apply to the section's main template only; bibliography/citation type-variant templates are a follow-up surface.

## Summary of Changes

Implemented Phase 2 of `docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md`: the deterministic propose/render/score/mutate synthesis loop.

- `crates/citum-migrate/src/synthesis/operators.rs` — four typed mutation operator families (component order, affix edit, label form, group boundary), enumerated in fixed deterministic order with no-op filtering and per-family budget caps.
- `crates/citum-migrate/src/synthesis/mod.rs` — the loop core: seed scoring (inferred + XML + Phase 1 patches), up to `MAX_SYNTHESIS_ROUNDS` (4) bounded mutation rounds accepting only under the Phase 1 acceptance rule, and a held-out rejection gate that walks the incumbent history back to the best non-regressing candidate.
- `measured_citation.rs` — Phase 1 helpers raised to `pub(crate)`; `select`/`select_bibliography` now delegate to the loop with zero rounds (unchanged semantics); selection results carry `synthesis_rounds` + `accepted_mutations`.
- `main.rs` — default migration path routed through `synthesize_citation`/`synthesize_bibliography` (no opt-in flag).
- Spec status → Active (Phases 1–2 shipped).

### Headline gate (seeded random-100, seed 20260610)

| metric | before | after |
|---|---|---|
| styles ≥90% combined | 64 | 66 |
| combined mean | 89.9 | 90.5 |
| combined p50 | 93.1 | 94.7 |

Zero per-style regressions; 13 styles improved (largest: iso690-full-note-es +25.0, pravny-obzor +20.0, the-journal-of-transport-history +10.0). Portfolio quality gate passed (154 styles, fidelity 1.0). Full cargo gate green (fmt/clippy -D warnings/1591 tests).

### Deferred follow-up

Mutation operators apply to each section's main template only; bibliography/citation type-variant templates remain a follow-up surface.
