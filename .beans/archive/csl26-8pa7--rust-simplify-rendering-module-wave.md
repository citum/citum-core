---
# csl26-8pa7
title: 'Rust simplify: rendering module wave'
status: completed
type: task
priority: normal
created_at: 2026-03-14T22:18:20Z
updated_at: 2026-03-15T10:48:57Z
---

Ongoing simplify passes on citum-engine. This session: extracted rendering.rs (2422L) into rendering/ module dir (mod.rs 833L, grouped.rs 951L, helpers.rs 78L, tests.rs 570L).

## Split: contributor module (2026-03-14)

Extracted contributor.rs (1150L) into contributor/ dir:
- names.rs: all name formatting (497L)
- substitute.rs: author substitution + DRY fix (212L)
- labels.rs: role label resolution (154L)
- mod.rs: orchestrator + resolve_contributor_overrides (301L)

All 706 tests pass, clippy clean.

## 2026-03-14
- crates/citum-engine/src/ffi/mod.rs (847→570 + 252 biblatex.rs): extracted biblatex module, parse_c_str! macro, parse_bibliography_json/load_style_yaml helpers; 32.7% reduction in mod.rs

## 2026-03-15 wave planned: bibliography.rs (863L), rendering/mod.rs (827L), document/djot.rs (641L)

## 2026-03-15 wave completed

Split bibliography.rs (863L) into bibliography/ dir:
- mod.rs (199L): core entry processing + facade
- compound.rs (212L): compound-entry merging
- grouping.rs (424L): grouped rendering + headings

Extracted collapse.rs (183L) from rendering/mod.rs (827→646L).
Minor djot.rs cleanup: derive(Default) on DjotParser.
All 706 tests pass, clippy clean.

## Summary of Changes

Split three processor files into submodules:
- bibliography.rs (863L) → bibliography/{mod,compound,grouping}.rs
- rendering/mod.rs (827→646L) → extracted collapse.rs (183L)
- djot.rs: derive(Default) cleanup
PR #376.
