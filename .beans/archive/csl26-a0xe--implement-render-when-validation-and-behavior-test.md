---
# csl26-a0xe
title: Implement render-when validation and behavior tests
status: scrapped
type: task
priority: normal
tags:
    - schema
    - rendering
    - styles
created_at: 2026-07-13T16:51:06Z
updated_at: 2026-07-13T17:14:56Z
blocked_by:
    - csl26-qyub
---

docs/specs/RENDER_WHEN_CONTRACT.md v1.0 documents the render-when contract (retention decided in csl26-qyub) but the following is not yet implemented:

- [ ] Schema validation rejects `render-when: {}` and same-field present/absent conditions
- [ ] Behavior tests cover present, absent, combined-AND, and nested cases
- [ ] Regenerate `docs/schemas/style.json` and `docs/schemas/server.json`
- [ ] Promote RENDER_WHEN_CONTRACT.md Status from Draft to Active in the implementation commit

Related: csl26-qyub (decision + contract spec)

## Reasons for Scrapping

The user asked to fold this work into the same commit as csl26-qyub instead of deferring it. Validation, tests, schema-gen, and the Active-status promotion all landed there directly.
