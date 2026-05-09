---
# csl26-ukpz
title: format_document — Pandoc-equivalent DocumentOptions fields
status: todo
type: feature
priority: low
created_at: 2026-05-09T00:21:13Z
updated_at: 2026-05-09T00:21:13Z
---

Resolve the four Pandoc-equivalent `DocumentOptions` fields marked TBD in
`docs/specs/SERVER_INTERACTIVE_API.md` and wire them through the engine:

- `suppress_bibliography` (boolean)
- `link_citations` (boolean)
- `link_bibliography` (boolean)
- `notes_after_punctuation` (boolean)

These were intentionally omitted from the Tier 1 implementation in csl26-isrv
because the engine doesn't yet support them and the design is unresolved.

## Tasks

- For each field, decide whether Citum adopts the Pandoc semantic, a refined
  variant, or skips it.
- Add the necessary engine plumbing (e.g., processor flags, output
  post-processing).
- Extend `DocumentOptions` and the `format_document` flow.
- Add behaviour tests.

## Out of Scope

Streaming/incremental progress events (Tier 3) — separate spec work needed.
