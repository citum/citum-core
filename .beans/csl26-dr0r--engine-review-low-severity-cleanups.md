---
# csl26-dr0r
title: Engine review low-severity cleanups
status: todo
type: task
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-04T02:42:26Z
---

Batch of low findings from the engine review: collection-editor lookups via ContributorRole::Unknown (add first-class role); get_variable_key builds variable-once keys from Debug formatting (use explicit names); with_compound_sets silently discards invalid sets without a warning; Renderer is pub with all-pub fields incl. RefCell scratch state (tighten to pub(crate)). docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md findings 14, 15, 16, 18.
