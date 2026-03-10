# Document Note Stream Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-03-10
**Supersedes:** N/A
**Related:** `crates/citum-engine/src/processor/document/mod.rs`, `crates/citum-engine/src/processor/mod.rs`

## Purpose
Define normative behavior for document-level note processing in note styles:
how inline citations become generated notes and how citations inside authored
notes participate in note context.

## Scope
In scope:
- Note-style citation context across inline prose citations and manual footnotes.
- Note ordering and position semantics in document mode.

Out of scope:
- Style template authoring guidance.
- Non-document citation APIs.
- Markdown manual-footnote parsing parity.

## Design
1. For note styles, citations in inline prose and citations inside authored footnotes participate in one shared note-context stream for position detection.
2. The shared note-context stream is strictly document-order and page-agnostic.
3. Inline prose citations in note styles render as generated note references (for example `[^citum-auto-3]`), and generated note definitions are emitted as one contiguous block after body content and before any bibliography section.
4. Generated note identifiers are internal implementation details, not a stable external contract.
5. Citations already inside authored footnotes render in place and are never converted to generated note definitions.
6. A footnote definition is one note slot in the shared stream even when it contains multiple citations and non-citation content.
7. Position-aware rendering uses the shared stream and the style’s position templates (`first`, `subsequent`, `ibid`).

## Note Ordering and Position Rules
1. The note sequence is linear and derived from document order, not page layout.
2. Generated note references and authored footnote references share this same linear sequence.
3. Position resolution (`first`, `subsequent`, `ibid`) is computed against that shared sequence.
4. `Ibid` eligibility is determined by the immediately preceding note context in that sequence.
5. `Ibid` applies only when the immediately preceding note context resolves to a single source.
6. When a footnote contains multiple citations, they share one note slot while still resolving per-citation position in source order within that slot.

## Implementation Notes
- Citation position logic for document rendering must remain independent of page
  layout or pagination.
- Styles may impose additional `ibid` constraints (for example, disallowing
  `ibid` across page breaks) using pagination information provided by downstream
  layout engines.

## Acceptance Criteria
- [ ] Mixed manual and generated note citations share one document-order position stream for `first/subsequent/ibid`.
- [ ] Generated note definitions are emitted as one contiguous block after body content.
- [ ] Citations inside manual footnotes render in place and do not generate duplicate auto footnotes.
- [ ] Manual and generated note sequencing is page-agnostic.

## Changelog
- v1.0 (2026-03-10): Initial active version.
