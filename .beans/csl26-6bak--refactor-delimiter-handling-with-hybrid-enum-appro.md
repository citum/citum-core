---
# csl26-6bak
title: Refactor delimiter handling with hybrid enum approach
status: in_progress
type: feature
priority: high
created_at: 2026-02-07T06:44:21Z
updated_at: 2026-02-07T12:11:38Z
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
