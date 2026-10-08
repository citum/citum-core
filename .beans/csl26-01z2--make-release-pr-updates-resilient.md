---
# csl26-01z2
title: Make release PR updates resilient
status: in-progress
type: bug
priority: high
tags:
    - release
    - ci
created_at: 2026-10-08T17:52:13Z
updated_at: 2026-10-08T18:01:00Z
---

Release runs 37378298134, 37781717085, and 37781714827 failed before updating release/next. The workflow must recover from schema-driven coverage-audit hash changes and concurrent merges without hiding audit failures.

## Checklist

- [x] Serialize release PR jobs and regenerate from latest main
- [x] Add schema-aware coverage manifest refresh support
- [x] Push and upsert the release PR before surfacing audit failure
- [x] Add workflow and script regression tests
- [x] Update the release workflow guide
- [x] Run focused validation and bean hygiene
- [ ] After landing, verify the release run updates PR #1289 from current main and reaches green CI

## Summary of Changes

- Serialized release PR generation and rebuilt it from the latest `origin/main` with refreshed inference.
- Made audit refresh fail open for branch push and PR upsert, then fail the job explicitly.
- Added schema-only manifest repinning, regression coverage, and recovery documentation.
- Validated the Python and Node suites, shell syntax, and actionlint. Zizmor reports only the pre-existing JSR dependency-install finding in the release workflow.
