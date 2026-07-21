---
# csl26-xg1o
title: Close commit-policy enforcement gap
status: in-progress
type: bug
priority: high
tags:
    - ci
    - hooks
created_at: 2026-07-21T20:21:42Z
updated_at: 2026-07-21T20:42:03Z
---

## Checklist

- [x] Add an always-on commit-policy workflow.
- [x] Validate squash-merge PR titles from the alint policy.
- [x] Add regression coverage and validate the workflow.
- [ ] Audit and synchronize the Tangled mirror once.

## Context

GitHub squash merge created 41ce890a with a 69-character PR title, bypassing local hooks. The docs-gated alint CI job did not run for the Rust/beans change.
