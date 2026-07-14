---
# csl26-nnkf
title: 'Review PR #1052: cross-role contributor lists'
status: completed
type: task
priority: high
tags:
    - review
    - contributors
created_at: 2026-07-14T21:13:37Z
updated_at: 2026-07-15T11:22:27Z
---

Careful multi-pass review of PR #1052 (specs, schema, engine, migration/tests, mechanical+hygiene) per plan this-pr-got-away-snug-kay.md. Findings report in chat for triage.

- [x] Pass 1: specs vs APA sources
- [x] Pass 2: schema surface
- [x] Pass 3: engine semantics
- [x] Pass 4: migration/tests/fixtures
- [x] Pass 5: /code-review supplement + verification + hygiene
- [x] Findings report delivered

Interim: pre-commit gate green (1971/1971), schema-gen no drift, oracle 20/20 + 45/46 (single pre-existing standard-ref failure; PR counts stale). Key manual findings: matching.rs not on effective_primary resolver; migrate drops translator label for split editor/translator; invented locale abbreviations (wrtr.) need content sign-off.

## Summary of Changes

Review-only (no code changes). Delivered 19-finding report in chat: 7 confirmed correctness findings (merged-list role-substitute suppression missing vs spec; Matcher fork off effective_primary; role_omitted bypass in structural label paths x2; empty-names substitute kills fallback chain + disambiguation key; sort/render divergence on empty merged component; migrate drops translator label for split editor/translator; 3 scalar-assumption stragglers), 1 locale-content decision (invented writer abbreviations), spec/doc gaps, efficiency bean candidates (uncached primary resolution in sort comparators), cleanup bundle. Gate green 1971/1971; schema-gen clean; oracle 20/20+45/46 (pre-existing standard-ref failure; PR counts stale). Awaiting Bruce's triage.

## Fix wave (2026-07-15)

Bruce triaged: polish PR maximally. All 7 correctness findings + spec gaps + locale content fixed in-PR via two Sonnet batches plus manual Matcher config fix (effective config, field rename). Deferred: csl26-78ds (efficiency), csl26-tsmg (cleanup). Gate 1986/1986 green, schema-gen no drift, oracle 20/20 + 45/46 unchanged. Fixes squashed into feat commit via jj; PR body refreshed; pushed with lease (CONFIRM given).
