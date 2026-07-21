---
# csl26-xg1o
title: Close commit-policy enforcement gap
status: completed
type: bug
priority: high
tags:
    - ci
    - hooks
created_at: 2026-07-21T20:21:42Z
updated_at: 2026-07-21T20:48:41Z
---

## Checklist

- [x] Add an always-on commit-policy workflow.
- [x] Validate squash-merge PR titles from the alint policy.
- [x] Add regression coverage and validate the workflow.
- [x] Transfer Tangled synchronization to csl26-pn7r.

## Context

GitHub squash merge created 41ce890a with a 69-character PR title, bypassing local hooks. The docs-gated alint CI job did not run for the Rust/beans change.

## Summary of Changes

Implemented and merged the always-on commit-policy workflow, PR-title validation, and event-range alint checks in PR #1080. Tangled synchronization is tracked separately in csl26-pn7r because it depends on PR #1081.
