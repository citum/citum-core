---
# csl26-8b1g
title: Create numeric-compound chemistry styles
status: completed
type: feature
priority: normal
created_at: 2026-03-06T16:23:29Z
updated_at: 2026-03-06T17:10:47Z
---

Create 5 new Citum styles using the compound-numeric feature:
1. numeric-comp.yaml - base numeric-compound style (biblatex numeric-comp analog)
2. angewandte-chemie.yaml - Angewandte Chemie International Edition (primary use case)
3. chem-acs.yaml - ACS style with compound grouping
4. chem-biochem.yaml - ACS Biochemistry style with compound
5. chem-rsc.yaml - Royal Society of Chemistry with compound

All styles use options.bibliography.compound-numeric to enable the sets-based grouping feature implemented in csl26-zafv.


## Summary of Changes

Created 5 new Citum styles exercising the compound-numeric feature:

| File | Style |
|---|---|
| `styles/numeric-comp.yaml` | Generic base (biblatex numeric-comp analog) |
| `styles/angewandte-chemie.yaml` | Angewandte Chemie Int'l Ed. |
| `styles/chem-acs.yaml` | ACS Chemistry with compound |
| `styles/chem-biochem.yaml` | ACS Biochemistry with compound |
| `styles/chem-rsc.yaml` | RSC with compound |

All validated via `citum check`. Smoke-tested with
`tests/fixtures/compound-numeric-refs.yaml` showing correct [1a]/[1b]
citation output and shared [1] bibliography numbering.
