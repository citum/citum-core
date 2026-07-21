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
updated_at: 2026-07-21T14:01:37Z
blocked_by:
    - csl26-7hsx
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

## Update (2026-07-21)

Root cause identified and fixed by csl26-7hsx: `resolve_localized_type_variant`'s callers passed `None` for the section-level `type_variants` fallback tier, so English items whose type wasn't redefined in the style's `en` locale override (e.g. `gbt7714.7.1.3:2`, the Coffee-drinking periodical) skipped straight to the locale block's flat, delimiter-less template — not a substitution/anonymous-author gap as originally diagnosed here.

After the fix, oracle re-run (`node scripts/oracle.js ... gb-t-7714-2025-numeric`) shows raw bibliography matches rising from 143/203 to 146/203. Of the original 18 tracked ids: 4 now raw-match the oracle exactly (`gbt7714.7.2.1:7`, `gbt7714.7.2.3:7`, `gbt7714.8.11.3.2:5`, `gbt7714.8.9.2:4`); the remaining 14 (including `gbt7714.7.1.3:2`) now have every structural component (title/volume/issue/year/pages) matching — their only remaining divergence is the pre-existing, already-registered Latin-script punctuation convention (full-width vs GB/T's own Latin half-width rule; see csl26-5y6k / MULTILINGUAL.md §3.2a), not a new or distinct bug.

Remaining scope for this bean: re-triage the *other* raw bibliography failures (57 total, up from an original baseline of ~60) not among these 18 tracked ids, since this fix's scope was limited to the locale-type-variant fallback.
