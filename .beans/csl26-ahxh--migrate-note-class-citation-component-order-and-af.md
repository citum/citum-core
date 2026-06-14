---
# csl26-ahxh
title: 'migrate: note-class citation component order and affixes diverge'
status: todo
type: bug
priority: normal
tags:
    - migrate
    - fidelity
    - note-styles
created_at: 2026-06-14T11:21:02Z
updated_at: 2026-06-14T11:49:29Z
parent: csl26-vmcr
---

Note-class styles in the sub-90 tail render bibliographic-prose citations with wrong component order/affixes vs citeproc. Example early-medieval-europe: oracle "J. Smith et al, 'Title', Journal of Climate Analytics 12.3 (2021), pp. 201–" vs citum "Smith et al, 2021, 205, '“Title”', Journal of Climate Analytics, 12, 3, accessed". Affects anabases, bulletin-de-correspondance-hellenique, the-journal-of-transport-history, histoire-at-politique. Converter-level: note citation template synthesis reorders components and leaks separators. Repro: node scripts/oracle.js styles-legacy/early-medieval-europe.csl --json --force-migrate
