---
# csl26-hxqq
title: 'Fix cross-entry fidelity: disambiguation + author substitution'
status: completed
type: feature
priority: high
created_at: 2026-06-02T13:26:26Z
updated_at: 2026-06-02T13:44:26Z
---

Two coupled bugs in the compat pipeline:
1. Migrated YAML for apa-7th, apa, chicago-author-date lacks disambiguation config and subsequent-author-substitute, even though source CSL declares both. 39/141 styles affected.
2. Compat infra (oracle, report-core) does not reliably detect or score cross-entry behaviors.

Spec: docs/specs/cross-entry-fidelity.md (to be written)
Plan: .claude/plans/there-s-a-little-bug-shiny-walrus.md

## Tasks
- [x] Phase 0: Investigate engine config-gating; reproduce bug via oracle; Perplexity research
- [x] Phase 1: Write docs/specs/cross-entry-fidelity.md
- [x] Phase 2: Fix infra (fixtures, oracle scoring, audit-cross-entry-parity.js, report-core.js)
- [x] Phase 3: Audit showed migrated styles correct; fix chicago-author-date.yaml em-dash
- [x] Phase 4: Verify, regenerate baseline, run Rust gate

## Summary of Changes

Config model (Phase 0): Engine disambiguation is per-style config gated via Processing::AuthorDate.config(), which defaults all three strategies to true. No YAML changes needed for disambiguation — it was already working.

Em-dash bug fixed: styles/experimental/chicago-author-date.yaml was missing subsequent-author-substitute and subsequent-author-substitute-rule: complete-all. Added. Verified with citum render refs.

Infra improvements:
- Added 4 new test fixtures (ITEM-35-38): same-family-different-given and consecutive same-author
- Added 2 new citation test IDs: disambiguate-givenname, subsequent-author-consecutive (both STRICT)
- New script: scripts/audit-cross-entry-parity.js
- check-core-quality.js gains --cross-entry-audit flag
- Spec: docs/specs/CROSS_ENTRY_FIDELITY.md

Portfolio audit: 134 migrated styles checked, 0 offenders.
