---
# csl26-h0rt
title: 'migrate: extend synthesis operators to type variants'
status: todo
type: feature
priority: normal
tags:
    - migrate
created_at: 2026-06-13T10:16:22Z
updated_at: 2026-06-13T10:54:53Z
---

The Phase 2 synthesis loop (bean csl26-8txa, `crates/citum-migrate/src/synthesis/`) applies its four mutation operator families (component order, affix edit, label form, group boundary) only to each section's **main/default template** — the citation `template` and the bibliography `template`. Per-reference-type override templates (`type_variants`) are not mutated by the generic operators.

The Phase 1 *typed patch* candidates already reach type variants (e.g. `type-local-default`, `article-journal-suppress-*`), and those win real selections (entry-encyclopedia, legal_case, article-journal). But the generic structural operators cannot refine a per-type template that is close but structurally off (wrong component order, stray affix, misplaced group boundary within one type variant).

## Scope

- Extend `operators::enumerate_mutations` (or add a type-variant-aware enumerator) so the loop can propose single mutations against a chosen `type_variants` entry, not just the main template.
- Thread a per-type allocation through `CandidateBudget`. The budget caps *how many candidate templates get rendered and scored* per selection — a compute/throughput limit, **not memory**. Each candidate costs a full `citum-engine` render of every fixture item plus scoring against the citeproc references, so the cap bounds that work. With one global `max_total`, enumerating over type variants multiplies the space (roughly type-variants × operators); the first type keys consume the whole cap and later type variants get truncated by `truncate_mutations` before they are scored — they "starve" (a winning mutation is dropped unevaluated). Give each type variant its own slice of the cap, or raise it, so no type variant is truncated unscored.
- Keep determinism (fixed type-key order) and the held-out rejection gate.
- Measure with the seeded random-100 scorecard (seed 20260610) + held-out gate; accept only on no regression.

## Expected impact

Moderate, not large. The headline scorecard wins so far (iso690-full-note-es +25, pravny-obzor +20) came from existing Phase 1 *type-patch seeds*, not from generic operators reaching type variants. Extending the operators to type variants should let a handful of additional near-threshold styles cross 90% combined fidelity by fixing per-type structural drift, but it widens the candidate space and needs the per-type budget allocation above to avoid scoring-throughput starvation. Estimate: low single-digit additional styles at threshold; treat as incremental refinement, not a step change.

## References

- Loop core: `crates/citum-migrate/src/synthesis/core.rs` (`run_rounds`, `RoundsContext`)
- Operators: `crates/citum-migrate/src/synthesis/operators.rs`
- Spec: `docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md`
