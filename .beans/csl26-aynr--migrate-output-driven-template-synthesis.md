---
# csl26-aynr
title: 'migrate: output-driven template synthesis'
status: draft
type: feature
priority: normal
created_at: 2026-06-11T16:06:50Z
updated_at: 2026-06-11T16:10:38Z
---

Direction surfaced while closing the csl26-vmcr wave (pointer in docs/architecture/audits/2026-06-11_MIGRATE_IMPROVEMENT_WAVE_OUTCOME.md). Motivation: every fragile bug that wave fixed (C3 order scrambling, conditional leakage, suppressed-variable poison, wrapper variants) lived in the structural XML layout compiler -- compiling a procedural CSL layout tree into declarative templates is a semantic mismatch patched bug by bug. XML attribute/options extraction was never the problem.

Replace XML layout compilation entirely: synthesize Citum templates by searching the candidate space against citeproc-js reference output. All machinery exists in-process after the csl26-vmcr wave: EmbeddedTemplateRuntime (deno_core) renders the citeproc reference; citum-engine renders candidates; token_jaccard in crates/citum-migrate/src/measured_citation.rs is the oracle-mirroring fitness function. Generalize measured selection from arbitrating 2 candidates to a propose/render/score/mutate loop (mutations: component order, affixes, labels, group boundaries; seeds: inferrer output). XML read only for declarative attributes (et-al, initialize-with, sort) -- the layout tree is never compiled. Deterministic, no LLM in the loop.

Prerequisites: held-out fixture items, positional scenario coverage (first/subsequent/ibid/locator), bounded mutation space. Acceptance test: the seeded random-100 scorecard (seed 20260610). Needs a spec in docs/specs/ before implementation.
