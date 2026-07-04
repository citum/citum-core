---
# csl26-qi7l
title: Reduce per-component config clones in render path
status: todo
type: task
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-04T02:42:26Z
---

Each ProcTemplateComponent carries an owned Config clone plus BibliographyConfig clone; RenderOptions/Renderer construction clones BibliographyConfig too. O(entries x components) deep clones per render pass. Borrow or Rc/Arc the configs. Watch cargo bench --bench rendering. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 9.
