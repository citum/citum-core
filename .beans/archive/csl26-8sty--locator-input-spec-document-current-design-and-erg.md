---
# csl26-8sty
title: 'Locator input spec: document current design and ergonomics options'
status: completed
type: task
priority: normal
created_at: 2026-06-02T11:09:58Z
updated_at: 2026-06-02T11:11:44Z
---

Write docs/specs/LOCATOR_INPUT.md documenting the three input surfaces (structured single, structured compound, Djot/prose), the canonical CitationLocator model, the plural-detection heuristic, and all ergonomic options (A0–A3, B0–B1, C). No code changes — doc PR only.

## Summary of Changes

Created `docs/specs/LOCATOR_INPUT.md`:
- Documents three input surfaces: structured YAML/JSON, Djot/prose, forward-compat bridge.
- Specifies the `CitationLocator` / `LocatorSegment` / `LocatorValue` data model with code anchors.
- Documents the plurality-detection heuristic (`is_plural`) and its known false-positive (`"figure A-3"`).
- Records the compact-map syntax as reverted (historical note) with a revival option (B1).
- Presents options A0–A3 (heuristic), B0–B1 (compact map), C (asymmetry) with tradeoffs.

Cross-linked in `docs/specs/LOCATOR_RENDERING.md` Related and `docs/specs/README.md` index.
