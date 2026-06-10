---
# csl26-cxi9
title: 'migrate: bib component order and label leakage'
status: todo
type: bug
created_at: 2026-06-10T16:56:45Z
updated_at: 2026-06-10T16:56:45Z
parent: csl26-vmcr
---

Cluster C3 (~8 deep-failure numeric styles) from docs/architecture/audits/2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md. Migrated bibliography entries render components in scrambled order with label/affix leakage, e.g. brazilian-journal-of-psychiatry: citum '2017: 7: Vaswani A...: vol. 30: pp. 5998-6008: available at' vs oracle '7Vaswani A... 2017;30:599'. Also proceedings-of-the-estonian-academy-of-sciences-numeric (17/53). One bounded migrate-research pass.
