---
# csl26-rgd6
title: Triage open CSL schema issues against Citum
status: completed
type: task
priority: normal
tags:
    - research
    - docs
    - schema
created_at: 2026-07-12T14:33:42Z
updated_at: 2026-07-12T18:13:52Z
---

Review all 112 currently-open issues on citation-style-language/schema and
categorize each against Citum's schema/engine into: (1) relevant, already
addressed (implemented or a documented DIVERGENCE_REGISTER decision), (2)
relevant, not addressed (gets a themed follow-up bean), (3) not relevant
(upstream repo process/tooling or CSL-XML/RNC-only mechanics).

Deliverables:
- [ ] Fetch full issue corpus (number/title/body/labels/url) to scratchpad
- [x] Build Citum schema surface reference (docs/schemas/*.json + DIVERGENCE_REGISTER.md + existing issue cross-references)
- [ ] Pass 1: meta/process filter (bucket 3 candidates by label/title)
- [ ] Pass 2: content triage against the surface for remaining issues, with evidence citations
- [x] Self-verify all bucket-1 "already addressed" claims (checked live against schema/locale source during triage; corrected #80 and #388 on verification)
- [x] Present draft triage table to Bruce for review/correction (2 review rounds: 5 corrections from under-built surface reference, 1 more from multilingual/date reconsideration — final: 49 bucket-1, 34 bucket-2, 29 bucket-3)
- [x] Dedupe against existing open beans, then create/revise themed bucket-2 beans (no true duplicates found; created epic csl26-kcda + 12 themed children covering all 34 bucket-2 issues)
- [x] Write audit report to docs/architecture/audits/2026-07-12_CSL_SCHEMA_ISSUE_TRIAGE.md

Plan: /home/bruce/.claude/plans/i-m-not-sure-how-typed-puzzle.md

## Summary of Changes

Triaged all 112 open issues on citation-style-language/schema against
Citum's schema/engine. Final: 49 bucket-1 (already addressed), 34 bucket-2
(real gaps), 29 bucket-3 (not relevant). Two review rounds with a
domain-expert (CSL maintainer) reviewer caught 6 false-positive bucket-2
classifications from an initially under-built schema surface reference
(missed full roles: block, messages: section, RawTermValue nesting) —
corrected before finalizing.

Deliverables:
- docs/architecture/audits/2026-07-12_CSL_SCHEMA_ISSUE_TRIAGE.md
- Epic csl26-kcda with 12 themed follow-up beans covering all 34 bucket-2 issues
