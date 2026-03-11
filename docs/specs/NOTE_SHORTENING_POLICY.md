# Note Shortening Policy Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-03-11
**Supersedes:** None
**Related:** `.beans/csl26-t79d--spec-normative-note-shortening-policy.md`, `docs/specs/REPEATED_NOTE_CITATION_STATE_MODEL.md`, `docs/specs/NOTE_POSITION_AUDIT.md`

## Purpose
Define a normative policy for repeated-note and shortened-note behavior across
note styles so Citum can distinguish current shipped behavior from
style-guide-intended behavior when implementing, auditing, and testing note
citations.

## Scope
In scope: style-family classification for repeated-note behavior, the split
between style-declared formatting and processor-managed state, and how future
audits should represent normative expectations without quoting copyrighted style
manual language.

Out of scope: changing shipped style behavior in this draft, publishing manual
excerpts, or deciding unresolved prose/integral repeated-citation semantics
without additional primary-source review.

## Design
The policy should classify note styles into behavioral families rather than
assuming one repeated-note model:

1. Lexical relative-marker styles, where immediate repeats may render a
   localized marker such as `Ibid.`, `Ibid`, `Id.`, or equivalent forms.
2. Shortened-note-first styles, where immediate and later repeats use an
   independently intelligible short form instead of a lexical relative marker.
3. Mixed or fallback styles, where lexical markers remain allowed but shortened
   forms are preferred or required in some contexts.
4. Legal or localized traditions whose repeated-note marker, punctuation, or
   locator syntax differs materially from general humanities note styles.

The policy should define processor-managed invariants:

- repeated-note state detection based on immediate-preceding citation identity
- locator-sensitive distinction between identical repeats and changed-locator
  repeats
- multi-source previous-note invalidation for lexical relative markers
- fallback from an unavailable lexical-marker form to the style’s subsequent
  short form when the style family requires it

The policy should define style-declared behavior:

- whether a style exposes a lexical relative marker at all
- the lexical marker’s punctuation, capitalization, and locator joining rules
- the structure of the subsequent short form
- any explicit distinction between note-start and authored/integral forms if a
  style guide requires one

The policy should also define how audits evolve:

- the current audit may continue to track shipped behavior for regression
  detection
- a future normative audit layer may report divergence from style-guide intent
  separately from shipped-style regressions
- family-level expectations should not over-specify authored/integral prose
  behavior until those semantics are resolved from primary sources

## Implementation Notes
- Use paraphrased rules derived from local manual review rather than quoted
  excerpts.
- Revisit Chicago, MHRA, New Hart’s Rules, OSCOLA, Bluebook-style `id.`, and
  localized forms such as German `ebd.` in the follow-up implementation.
- If the normative model requires more than one audit layer, keep the current
  shipped-style regression checks stable while adding explicit normative
  reporting.

## Acceptance Criteria
- [ ] The spec defines note-style behavioral families for repeated and
  shortened citations.
- [ ] The spec separates style-declared formatting from processor-managed
  repeated-note state.
- [ ] The spec records how multi-source previous notes affect lexical relative
  markers.
- [ ] The spec defines whether normative audit expectations should be tracked
  separately from current shipped behavior.
- [ ] Follow-up implementation can use this spec to tighten style-family tests
  and manifest expectations without consulting copyrighted source text.

## Changelog
- v1.0 (2026-03-11): Initial draft.
