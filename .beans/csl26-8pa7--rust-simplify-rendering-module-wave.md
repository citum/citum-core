---
# csl26-8pa7
title: 'Rust simplify: rendering module wave'
status: in-progress
type: task
priority: normal
created_at: 2026-03-14T22:18:20Z
updated_at: 2026-03-15T00:55:42Z
---

Ongoing simplify passes on citum-engine. This session: extracted rendering.rs (2422L) into rendering/ module dir (mod.rs 833L, grouped.rs 951L, helpers.rs 78L, tests.rs 570L).

## Split: contributor module (2026-03-14)

Extracted contributor.rs (1150L) into contributor/ dir:
- names.rs: all name formatting (497L)
- substitute.rs: author substitution + DRY fix (212L)
- labels.rs: role label resolution (154L)
- mod.rs: orchestrator + resolve_contributor_overrides (301L)

All 706 tests pass, clippy clean.

## 2026-03-14\n- crates/citum-engine/src/ffi.rs (847→570 mod.rs + 252 biblatex.rs): extracted biblatex module, parse_c_str! macro, parse_bibliography_json/load_style_yaml helpers; 32.7% reduction in mod.rs
