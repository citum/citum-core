---
# csl26-10lt
title: 'Delta instrument: top-k targets, embedded-only mode, memory cleanup'
status: todo
type: task
priority: normal
tags:
    - migrate
    - scorecard
created_at: 2026-07-17T17:53:29Z
updated_at: 2026-07-17T17:53:38Z
---

The 2026-07-17 delta-derivability measurement showed naive single-best-target forcing has strongly negative mean fidelity delta (-14.9 random / -26.0 styles) while winners gain +12..+32 pts - target selection must be measured, not assumed. Extend scripts/measure-delta-derivability.js: (1) try top-k (e.g. 3) targets per candidate and keep the best wrapper; (2) --targets embedded mode restricting to tuned/embedded parents; (3) per-pair citeproc/engine cleanup - the sweep peaks ~6GB and was OOM-killed at concurrency 2 in a 6GB scope; until fixed, run sandboxed at --concurrency 1..2. Then re-measure both corpora. Context: docs/architecture/audits/2026-07-17_EXTENDS_DELTA_DERIVABILITY.md
