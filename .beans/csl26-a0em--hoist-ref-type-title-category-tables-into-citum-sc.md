---
# csl26-a0em
title: Hoist ref-type title-category tables into citum-schema-style
status: todo
type: task
priority: high
created_at: 2026-07-06T18:42:06Z
updated_at: 2026-07-06T18:42:06Z
parent: csl26-al39
---

Audit F1 (2026-07-06 migrate review): passes/sqi_refinement.rs::effective_title_rendering re-implements the engine's ref-type→title-category mapping (citum-engine/src/values/type_class.rs: title_category, container_title_category, parent_serial_title_category — all pub(crate)). SQI pruning must be the exact inverse of engine defaulting; divergent tables silently change rendering after pruning. Fix: move the classification functions into citum-schema-style (next to TitleConfig), re-export for the engine, and have sqi_refinement consume the shared table. Add a cross-crate test asserting prune-then-render equals render-unpruned for every classified type.
