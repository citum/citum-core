---
# csl26-bxnc
title: Add org-mode input/output support
status: completed
type: feature
priority: normal
created_at: 2026-03-03T23:31:35Z
updated_at: 2026-03-03T23:39:52Z
---

Add orgize as a regular dependency to citum-engine. Implement:
1. AnnotationFormat::Org variant in io.rs
2. render_org_inline in rich_text.rs (parse org annotations via orgize)
3. crates/citum-engine/src/render/org.rs — OrgOutputFormat implementing OutputFormat (emit *bold*, /italic/, [[url][desc]], =verbatim=)
4. Wire Org into bibliography annotation dispatch
5. Tests for both parse and render directions

## Summary of Changes

- Added `orgize = "0.9"` as a regular dependency in `citum-engine`
- Added `AnnotationFormat::Org` variant in `io.rs`
- Added `render_org_inline` in `rich_text.rs` — stack-based orgize event walker mapping bold/italic/verbatim/links/text to OutputFormat methods
- Created `crates/citum-engine/src/render/org.rs` with `OrgOutputFormat` implementing OutputFormat (emph→//, strong→**, small_caps→~~, link→[[url][desc]])
- Wired `AnnotationFormat::Org` into bibliography annotation dispatch in `bibliography.rs`
- 278/278 tests pass
