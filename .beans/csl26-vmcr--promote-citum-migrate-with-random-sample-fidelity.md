---
# csl26-vmcr
title: Promote citum-migrate with random-sample fidelity metrics
status: in-progress
type: epic
priority: deferred
created_at: 2026-06-10T16:27:51Z
updated_at: 2026-06-11T15:57:01Z
---

Public docs site (docs.citum.org) never mentions citum-migrate. Goal: measure converter quality on a seeded, stratified random sample of ~100 independent parent CSL styles using existing fidelity+SQI tooling, record a baseline audit, then publish a Migrate page with truthful metrics and a weekly CI-regenerated scorecard. Quality bar: >=80% of sampled styles at >=90% combined strict citation+bibliography fidelity, no style class below 60%; below the bar, run a migrate-research improvement wave first. Plan: ~/.claude/plans/i-have-a-task-federated-gosling.md

## Outcome (2026-06-11)

Improvement wave concluded; publication deferred. The 80/100 quality bar was not met and is no longer a near-term target: baseline 43/100 at >=90% combined strict fidelity, 53/100 after the wave (note-class repeat forms, suppressed-variable poison, wrapper full variants, measured citation selection PR #907), plus the C3 order/leakage fix (PR #908, merged) expected to add several more points. Reaching 80 requires flipping the entire 80-90% close-miss band AND most of the 70-80% band AND note-class recovery -- multiple waves with rising cost per point, with remaining gaps increasingly engine-level rather than converter-level.

Decision: stop LLM-driven improvement waves; no Migrate page or weekly scorecard until the number is earned by ordinary engineering. Remaining levers stay in backlog as normal bugs: csl26-y4o7 (engine once-only consumption semantics), csl26-21ep (C5 physics compact form, low). The public-page task csl26-rksq is deferred indefinitely. Full record: docs/architecture/audits/2026-06-11_MIGRATE_IMPROVEMENT_WAVE_OUTCOME.md.
