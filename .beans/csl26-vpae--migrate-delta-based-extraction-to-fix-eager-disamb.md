---
# csl26-vpae
title: 'migrate: delta-based extraction to fix eager disambiguation-default materialization'
status: todo
type: bug
priority: normal
created_at: 2026-06-20T18:51:16Z
updated_at: 2026-06-20T18:51:16Z
---

The migrate options-extractor eagerly materializes disambiguation defaults (e.g. year_suffix: unwrap_or(true)) before comparing against named Processing presets, so fold_to_named_processing can never match. Fix: leave unset CSL attributes as None and record only explicit overrides that differ from the preset — the same delta philosophy as extends:. Design decisions captured in docs/reference/PROCESSING_MIGRATION.md (csl26-1861); implementation deferred from that bean's doc-only scope.
