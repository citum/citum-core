---
# csl26-rz3s
title: 'Schema discovery: server.json + /schemas/ index page'
status: completed
type: task
priority: normal
created_at: 2026-05-14T11:21:04Z
updated_at: 2026-05-14T11:25:27Z
---

Split citum-server schema feature, add server.json to CLI export, create docs/schemas/index.html, update docs/index.html teaser. PR workflow.

## Summary of Changes

- Split citum-server schema feature: schema-types (schemars only) vs schema (http + schema-types)
- Added Server variant to SchemaType; schema --out-dir now exports server.json (7 schemas total)
- Created docs/schemas/index.html cataloguing all 7 schemas in 3 groups
- Replaced docs/index.html schema section with compact teaser + link to /schemas/
- No CI/release workflow changes needed
- PR #693
