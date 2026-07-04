---
# csl26-dog9
title: 'Design: explicit render-run state for Processor'
status: todo
type: task
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-04T02:42:26Z
---

Processor uses RefCell interior mutability (citation_numbers, cited_ids, first_note_by_id, dynamic compound maps), making &self render methods order-dependent and non-idempotent, with invariants recorded only in comments. Design an explicit per-run state object so ordering contracts are typed and Processor becomes reusable/shareable. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 6.
