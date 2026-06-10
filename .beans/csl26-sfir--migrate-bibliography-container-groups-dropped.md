---
# csl26-sfir
title: 'migrate: bibliography container groups dropped'
status: todo
type: bug
priority: high
created_at: 2026-06-10T16:56:45Z
updated_at: 2026-06-10T16:56:45Z
parent: csl26-vmcr
---

Cluster C2 (largest reach, ~24 close-miss styles in the 82-90% band) from docs/architecture/audits/2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md. Migrated bibliographies drop container/periodical groups: journal name, year, volume vanish from rendered entries, and citation-number prefixes are lost. Evidence: zeitschrift-fur-allgemeinmedizin renders 'Rodriguez M. Major Breakthrough...' where citeproc-js renders '16.Rodriguez M. Major Breakthrough... The New York Times. 2024'. One bounded migrate-research pass; measure with node scripts/report-migrate-sqi.js --corpus random --seed 20260610.
