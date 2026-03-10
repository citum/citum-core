---
# csl26-qfa3
title: Upgrade note styles for repeated-position overrides and refresh compat snapshot
status: todo
type: task
priority: normal
tags:
    - styles
    - compatibility
created_at: 2026-03-10T18:31:26Z
updated_at: 2026-03-10T19:55:18Z
---

Follow-up after repeated-note semantics engine/migration work:

- re-migrate target note styles in `--template-source xml` mode first, and record whether `citation.subsequent` / `citation.ibid` now surface directly from migrate output
- classify each target as `migration now sufficient`, `needs YAML cleanup only`, or `still blocked by mixed-condition position logic`
- audit migrated note styles for use of `citation.subsequent` / `citation.ibid` (Chicago notes, OSCOLA, MHRA, Bluebook-like styles)
- run style upgrades where needed to express intended immediate-repeat behavior without flattening intentional style-authored distinctions
- prioritize OSCOLA, MHRA, Chicago notes, and Bluebook-like styles separately; legal-note trees may still diverge in what migrate can preserve
- run oracle batch impact and core report checks
- refresh docs/compat.html if portfolio metrics or examples change
- decide whether baseline updates are needed in dedicated baseline PR
