---
# csl26-jt3h
title: Improve public citation and server schema integration docs
status: completed
type: task
priority: normal
tags:
    - docs
    - schema
    - dx
created_at: 2026-10-27T12:54:57Z
updated_at: 2026-10-27T15:27:19Z
parent: csl26-e0av
---

## Goal

Citation and server schemas serve application developers. Citation fields are described, while the server schema has 13 property paths without effective descriptions; neither has a concise request-oriented newcomer path.

## Acceptance criteria

- [x] Document common citation and JSON-RPC tasks with complete requests and responses.
- [x] Resolve or explicitly justify the 13 server description gaps.
- [x] Validate examples against citation.json and server.json.
- [x] Build, inspect, and verify the deployed pages.

## Summary of Changes

- Added the public citation and JSON-RPC integration guide with schema-validated citation, render, and format-document examples.
- Added effective descriptions for all 13 server schema property gaps without changing runtime or wire shape.
- Added guide links from the schema, reference, and server README entry points.
- Regenerated schemas and data-model references; verified examples, tests, docs build, links, and responsive pages.
