---
# csl26-kuwj
title: Add ProcessingCustom.base schema field for custom-as-delta processing
status: draft
type: task
priority: normal
created_at: 2026-07-06T23:39:03Z
updated_at: 2026-07-06T23:39:07Z
parent: csl26-al39
---

Layer 2 of the delta-based processing extraction design (docs/reference/PROCESSING_MIGRATION.md,
csl26-vpae layer 1 landed in csl26-vpae). Allow ProcessingCustom to carry an optional
base: Processing field (named presets only); resolution overlays the sparse custom
fields onto base.config() instead of requiring an exact whole-struct match. YAML would
read processing: { base: author-date, sort: ... } -- the same delta philosophy as
extends:. Touches citum-schema-style (serde + resolution) and citum-engine call sites
of Processing::config(); requires just schema-gen in the same commit.

Needs a schema review pass first (deferred from csl26-vpae per its own sizing note).
