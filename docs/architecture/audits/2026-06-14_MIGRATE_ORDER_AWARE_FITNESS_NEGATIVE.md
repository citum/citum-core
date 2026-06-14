# Migrate Order-Aware Fitness — Negative Result

- **Date:** 2026-06-14
- **Spec:** [OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md](../../specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md) (Phase 2 synthesis loop)
- **Related beans:** `csl26-8txa` (Phase 2 loop), `csl26-h0rt` (type-variant operator widening — kept `todo`, unpromising for the same reason below)
- **Baseline:** 67/100 styles at ≥90% combined strict fidelity (seed 20260610), mean 90.5, p50 94.7, p90 100, p10 74.1
- **Outcome:** flat headline, net −1.7 per-style churn — **reverted, not shipped**

## What was tried

A web-research pass (Perplexity) proposed refinements to the Phase 2
propose/render/score/mutate loop. Its highest-confidence suggestion was a
**structured, order-aware fitness function** to replace the blunt bag-of-words
similarity. The grounded, in-tree version of that idea was implemented and
measured:

- A deterministic, dependency-free **token-sequence similarity** (normalized
  longest-common-subsequence / Dice ratio over the normalized token sequence)
  in `crates/citum-migrate/src/measured_citation.rs`.
- Blended into the `similarity_sum` **gradient** the acceptance rule ties on
  (`candidate_beats`), weighted 0.5 against the existing Jaccard.
- **Pass classification deliberately left on raw Jaccard** so headline pass
  counts could not churn from reclassification — the order signal influenced
  only tie-breaks and the loop's preference among equal-pass candidates.

The intent was to give the `component-order` and `group-boundary` mutation
operators a usable gradient: under bag-of-words Jaccard, a pure reorder scores
identically to the incumbent, so those operators are effectively inert.

## Measured result

Seeded random-100 scorecard (`node scripts/report-migrate-sqi.js --corpus
random --sample 100 --seed 20260610`), baseline vs. branch:

| Metric | Baseline | After |
|---|---|---|
| Styles ≥90% combined | 67/100 | **67/100** |
| Mean combined | 90.5 | 90.5 |
| Per-style movement | — | 97 unchanged, 1 improved (+1.7), 2 regressed (−1.7 each) |

## Why it is structurally futile, not merely under-tuned

The scorecard headline is **pass-count based**: combined fidelity is
`passed / total` at a fixed 0.60 per-item similarity threshold. A
`component-order` or `group-boundary` mutation is **pass-neutral by
construction** — permuting rendered tokens leaves the Jaccard ratio unchanged,
and on the bibliography side an exact-match entry stays an exact match. So a
gradient-only refinement can only change *which equal-pass candidate wins* via
the tie margin; it can never move a pass-count headline.

This is not specific to the chosen LCS weight. The reserved tuning levers
(adjusting the sequence weight, adding a degenerate-surface penalty) operate on
the **same pass-neutral `similarity_sum`**, so they cannot move the headline
either. The only lever that could is making **pass classification itself
order-sensitive** (blend ≥ 0.60 instead of Jaccard ≥ 0.60) — which broadly
reclassifies currently-passing items and reintroduces exactly the churn the
measurement bar exists to prevent. Not pursued.

The other Perplexity suggestions were assessed and declined: "adaptive proposal
order" is inert because the loop already scores every proposal up to budget and
the budget rarely truncates; "near-improvement tolerance" already exists as the
`TIE_MARGIN` path; composite/numeric mutations and fixture enrichment *widen*
the search space, the direction prior waves already found unrewarding.

## Decision

- Do **not** ship output-driven *scoring* refinements expecting a headline gain;
  the gradient is decoupled from the pass-count metric by design.
- The change was reverted; this record is the durable artifact.
- The seeded random-100 scorecard remains the measurement instrument of record.

## What this means for high-fidelity migration

High-fidelity migration is **not** out of reach — most of it is already done.
The current distribution is left-skewed and high: median style 94.7%, p90 100%.
The unmet part is the bar (>80/100 at ≥90% combined) and the low tail (p10 74).

What this experiment confirms is **where the remaining gains are not**: not in
smarter candidate *selection*. When citeproc output cannot be matched by any
candidate because `citum-engine` renders correct template data incorrectly, no
amount of scoring search helps — the spec's own "engine-level gaps" failure
mode. This is consistent with every prior migrate audit
([2026-06-11 wave outcome](2026-06-11_MIGRATE_IMPROVEMENT_WAVE_OUTCOME.md))
and the convergence-plateau note in `crates/citum-migrate/CLAUDE.md`.

The productive next levers, in order:

1. **Engine rendering fixes** for the sub-threshold tail — classify the 33
   sub-90 styles into *engine-level* (correct template, wrong render),
   *converter-level* (wrong template), and *genuinely hard* (legal,
   multi-locale) before any further converter work. The first cohort also
   repairs checked-in styles.
2. **Hand-authoring** for top parent styles, already the canonical path per the
   migrate crate's `CLAUDE.md`.

Neither requires more fitness-function engineering.
