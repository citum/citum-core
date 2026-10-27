---
# csl26-76n8
title: Improve public style and locale schema authoring docs
status: completed
type: task
priority: normal
tags:
    - docs
    - schema
    - dx
created_at: 2026-10-27T12:54:57Z
updated_at: 2026-10-27T16:36:51Z
parent: csl26-e0av
---

## Goal

Style and locale schemas serve style authors but currently split guidance across raw JSON, the style guide, and locale references. The audit found 19 style and 8 locale property paths without effective descriptions.

## Acceptance criteria

- [x] Add task-oriented entry points for style and locale authors.
- [x] Resolve or explicitly justify every missing effective description.
- [x] Add schema-validated style and locale examples.
- [x] Build, inspect, and verify the deployed pages.

## Summary of Changes

Added task-oriented style and locale authoring entry points with complete
schema-validated examples. Documented every previously missing effective field
description, regenerated the published schemas, and verified that normalized
wire shapes match main. Built the public pages in a clean scratch worktree,
checked internal links, and inspected the affected pages at desktop and mobile
widths.
