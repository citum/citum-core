---
# csl26-aawl
title: Extract shared tier-1/session document pipeline
status: todo
type: task
created_at: 2026-07-04T02:42:25Z
updated_at: 2026-07-04T02:42:25Z
---

DocumentSession::render_citations duplicates ~130 lines of format_document_with_style (locale warning, warning scans, missing-ref filtering, nocite, 6-way OutputFormatKind dispatch repeated 5x). Extract a shared prepare_processor helper and a single format-dispatch helper. Fold in session loose ends: unused _style_input param in DocumentSession::new; diff_formatted_citations omits deletions; process_document_with_caller_blocks silently ignores frontmatter errors. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md findings 4, 17.
