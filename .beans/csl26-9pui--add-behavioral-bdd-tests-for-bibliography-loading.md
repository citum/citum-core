---
# csl26-9pui
title: Add behavioral BDD tests for bibliography loading
status: in-progress
type: task
priority: normal
tags:
    - testing
created_at: 2026-03-14T15:51:53Z
updated_at: 2026-05-01T10:40:40Z
---

The io.rs parse branch tests added in PR #365 are unit tests against private helpers. Add behavioral coverage for bibliography loading through the public API (load_bibliography_with_sets or CLI), using the project's BDD naming convention (describe_X::it_Y). Cover: CSL-JSON array, Citum YAML, wrapped legacy format, IndexMap format. Should appear in the behavior coverage report.
