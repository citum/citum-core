---
# csl26-vmcr
title: Promote citum-migrate with random-sample fidelity metrics
status: todo
type: epic
priority: deferred
tags:
    - fidelity
    - scorecard
    - migrate
created_at: 2026-06-10T16:27:51Z
updated_at: 2026-06-12T17:25:53Z
---

Public docs site (docs.citum.org) never mentions citum-migrate. Goal: measure converter quality on a seeded, stratified random sample of ~100 independent parent CSL styles using existing fidelity+SQI tooling, record a baseline audit, then publish a Migrate page with truthful metrics and a weekly CI-regenerated scorecard. Quality bar: >=80% of sampled styles at >=90% combined strict citation+bibliography fidelity, no style class below 60%; below the bar, run a migrate-research improvement wave first. Plan: ~/.claude/plans/i-have-a-task-federated-gosling.md

## Outcome (2026-06-11)

Improvement wave concluded; publication deferred. The 80/100 quality bar was not met and is no longer a near-term target: baseline 43/100 at >=90% combined strict fidelity, 53/100 after the wave (note-class repeat forms, suppressed-variable poison, wrapper full variants, measured citation selection PR #907), plus the C3 order/leakage fix (PR #908, merged) expected to add several more points. Reaching 80 requires flipping the entire 80-90% close-miss band AND most of the 70-80% band AND note-class recovery -- multiple waves with rising cost per point, with remaining gaps increasingly engine-level rather than converter-level.

Decision: stop LLM-driven improvement waves; no Migrate page or weekly scorecard until the number is earned by ordinary engineering. Full record: docs/architecture/audits/2026-06-11_MIGRATE_IMPROVEMENT_WAVE_OUTCOME.md.

## Active migration backlog

- csl26-y4o7: engine/migrate once-only variable consumption semantics. This is the clearest current engine-side migration fidelity blocker: migrated YAML can be structurally reasonable but still lose bibliography fields because suppressed groups consume variables inconsistently.
- csl26-21ep: compact physics style-family migration defect. This is a proven fidelity gap for the random-sample physics cluster, but it still needs diagnosis before classifying as converter-only, engine-side, or schema-driven.
- csl26-aynr: output-driven template synthesis. This is the long-term sustainable migration architecture after the measured-candidate PRs; it needs a spec before implementation.
- csl26-h7xz: document the template inferrer in the rendering workflow.
- csl26-rksq: public Migrate page, scorecard page, and weekly regeneration. Deferred until fidelity metrics justify publication.

## Not direct migration blockers

The following open draft/schema beans are broader multilingual or data-model backlog, not direct blockers for the current CSL random-sample migration fidelity work: csl26-6rjq, csl26-xz2t, csl26-1b4e, csl26-dno4, csl26-ldgf, csl26-9oee. Triage them in a separate multilingual/schema bean-hygiene pass if they need promotion, scrapping, or decomposition.
