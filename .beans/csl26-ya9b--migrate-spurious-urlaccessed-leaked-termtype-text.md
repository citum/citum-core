---
# csl26-ya9b
title: 'migrate: spurious URL/accessed + leaked term/type text in bibliography'
status: todo
type: bug
priority: normal
tags:
    - fidelity
    - migrate
created_at: 2026-06-14T11:21:02Z
updated_at: 2026-06-14T11:49:29Z
parent: csl26-vmcr
---

Migrated bibliographies emit fields citeproc omits and leak literal term/type text. china-information: 'Renaissance Art and Culture. Entry encyclopedia. in. Encyclopedia of World History' (leaked 'Entry encyclopedia. in.'); trailing 'accessed' / 'https://...' on non-web types across journal-of-advertising-research, early-medieval-europe. Converter-level: type-template selection emits an entry-encyclopedia literal and unconditional url/accessed. Repro: node scripts/oracle.js styles-legacy/china-information.csl --json --force-migrate
