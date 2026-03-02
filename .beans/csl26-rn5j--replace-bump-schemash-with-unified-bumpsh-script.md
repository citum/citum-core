---
# csl26-rn5j
title: Replace bump-schema.sh with unified bump.sh script
status: completed
type: task
created_at: 2026-03-02T22:24:12Z
updated_at: 2026-03-02T22:24:12Z
---

Replace scripts/bump-schema.sh with scripts/bump.sh supporting two-track versioning (schema + engine/workspace).

## Spec

- bump.sh [schema|engine|all] [patch|minor|major] [--dry-run]
- Default track when omitted: all (bumps both)
- schema track: updates default_version() in crates/citum-schema/src/lib.rs, tags schema-v*
- engine track: updates version = in root Cargo.toml, tags v*
- all: bumps both, single commit, both tags; aborts if either validation fails
- --dry-run: show computed new version + git log since last tag, exit without modifying anything
- Changelog since last tag: git log <tag>..HEAD --oneline

## Tasks
- [x] Write scripts/bump.sh
- [x] Delete scripts/bump-schema.sh
- [x] Update docs/reference/SCHEMA_VERSIONING.md to reference bump.sh

## Summary of Changes

Implemented unified scripts/bump.sh with full two-track versioning support:
- Supports schema/engine/all tracks with flexible argument parsing
- patch/minor/major bump types with proper SemVer computation
- --dry-run preview mode showing computed versions and git logs
- macOS/Linux sed compatibility
- Interactive commit confirmation with detailed diffs
- Automatic git tag creation for each track
- Validation: cargo test for schema, cargo fmt + clippy for engine
- Updated docs/reference/SCHEMA_VERSIONING.md with new usage examples
