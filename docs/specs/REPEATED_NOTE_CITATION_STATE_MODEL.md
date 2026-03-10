# Repeated Note Citation State Model Specification

**Status:** Active
**Version:** 1.1
**Date:** 2026-03-10
**Supersedes:** N/A
**Related:** `docs/specs/NOTE_STYLE_DOCUMENT_NOTE_CONTEXT.md`, `crates/citum-engine/src/processor/mod.rs`, `crates/citum-migrate/src/upsampler.rs`

## Purpose
Define a style-driven repeated-citation model for note processing that reliably distinguishes first citations, subsequent citations, and immediate repeats, while preserving authored-footnote integral `ibid` composition behavior.

## Scope
In scope:
- Position classification for `first`, `subsequent`, `ibid`, and `ibid-with-locator`.
- Position-spec resolution precedence and fallback behavior for citation rendering.
- Migration behavior for CSL `position` branches in citation layouts.

Out of scope:
- New position states such as `near-note` or `far-note`.
- Per-item position tracking inside multi-item citation clusters.
- Bibliography `subsequent-author-substitute` behavior.

## Design
1. Position classification uses four existing states: `First`, `Subsequent`, `Ibid`, `IbidWithLocator`.
2. For single-item note citations, immediate repeats are resolved by item identity plus locator comparison:
   - same item + same locator => `Ibid`
   - same item + different locator => `IbidWithLocator`
3. Multi-item citations remain `First`/`Subsequent` only in this wave.
4. Citation rendering resolves position-specific specs using precedence:
   - `Ibid` / `IbidWithLocator`: use `citation.ibid` when present
   - otherwise fallback to `citation.subsequent` when present
   - otherwise use base `citation`
5. Migration from CSL position branches maps citation layout branches to Citum citation specs:
   - `position="subsequent"` => `citation.subsequent.template`
   - `position="ibid"` or `position="ibid-with-locator"` => `citation.ibid.template`
   - `position="first"` or `<else>` => base `citation.template`
6. Supported migration shapes include nested/non-root position chooses and multiple independent position chooses inside one citation layout; sibling content is preserved in every variant.
7. If CSL position conditions are mixed with unsupported conditional attributes in the same decision tree, migration preserves existing fallback behavior and emits an explicit warning.

## Implementation Notes
- Authored-footnote integral `ibid` composition remains unchanged; it consumes improved position classification and spec fallback behavior.
- Mapping from CSL position conditions is citation-layout specific and does not alter bibliography mapping logic.

## Acceptance Criteria
- [ ] Same source cited consecutively with equal locator resolves to `Ibid`.
- [ ] Same source cited consecutively with changed locator resolves to `IbidWithLocator`.
- [ ] `resolve_for_position` falls back from missing `ibid` to `subsequent` for `Ibid` / `IbidWithLocator`.
- [ ] Note-style citations with only `subsequent` templates render immediate repeats without lexical `ibid`.
- [ ] CSL citation `position` branches are preserved into `citation.subsequent` and `citation.ibid` templates when the tree is supported, including nested and multi-choose layouts.
- [ ] Unsupported mixed CSL position trees keep existing fallback behavior and warn without panicking.
- [ ] Existing authored-footnote integral `ibid` composition tests continue to pass.

## Changelog
- v1.1 (2026-03-10): Clarified nested and multi-choose position-tree migration support while keeping mixed-condition trees on the warning path.
- v1.0 (2026-03-10): Initial draft, then activated with engine, schema fallback, and migration position-branch implementation.
