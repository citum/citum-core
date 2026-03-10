# Document Note Stream Specification

**Status:** Active
**Version:** 1.1
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
8. For integral citations rendered in authored footnotes, rendering is two-channel:
   narrative anchor text and reduced citation text. `ibid` and `ibid-with-locator`
   apply only to the reduced citation channel and do not suppress the anchor.

## Note Ordering and Position Rules
1. The note sequence is linear and derived from document order, not page layout.
2. Generated note references and authored footnote references share this same linear sequence.
3. Position resolution (`first`, `subsequent`, `ibid`) is computed against that shared sequence.
4. `Ibid` eligibility is determined by the immediately preceding note context in that sequence.
5. `Ibid` applies only when the immediately preceding note context resolves to a single source.
6. When a footnote contains multiple citations, they share one note slot while still resolving per-citation position in source order within that slot.

## Integral `Ibid` in Authored Notes
1. In note styles, if a citation is integral, appears in authored footnote content, and resolves to `ibid` or `ibid-with-locator`, the default output is `anchor + (reduced citation)`.
2. Default reduced-citation output is locale/style term text for `ibid`, and that same term plus locator for `ibid-with-locator`.
3. If the style defines an explicit position+integral template (for example `citation.ibid.integral.template`, including localized integral templates under `citation.ibid.integral.locales`), that explicit template overrides the default composition rule.
4. If anchor text cannot be rendered, the processor falls back to reduced citation output only.
5. The processor must never concatenate anchor and reduced citation tokens without spacing or punctuation (for example, `SmithIbid` is invalid).
6. The processor must not inject `ibid` punctuation; punctuation and spelling come from style or locale term data.

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
- [ ] Integral authored-footnote `ibid` renders as `anchor + (ibid term)` by default, not `ibid`-only.
- [ ] Integral authored-footnote `ibid-with-locator` preserves locator in reduced citation output.
- [ ] No concatenation regressions occur (`KuhnIbid`, `SmithIbid`).
- [ ] Existing non-integral `ibid` behavior remains unchanged.

## Changelog
- v1.1 (2026-03-10): Added authored-note integral `ibid` composition rule and acceptance criteria.
- v1.0 (2026-03-10): Initial active version.
