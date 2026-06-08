---
# csl26-o8ji
title: Consolidate bibliography block rendering to engine core
status: todo
type: task
priority: high
created_at: 2026-06-08T12:57:00Z
updated_at: 2026-06-08T12:57:32Z
---

Two separate implementations of sectional bibliography rendering currently exist in the engine:

1. **Document pipeline path** (`processor/document/pipeline.rs`): parses `:::bibliography{...}` markers from Djot/Markdown content and replaces them inline. Entry point: `process_document_with_bibliography_blocks`.

2. **API path** (`api/document.rs`): `format_bibliography_blocks` accepts a `Vec<BibliographyBlockRequest>` and returns `Vec<FormattedBibliographyBlock>`. Introduced in feat(server): add bibliography blocks (PR #884).

Both ultimately call `render_document_bibliography_block` on the `Processor`, but the surrounding orchestration is duplicated and the two entry points are not composable.

## Goal

Move the sectional bibliography concept fully into the engine core as a single, well-defined primitive. The CLI and server should be thin wrappers over that primitive — not independent implementations.

## Work items

- [ ] Audit the two paths and define the canonical engine-level API for sectional bibliography rendering
- [ ] Consolidate into a single code path in the engine (document pipeline and API path should share the same rendering logic)
- [ ] Update CLI (`render doc`) to expose sectional bibliographies via the engine primitive
- [ ] Update server to confirm it uses the same primitive (should be a no-op or minor wiring change)
- [ ] Remove any duplicated orchestration code

## Context

Discovered during review of PR #884. The server-side `bibliography_blocks` field was added without wiring the CLI to the same mechanism, revealing the DRY violation. See PR #884 discussion.

## Related

- [[csl26-k9y0]] fixed the assigned-dedup problem in the fenced-div pipeline path — the same mechanism this PR added to the API path. Consolidation should carry that fix through to the unified primitive.
