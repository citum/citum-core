---
# csl26-wfua
title: Consolidate engine error types on thiserror
status: todo
type: task
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-04T02:42:26Z
---

FormatDocumentError and DocumentSessionError hand-roll Display/Error despite thiserror being a dependency; ProcessorError variants are stringly (ParseError("BIBLIOGRAPHY"/"FRONTMATTER", ...) used for validation and frontmatter). Add typed variants and derive consistently. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 13.
