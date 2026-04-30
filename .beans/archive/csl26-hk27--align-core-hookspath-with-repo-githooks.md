---
# csl26-hk27
title: Align core hooksPath with repo githooks
status: completed
type: task
priority: normal
tags:
    - infra
created_at: 2026-03-13T22:58:00Z
updated_at: 2026-04-30T20:30:14Z
---

This checkout has the real commit hook in `.githooks/commit-msg`, but Git is
currently configured with `core.hooksPath = .git/hooks`, which only contains
sample hooks.

## Objective

Make the repo's commit hook policy enforceable by default in active checkouts.

## Scope

- decide whether the project should standardize on `.githooks`
- add or update bootstrap instructions so contributors set
  `git config core.hooksPath .githooks`
- consider adding a guard or setup script that verifies the active hooks path
- verify that the documented commit rules match the hook behavior

## Checklist

- [x] Confirm the intended canonical hooks directory for this repo
- [x] Add setup documentation for configuring `core.hooksPath`
- [x] Add a lightweight verification step or bootstrap helper if needed
- [x] Verify commit-msg enforcement matches the documented 50/72 rule
- [x] Archive this bean once the repo bootstrap path is unambiguous

## Summary of Changes

All objectives met as of ~2026-04-11:
- `scripts/install-hooks.sh` sets `core.hooksPath = .githooks`
- `.githooks/` contains commit-msg, pre-commit, pre-push hooks
- Bootstrap docs reference install-hooks.sh
- commit-msg hook enforces 50-char subject limit (matches CLAUDE.md)
