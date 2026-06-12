---
# csl26-aynr
title: 'migrate: output-driven template synthesis'
status: in-progress
type: feature
priority: normal
tags:
    - migrate
    - fidelity
    - architecture
created_at: 2026-06-11T16:06:50Z
updated_at: 2026-06-12T20:45:34Z
---

Direction surfaced while closing the csl26-vmcr wave (pointer in docs/architecture/audits/2026-06-11_MIGRATE_IMPROVEMENT_WAVE_OUTCOME.md) and was reinforced by PR #910's measured-candidate selection work. Motivation: every fragile bug that wave fixed or exposed (C3 order scrambling, conditional leakage, suppressed-variable poison, wrapper variants) lived in the structural XML layout compiler -- compiling a procedural CSL layout tree into declarative templates is a semantic mismatch patched bug by bug. XML attribute/options extraction was never the problem.

Replace XML layout compilation entirely: synthesize Citum templates by searching the candidate space against citeproc-js reference output. All machinery exists in-process after the csl26-vmcr wave: EmbeddedTemplateRuntime (deno_core) renders the citeproc reference; citum-engine renders candidates; token_jaccard in crates/citum-migrate/src/measured_citation.rs is the oracle-mirroring fitness function. Generalize measured selection from arbitrating 2 candidates to a propose/render/score/mutate loop (mutations: component order, affixes, labels, group boundaries; seeds: inferrer output). XML read only for declarative attributes (et-al, initialize-with, sort) -- the layout tree is never compiled. Deterministic, no LLM in the loop.

Status: promoted to `todo` because this is the sustainable long-term migration strategy, but implementation remains blocked on design. Needs a spec in docs/specs/ before code.

Prerequisites: held-out fixture items, positional scenario coverage (first/subsequent/ibid/locator), bounded mutation space, and an explicit candidate-family budget so scoring cannot become unbounded. Acceptance test: the seeded random-100 scorecard (seed 20260610).

## Progress (2026-06-12)

Phase 2 design and prerequisites shipped on PR branch
codex/migrate-synthesis-prereqs:

- [x] Spec: docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md restructured into Phase 1 (shipped selector) + Phase 2 (Draft synthesis loop; replace-default integration)
- [x] Positional scenario coverage: five citation scenarios (first, first+locator, subsequent, ibid, ibid+locator) across JS runtime and engine
- [x] Held-out fixture set tests/fixtures/references-heldout.json with post-selection validation reporting
- [x] CandidateBudget caps on measured candidate generation
- [ ] Synthesis loop implementation (tracked in child bean)
