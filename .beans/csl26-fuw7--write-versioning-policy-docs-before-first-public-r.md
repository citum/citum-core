---
# csl26-fuw7
title: write versioning policy docs before first public release
status: todo
type: task
priority: deferred
created_at: 2026-02-24T17:28:17Z
updated_at: 2026-02-24T17:28:21Z
blocked_by:
    - csl26-yipx
---

Before any public release or first external user, write: (1) docs/architecture/design/VERSIONING.md - the compatibility contract for style authors and tool builders, covering: what constitutes a major vs minor schema change, the deprecation policy (2-version window), the csln migrate subcommand specification, and how deny_unknown_fields interacts with forward compatibility. (2) docs/architecture/SCHEMA_CHANGELOG.md - machine-readable record of every schema field addition, deprecation, and removal keyed by version. These docs make the versioning system legible and are prerequisites for any public announcement. Blocked by version validation task csl26-yipx.
