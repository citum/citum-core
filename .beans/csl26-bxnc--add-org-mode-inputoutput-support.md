---
# csl26-bxnc
title: Add org-mode input/output support
status: in-progress
type: feature
created_at: 2026-03-03T23:31:35Z
updated_at: 2026-03-03T23:31:35Z
---

Add orgize as a regular dependency to citum-engine. Implement:
1. AnnotationFormat::Org variant in io.rs
2. render_org_inline in rich_text.rs (parse org annotations via orgize)
3. crates/citum-engine/src/render/org.rs — OrgOutputFormat implementing OutputFormat (emit *bold*, /italic/, [[url][desc]], =verbatim=)
4. Wire Org into bibliography annotation dispatch
5. Tests for both parse and render directions
