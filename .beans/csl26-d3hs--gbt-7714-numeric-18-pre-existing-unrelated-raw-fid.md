---
# csl26-d3hs
title: 'GB/T 7714 numeric: 18 pre-existing unrelated raw fidelity failures'
status: todo
type: bug
priority: normal
tags:
    - fidelity
    - style
    - gb-t
created_at: 2026-07-17T22:53:37Z
updated_at: 2026-07-17T22:53:55Z
---

node scripts/oracle.js tests/fixtures/csl-m/gb-t-7714-2025-numeric.csl --json --scope both --refs-fixture tests/fixtures/test-items-library/gb-t-7714-2025.json --citations-fixture tests/fixtures/test-items-library/gb-t-7714-2025-numeric-citations.json --case-insensitive

shows 18 unmasked raw bibliography mismatches (of 203 refs) unrelated to punctuation, discovered while verifying csl26-fn9x (GB/T Latin-script punctuation fix). These keep gb-t-7714-2025-numeric's `min_pass_rate: 1.0` gate at `fail` independent of that fix — the gate was already failing before csl26-fn9x touched anything (85.2% raw / 85.7% adjusted at baseline).

Example: `gbt7714.7.1.3:2` (anonymous-author periodical, "Coffee drinking and cancer of the pancreas") is missing its year and issue components entirely, with wrong component ordering:

```
oracle: [21]Coffee drinking and cancer of the pancreas[J]. Br Med J，1981，283（6292）：628
citum:  [21]283Coffee drinking and cancer of the pancreas1981628
```

Looks like a substitution/anonymous-author-fallback template gap in the numeric type-variant for periodicals, not a punctuation issue.

Full list of affected ids (from baseline, still failing after adjustment):
gbt7714.7.1.3:2, gbt7714.7.2.1:7, gbt7714.7.2.3:7, gbt7714.8.11.3.2:5, gbt7714.8.14.3:3, gbt7714.8.15.2:3, gbt7714.8.1:4, gbt7714.8.4.2:4, gbt7714.8.5.3:8, gbt7714.8.5.3:9, gbt7714.8.6.1:5, gbt7714.8.6.3:2, gbt7714.8.8.3:4, gbt7714.8.9.2:4, gbt7714.9.2.1.3:1, gbt7714.9.2.2:6, gbt7714.9.3.1.1:2, gbt7714.9.3.1.2:1

- [ ] Triage each of the 18 ids by root cause (anonymous-author substitution, missing components, ordering)
- [ ] Fix the underlying template/engine gaps
- [ ] Confirm gb-t-7714-2025-numeric reaches its declared min_pass_rate: 1.0 gate
