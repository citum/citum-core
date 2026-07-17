---
# csl26-fn9x
title: GB/T renders Latin-script refs with CJK punctuation
status: todo
type: bug
priority: high
tags:
    - fidelity
    - multilingual
    - style
created_at: 2026-07-17T21:06:17Z
updated_at: 2026-07-17T21:06:27Z
---

citum render refs --style gb-t-7714-2025-numeric -b tests/fixtures/test-items-library/gb-t-7714-2025.json renders English refs with full-width punctuation (Chichester：John Wiley & Sons，2020：35) where GB/T practice uses Latin punctuation for Latin-script references. Root cause: the CSL-M source style hardcodes full-width delimiters, citeproc-js reproduces them, and our fidelity gate uses citeproc-js as sole authority (verification-policy: gb-t-7714-2025-numeric, authority: citeproc-js) - so byte-parity (198/203 in PR 1064) is satisfied while the output is non-conformant with the standard itself. Fix belongs in the Citum GB/T styles (per-item-language punctuation, see docs/specs/MULTILINGUAL.md), with the intentional oracle divergence registered in scripts/report-data/verification-policy.yaml divergences. Same failure class as the tier-0 alias negative result: verification proxy weaker than the real requirement.
