---
# csl26-fuw7
title: write versioning policy docs before first public release
status: todo
type: task
priority: deferred
created_at: 2026-02-24T17:28:17Z
updated_at: 2026-03-22T14:28:01Z
blocked_by:
    - csl26-yipx
---

Before any public release or first external user, finish the normative
versioning docs that sit above the operational release automation:

1. `docs/architecture/design/VERSIONING.md` — the compatibility contract for
   style authors and tool builders, covering major/minor schema changes, the
   deprecation policy (2-version window), the `citum-migrate` command
   specification, and how `deny_unknown_fields` interacts with forward
   compatibility.
2. `docs/architecture/SCHEMA_CHANGELOG.md` — machine-readable record of every
   schema field addition, deprecation, and removal keyed by version.

Operational release wiring now lives in `.github/workflows/release-plz.yml` and
`docs/reference/SCHEMA_VERSIONING.md`:

- root `CHANGELOG.md` is now workspace-wide
- `docs/schemas/*` is now the canonical committed schema artifact set
- schema bump automation uses the `Schema-Bump: patch|minor|major` footer

This bean remains the place for the deeper compatibility contract and durable
schema-history docs. Blocked by version validation task `csl26-yipx`.
