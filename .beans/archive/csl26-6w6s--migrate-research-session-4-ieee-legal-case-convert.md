---
# csl26-6w6s
title: 'migrate-research: session 4 — IEEE legal-case converter-gap'
status: completed
type: task
priority: normal
created_at: 2026-03-28T00:04:15Z
updated_at: 2026-03-28T00:15:20Z
---

Session 4 on branch fix/migrate-research-session-4. Continue from session-3: attempt-2 (IEEE legal-case double-quote + vol.dup converter-gap). Expand corpus if IEEE is fixed to check for new converter gaps.

## Summary of Changes

Attempt-1: Fixed IEEE legal_case bibliography rendering (+1 scenario: 32/33 → 33/33).

Changes: normalize_legal_case_type_template now strips title quote flags, authority variable (all styles), Term::In, Title::ParentMonograph, and duplicate Number::Volume from inferred legal_case templates.

Commit: 7d9a5d4b on branch fix/migrate-research-session-4.
All remaining corpus failures are engine-gaps (patent number) or complex converter issues deferred to future sessions.
