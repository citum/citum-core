---
# csl26-p1rn
title: 'Phase 1: GitHub org transfer and crate rename'
status: completed
type: milestone
priority: normal
created_at: 2026-02-22T00:00:00Z
updated_at: 2026-03-01T13:06:14Z
blocking:
    - csl26-modz
    - csl26-p0dc
---

Rename crates and transfer repositories under the `citum` GitHub organization.

Execute at a natural pause between active style-migration waves; renaming
during active wave work corrupts path references in agent skills, oracle
scripts, and bean task files.

## Prerequisites

* csl26-p0dc complete (clean boundary before rename)
* Current migration wave concluded

## Steps

1. Create `citum` GitHub organization
2. Transfer `bdarcus/csl26` -> `citum/citum-core`
3. Transfer `style-hub` -> `citum/citum-hub`
4. Update `package.name` in each Cargo.toml (see CITUM_MODULARIZATION.md for mapping)
5. Rename crate directories to match new names
6. Update all `path = "../..."` references in workspace Cargo.toml
7. Set `publish = true` for: citum-schema, citum-engine, csln-edtf
8. Keep `publish = false` for: citum-migrate, csl-legacy, citum-analyze
9. Update CLAUDE.md paths, oracle scripts, agent skill references

## Note

Do not publish to crates.io during this phase. Use GitHub as distribution
until schema reaches 1.0 stability.

Refs: csl26-modz, docs/architecture/CITUM_MODULARIZATION.md
