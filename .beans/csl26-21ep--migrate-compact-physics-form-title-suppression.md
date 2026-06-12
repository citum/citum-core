---
# csl26-21ep
title: 'migrate: compact physics form title suppression'
status: todo
type: bug
priority: low
tags:
    - migrate
    - fidelity
    - style-family
created_at: 2026-06-10T16:56:45Z
updated_at: 2026-06-12T17:25:53Z
parent: csl26-vmcr
---

Cluster C5 (small band, physics family) from docs/architecture/audits/2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md. This is a proven random-sample migration fidelity defect, but not yet proven to be converter-only, engine-side, or schema-driven.

Compact physics styles should suppress article titles and render page-only locators (`T. S. Kuhn, Philosophy of Science 37, 1 (1970)`). Migrated output includes the title and drops the page. Evidence: `springer-physics-author-date` (11/38).

Next step: one bounded migrate-research pass that classifies the root cause before implementation.
