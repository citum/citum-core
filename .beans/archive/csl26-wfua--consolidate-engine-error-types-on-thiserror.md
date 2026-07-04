---
# csl26-wfua
title: Consolidate engine error types on thiserror
status: completed
type: task
priority: normal
tags:
    - errors
    - types
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-04T19:51:21Z
parent: csl26-8m2p
---

FormatDocumentError and DocumentSessionError hand-roll Display/Error despite thiserror being a dependency; ProcessorError variants are stringly (ParseError("BIBLIOGRAPHY"/"FRONTMATTER", ...) used for validation and frontmatter). Add typed variants and derive consistently. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 13.

## Summary of Changes

Typed citum-engine parse errors into frontmatter, compound-set validation, and named reference parse variants; moved document/session API errors onto thiserror derives; updated engine and citum-io call sites plus tests for the new public error shape.
