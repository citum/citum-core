---
# csl26-zrz5
title: implement `disambiguate.ignore` signature exclusion option
status: todo
type: task
priority: normal
created_at: 2026-05-29T11:14:56Z
updated_at: 2026-05-29T11:37:19Z
---

Implement the `disambiguate.ignore` list option specified in docs/specs/DISAMBIGUATION.md §6.

## Steps

- [ ] Add `ignore: Option<Vec<ReferenceVariable>>` to the `Disambiguation` struct in `crates/citum-schema-style/src/options/processing.rs:393`
- [ ] Thread into `DisambiguationFlags` and plumb into `build_group_key` in `crates/citum-engine/src/processor/disambiguation.rs`
- [ ] Regenerate JSON schemas: `cargo run --bin citum --features schema -- schema --out-dir docs/schemas`
- [ ] Add tests for `ignore: [original-date]` variant
- [ ] Set spec status to Active
