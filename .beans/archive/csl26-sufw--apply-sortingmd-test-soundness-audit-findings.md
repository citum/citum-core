---
# csl26-sufw
title: Apply SORTING.md test-soundness audit findings
status: completed
type: task
priority: high
created_at: 2026-06-12T12:00:51Z
updated_at: 2026-06-12T12:16:49Z
---

Trim vacuous/redundant tests, fix broken assertions, add coverage-gap tests, clarify spec advisories, and mark ledger addressed. Branch: test-soundness-sorting-apply-findings.

## Summary of Changes

- Skill: revised Step 5 to auto-proceed (feat commit already merged)
- Trimmed 4 vacuous/redundant tests from sort_oracle.rs
- Fixed 2 broken assertions; replaced deleted test with substantive version
- Added 5 new coverage-gap tests across sort_oracle.rs, bibliography.rs, citations.rs
- Clarified 2 advisory spec silences in docs/specs/SORTING.md
- Marked ledger row addressed; added Resolution section to audit record
