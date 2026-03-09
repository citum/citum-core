---
# csl26-ccqg
title: Define multi-authority style verification model
status: completed
type: feature
priority: high
created_at: 2026-03-07T02:11:51Z
updated_at: 2026-03-09T20:12:33Z
---

Plan the shift from vague oracle language to an explicit authority-source and
regression-baseline model for style evolution, reporting, CI, and docs.

Architecture doc:
- `docs/architecture/MULTI_AUTHORITY_STYLE_VERIFICATION_PLAN_2026-03-07.md`

Deliverables:
- Architecture doc in `docs/architecture/`
- Cross-linked bean and doc
- Branch commit for morning review

Scope:
- Authority source taxonomy
- Precedence and adjudication policy
- Reporting and CI impact
- Style rollout from biblatex/CSL table
- Documentation update plan

Current emphasis:
- external registry in `scripts/report-data/verification-policy.yaml`
- adjudication notes under `docs/adjudication/`
- citeproc-js over biblatex as the initial default precedence
- fixture sufficiency as the key innovation
- family comparison fixtures, including legal via adapted CSL-M intake

## Fixture Sufficiency Gaps (Step 3)

- [x] Updated fixture-sufficiency.yaml with expanded family declarations
- [x] Expanded compound-numeric-refs.json from 5 to 14 entries (9 types)
- [x] Created references-physics-numeric.json (15 entries, 8 types)
- [x] Created references-author-date.json (15 entries, 8 types)
- [x] Created references-humanities-note.json (15 entries, 7 types)
- [x] Expanded references-legal.json from 3 to 11 entries (6 types)
- [x] Created references-csl-m-adapted.json (12 entries, 4 types)
- [x] All JSON validated, 538 tests pass, quality gate passes

## Family Fixture Wiring

- [x] Wired family fixture sets into report-core.js oracle pipeline
- [x] Added normalizeRefsToKeyed() for array/wrapped fixture formats
- [x] Added runFamilyFixtureOracle() + mergeOracleResults()
- [x] Quality gate now correctly reports regressions:
  - apa-7th: 0.988 (no-date item missing n.d./retrieved/URL)
  - chicago-notes: 0.878 (manuscript, interview, translator, personal_communication gaps)

## Engine + Style Recovery

- [x] Engine: emit locale no-date term (n.d.) for missing issued dates
- [x] APA-7th: add Retrieved/URL rendering for report type-template
- [x] APA-7th family fixture: 15/15 (recovered from 14/15)
- [x] APA-7th core fixture: 32/32 + 18/18 (no regression)
- [ ] Chicago Notes: 6 citation failures remain (manuscript, interview, translator, encyclopedia, personal_communication)
