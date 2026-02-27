---
# csl26-6bak
title: Refactor delimiter handling with hybrid enum approach
status: completed
type: feature
priority: high
created_at: 2026-02-07T06:44:21Z
updated_at: 2026-02-27T14:20:06Z
parent: csl26-u1in
---

Current delimiter handling is scattered across the codebase. The hybrid enum
exists in schema, but migrate/compiler and engine still contain duplicated
or ad-hoc conversion logic.

Implementation checklist:
- [x] Centralize delimiter parsing/normalization usage around
      `DelimiterPunctuation::from_csl_string` in schema consumers
- [x] Replace migrate `map_delimiter` hand-written matcher with shared schema
      conversion
- [x] Refactor engine citation delimiter normalization to use enum parsing
      (remove string-only `"none"` special-casing)
- [x] Add targeted tests for delimiter normalization edge cases
      (`none`, empty, trimmed values, custom)
- [x] Run Rust verification suite:
      `cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run`

Acceptance criteria:
- No duplicate delimiter mapping table remains in migrate compiler.
- Citation processing handles `none`/empty delimiters via shared enum
  semantics.
- Existing tests pass and new delimiter-focused coverage is present.

Refs: GitHub #126

## Summary of Changes

Centralized delimiter parsing/normalization around `DelimiterPunctuation::from_csl_string`. Replaced migrate `map_delimiter` hand-written matcher with shared schema conversion. Refactored engine citation delimiter normalization to use enum parsing (removed string-only `"none"` special-casing). Added targeted tests for delimiter normalization edge cases. All checklist items completed and verification suite passes.
