---
# csl26-rj4c
title: Random-corpus mode + baseline run for migrate SQI scorecard
status: in-progress
type: task
priority: normal
created_at: 2026-06-10T16:28:02Z
updated_at: 2026-06-10T16:57:27Z
parent: csl26-vmcr
---

Extend scripts/report-migrate-sqi.js with --corpus random --sample N --seed S: enumerate styles-legacy/*.csl independents, classify by citation-format attr, stratified seeded sampling (mulberry32), graceful failure taxonomy (migrate_failed/oracle_failed), headline aggregates (% styles >=90% strict fidelity, per-class breakdown). Extend report-migrate-sqi.test.js. Run pilot then full 100-style baseline; commit date-stamped audit in docs/architecture/audits/ + JSON snapshot in scripts/report-data/.

- [x] sampler + classification + failure taxonomy in report-migrate-sqi.js
- [x] tests in report-migrate-sqi.test.js (+ scripts/lib/corpus-sample.test.js)
- [x] pilot run (sample 10) — wiring validated; allocator floor-scaling fix
- [x] full run (sample 100, seed 20260610)
- [x] baseline audit doc + JSON snapshot committed (docs/architecture/audits/2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md)
- [x] evaluate against quality bar, report to user — BELOW BAR: 43/100 at >=90%; improvement wave activated (clusters C1-C5)
