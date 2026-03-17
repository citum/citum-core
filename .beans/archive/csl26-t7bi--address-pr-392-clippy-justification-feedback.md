---
# csl26-t7bi
title: 'Address PR #392 clippy justification feedback'
status: completed
type: task
priority: normal
created_at: 2026-03-16T23:11:26Z
updated_at: 2026-03-16T23:49:44Z
---

Fix-not-suppress lints with ≤5 violations, rewrite remaining reasons with technically precise justifications, fix specific file issues.

## Summary of Changes

- Fixed 9 low-count clippy lints in code (unnested_or_patterns, ref_option_ref, wildcard_imports, doc_link_with_quotes, format_collect, fn_params_excessive_bools, implicit_clone, manual_let_else, self_only_used_in_recursion) and removed them from workspace allow list
- Rewrote all 24 remaining suppressions with technically precise reasons
- Removed redundant crate-level allow in csl-legacy/src/parser.rs
- Fixed wrong dead_code reason on assign_macro_order in citum-migrate/src/lib.rs
- Updated allow_attributes_without_reason comment to say 'any #[allow(...)] attribute' not just clippy

Commit: 80c8850
