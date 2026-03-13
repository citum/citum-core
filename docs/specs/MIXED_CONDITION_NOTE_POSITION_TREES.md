# Mixed-Condition Note Position Trees Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-03-10
**Supersedes:** None
**Related:** `csl26-3go0`, `.beans/archive/csl26-qfa3--upgrade-note-styles-for-repeated-position-override.md`, `.beans/archive/csl26-494i--extend-migration-for-complex-citation-position-cho.md`

## Purpose
Define how XML-mode citation migration specializes legacy CSL `choose` trees that mix `position` predicates with other branch conditions so note-style repeated citation overrides can be emitted without flattening non-position sibling content.

## Scope
In scope: XML-template migration in `citum-migrate` for note-style citation trees where `position` is combined with `type`, `variable`, `locator`, `is-numeric`, or `is-uncertain-date` and can be specialized by stripping `position` while preserving the remaining branch semantics. Out of scope: general boolean normalization across unrelated CSL conditions, schema changes, CLI changes, and style-local cleanup beyond documenting residual divergences.

## Design
`Upsampler::extract_citation_position_templates` must keep the existing fast path for pure position-only trees. For mixed trees, the migrator must rewrite each requested position variant by cloning the legacy `choose`, removing branches whose `position` does not match the target, stripping `position` from matching branches, recursively rewriting retained children, and preserving all non-position predicates and sibling branches that never referenced `position`.

The rewrite may collapse only trivial results. A rewritten tree with no surviving content emits nothing. A tree that reduces to exactly one surviving unconditional branch and no remaining `else` or fallback branches emits that branch's children inline. Otherwise the tree remains a `choose` so non-position logic is preserved in the compiled Citum condition block.

Unsupported cases remain fallback-only. Unknown `position` tokens, ambiguous duplicate pure position-only branches, or rewritten trees that cannot be normalized to a single fallback path must mark the position extraction unsupported and leave XML compilation on the base citation template only. This includes rewritten trees that retain both an unconditional branch and an `else` branch, or any other shape that would leave multiple fallback paths after specialization.

The emitted override model does not change. Base or first-position content continues to populate `citation.template`, subsequent content populates `citation.subsequent`, and both `ibid` tokens continue to merge into one `citation.ibid` override.

## Implementation Notes
The initial acceptance target is the blocked note-family cluster from `csl26-3go0`: `chicago-notes`, `chicago-notes-bibliography-17th-edition`, `mhra-notes`, `mhra-notes-publisher-place`, `mhra-notes-publisher-place-no-url`, `new-harts-rules-notes`, `new-harts-rules-notes-label-page`, and `new-harts-rules-notes-label-page-no-url`.

Regression coverage must include both unit tests for the upsampler rewrite and XML compilation tests that model Chicago/MHRA note shapes with mixed `position` branches.

## Acceptance Criteria
- [ ] `citum-migrate --template-source xml` emits repeated-position overrides for the blocked note families when their legacy CSL encodes `position` alongside sibling `type`, `variable`, or `locator` conditions in one `choose` tree.
- [ ] Mixed-tree specialization preserves non-position sibling content instead of collapsing the full branch tree into the base citation template.
- [ ] Pure position-only duplicate branches and unknown `position` tokens still fall back safely without partial overrides.
- [ ] Regression tests cover mixed `position + type`, `position + variable`, `position + locator`, and one still-unsupported shape.

## Changelog
- v1.0 (2026-03-10): Activated with the initial mixed-condition note-position migration implementation.
