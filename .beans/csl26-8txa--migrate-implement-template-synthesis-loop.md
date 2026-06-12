---
# csl26-8txa
title: 'migrate: implement template synthesis loop'
status: todo
type: feature
created_at: 2026-06-12T20:46:02Z
updated_at: 2026-06-12T20:46:02Z
blocking:
    - csl26-aynr
---

Implement the Phase 2 propose/render/score/mutate synthesis loop designed in docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md. Prerequisites (positional scenarios, held-out validation, candidate budget) shipped via bean csl26-aynr. Seeds: inferrer output + XML-compiled templates (transition only). Mutation operators: component order, affixes, label forms, group boundaries. Deterministic, bounded by CandidateBudget, held-out regression rejects. Integration: replaces the default migration path; headline gate is the seeded random-100 scorecard (seed 20260610).
