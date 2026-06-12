---
# csl26-rksq
title: Public Migrate page, scorecard page, weekly CI regeneration
status: todo
type: task
priority: deferred
tags:
    - migrate
    - docs
    - scorecard
created_at: 2026-06-10T16:28:16Z
updated_at: 2026-06-12T17:25:53Z
parent: csl26-vmcr
blocked_by:
    - csl26-rj4c
---

Publish citum-migrate on docs.citum.org once baseline numbers are in: docs/guides/MIGRATING_FROM_CSL.md rendered to docs/migrate.html via build-doc-pages.js; nav link in build-layout.js; Coming-from-CSL section on index.html; converter-fidelity card on reports.html; committed docs/reports/MIGRATE_SCORECARD.md rendered to migrate-scorecard.html; weekly .github/workflows/migrate-scorecard.yml regenerating the snapshot via automated PR. All claims sourced from the measured baseline; copy confirmed with user before commit.

- [ ] MIGRATING_FROM_CSL.md guide + build-doc-pages entry
- [ ] nav + index + reports tie-ins
- [ ] scorecard markdown snapshot + page
- [ ] weekly workflow
- [ ] user confirms copy
