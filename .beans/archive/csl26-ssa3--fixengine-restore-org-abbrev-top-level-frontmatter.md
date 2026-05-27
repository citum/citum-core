---
# csl26-ssa3
title: 'fix(engine): restore org-abbrev top-level frontmatter and surface parse errors'
status: completed
type: bug
priority: normal
created_at: 2026-05-27T17:41:11Z
updated_at: 2026-05-27T17:45:58Z
---

PR #817 review triage fixes: (1) org_abbreviation_memory dropped as top-level DocumentFrontmatter field — regression vs main; (2) parse_frontmatter silently drops all frontmatter on YAML error instead of surfacing it.

## Summary of Changes

- Restored `org_abbreviation_memory` as a top-level field in `DocumentFrontmatter` (regression vs main where it was removed in favour of options-only access)
- Added `frontmatter_org_abbreviation_memory` to `ParsedDocument`; wired through djot/mod.rs and markdown.rs with same precedence pattern as `integral_name_memory`
- Updated `pipeline.rs` effective-org-override to fall back to top-level frontmatter field when `options.org-abbreviation-memory` is absent
- Changed `parse_frontmatter` return type from `(Option<…>, &str)` to `(Result<Option<…>, String>, &str)`; callers split into `(frontmatter, frontmatter_error)`
- Added `frontmatter_error: Option<String>` to `ParsedDocument`; pipeline now calls `eprintln!` + `process::exit(1)` on parse error instead of silently discarding frontmatter
- Deferred: locale override alignment — see bean csl26-9l88
