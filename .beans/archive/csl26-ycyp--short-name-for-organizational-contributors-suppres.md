---
# csl26-ycyp
title: Short-name for organizational contributors + suppress-author memory fix
status: completed
type: epic
priority: normal
created_at: 2026-05-15T11:09:11Z
updated_at: 2026-05-15T11:19:28Z
---

Three related items in one PR: (1) keep abbreviation-map as a flat string map and align the JSON schema, (2) add short-name field to SimpleName for organizational acronyms (WHO, etc.) with first-mention parenthetical rendering, (3) treat suppress-author citations as integral for name-memory purposes. Spec: docs/specs/SHORT_NAME.md

## Summary of Changes\n\nAll five child tasks completed in one PR:\n1. AbbreviationMap: flat transparent map schema regenerated with serde compatibility coverage\n2. Spec: docs/specs/SHORT_NAME.md\n3. SimpleName.short_name + FlatName.short_name schema fields\n4. Engine: integral name state + short_name_display wired into NameFormatContext; first-mention parenthetical and subsequent short-only rendering for literal names\n5. suppress-author treated as integral for name-memory tracking
