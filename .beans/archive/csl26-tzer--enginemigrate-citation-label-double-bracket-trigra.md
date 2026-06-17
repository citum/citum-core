---
# csl26-tzer
title: 'engine/migrate: citation-label double-bracket + trigraph length'
status: completed
type: bug
priority: normal
tags:
    - fidelity
    - migrate
    - engine
created_at: 2026-06-14T11:19:46Z
updated_at: 2026-06-17T00:52:22Z
parent: csl26-vmcr
blocked_by:
    - csl26-cvlm
---

After the converter emits citation-label + Processing::Label (csl26-cvlm), label styles render but with two residual format diffs vs citeproc:

1. **Double brackets**: american-mathematical-society-label renders '[[Kuh62]]' (cluster wrap '[...]' + per-item label '[Kuh62]') where citeproc gives '[Kuhn62]'. din-1505-2-alphanumeric renders single '[Kuh62]' correctly — it carries citation 'wrap: brackets' while AMS does not, yet AMS still double-wraps. Locus: interaction between the engine label-wrap (citum-engine processor/labels.rs) and the migrated citation cluster affix. Determine whether the label should render bare and the cluster supply brackets, vs label self-wraps.
2. **Trigraph length**: engine 'Kuh62' (3 chars) vs citeproc 'Kuhn62' (4). LabelConfig preset 'alpha' single-author name length differs from AMS. Tune LabelParams or detect from CSL.

Repro: node scripts/oracle.js styles-legacy/american-mathematical-society-label.csl --json --force-migrate
