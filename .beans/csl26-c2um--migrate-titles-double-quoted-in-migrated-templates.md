---
# csl26-c2um
title: 'migrate: titles double-quoted in migrated templates'
status: todo
type: bug
priority: normal
tags:
    - migrate
    - fidelity
created_at: 2026-06-14T11:20:25Z
updated_at: 2026-06-14T11:49:29Z
parent: csl26-vmcr
---

Several author-date/note styles render titles with doubled quotation marks, e.g. journal-of-advertising-research and early-medieval-europe produce '““Title””'. The converter wraps a title component in quotes that the engine/locale also applies (or wraps twice). Converter-level template defect. Repro: node scripts/oracle.js styles-legacy/journal-of-advertising-research.csl --json --force-migrate
