---
# csl26-8uzr
title: Refactor compound grouping to entry sets
status: completed
type: task
priority: normal
created_at: 2026-03-05T19:58:00Z
updated_at: 2026-03-05T21:22:16Z
blocked_by:
    - csl26-zafv
---

Refactor compound numeric grouping from `group-key` field on InputReference to biblatex-style entry sets.

Current `group-key` is a field on every reference variant (15 structs). biblatex models this as a relationship (entry sets) which is cleaner and more aligned with our design principles.

## Tasks

- [ ] Design entry set data model (top-level `sets` in bibliography input?)
- [ ] Remove `group_key` from all InputReference variants
- [ ] Update `initialize_numeric_citation_numbers` to read from sets
- [ ] Update `merge_compound_entries` to read from sets
- [ ] Update tests and fixtures
- [ ] Consider cite-site override (mciteplus-style) as future extension

## Context

- Architecture doc: docs/architecture/CSL26_ZAFV_NUMERIC_COMPOUND_CITATIONS.md
- Parent: csl26-zafv
- Needs /dplan session before implementation
