---
# csl26-yipx
title: wire Style.version to schema validation in citum check
status: todo
type: feature
priority: normal
created_at: 2026-02-24T17:28:12Z
updated_at: 2026-02-24T17:28:12Z
---

Style.version exists as a string field but is completely ignored at parse time. Add a SchemaVersion struct that parses the X.Y string into (major, minor) integers. Wire it into the `citum check` command to: (1) emit version info in structured JSON output, (2) reject styles where major > current supported major with a clear error message, (3) warn (but succeed) when minor > current supported minor. This gives us the gating mechanism before we need it and makes `citum check` useful for tooling. No behavior change for existing valid styles.
