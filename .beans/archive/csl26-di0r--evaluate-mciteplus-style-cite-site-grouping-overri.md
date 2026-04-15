---
# csl26-di0r
title: Evaluate mciteplus-style cite-site grouping overrides
status: completed
type: task
priority: normal
created_at: 2026-03-05T21:27:17Z
updated_at: 2026-04-15T00:08:21Z
---

Evaluate and design a mechanism for "cite-site" compound grouping overrides,
inspired by the LaTeX `mciteplus` package. This follows the initial
implementation of static relational compound sets in the engine.

See spec: `docs/specs/CITE_SITE_COMPOUND_GROUPING.md`

## Next Steps

- [x] Survey existing chemistry style requirements (RSC, ACS) for dynamic grouping edge cases.
- [x] Implement a prototype in `citum-engine` following the citation-level grouping design.

## Summary of Changes

Added cite-site dynamic compound grouping via Citation::grouped flag.

- Schema: grouped bool field on Citation (serde skip when false)
- Engine: resolve_dynamic_group() registers dynamic sets at cite-time, updating
  citation numbers for tails and populating four new RefCell index fields
- Rendering: render_citation_content merges static + dynamic maps before building
  CompoundRenderData so sub-label logic works transparently
- Conflict resolution: static sets always win; first-occurrence wins for dynamic
- Non-numeric styles: silent no-op (no compound-numeric config = early return)
- Spec: docs/specs/CITE_SITE_COMPOUND_GROUPING.md promoted Draft to Active
- Tests: 4 new unit tests covering all spec-mandated cases
