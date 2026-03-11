# Note-Start Repeated-Note Policy Specification

**Status:** Draft
**Version:** 0.1
**Date:** 2026-03-11
**Supersedes:** None
**Related:** `.beans/csl26-nts1--spec-note-start-repeated-note-policy.md`, `docs/specs/NOTE_SHORTENING_POLICY.md`, `docs/specs/NOTE_STYLE_DOCUMENT_NOTE_CONTEXT.md`

## Purpose
Define a narrow normative policy for repeated-note behavior at note start so
Citum can decide whether note-start position is a settled style-declared
dimension, rather than leaving it bundled into the broader unresolved repeated
note space.

## Scope
In scope:
- whether immediate repeats at note start differ from the same repeated cite in
  other note positions
- how that distinction should be modeled if it exists
- which shipped note families should be settled first
- what fixtures and audit expectations are needed once the model is settled

Out of scope:
- prose/integral repeated-cite semantics
- changing current shipped output in this draft
- broad restatement of the full repeated-note policy

## Design
This follow-up should begin with the families most likely to expose a real
note-start distinction:

1. Chicago full-note and shortened-note families
2. OSCOLA and OSCOLA-no-ibid

Only after those are settled should the model expand to:

3. MHRA full-note and shortened-note families
4. New Hart's family
5. Thomson Reuters legal short-note family

The implementation contract should answer:

1. Is note-start behavior a style-declared rendering distinction, a processor
   state distinction, or both?
2. If style-declared, is it represented as a dedicated override, a mode of an
   existing repeated-note override, or another explicit schema shape?
3. If no stable family distinction exists, should note-start remain explicitly
   unresolved instead of being modeled?
4. What is the minimum fixture set required to test note-start repeated-cite
   behavior without over-specifying prose/integral behavior?

## Implementation Notes
- Prefer paraphrased style-manual summaries only.
- Reuse the two-layer audit approach from `NOTE_SHORTENING_POLICY.md`.
- Avoid widening this work into authored-note prose handling.

## Acceptance Criteria
- [ ] The spec decides whether note-start repeated-cite behavior is a real
  normative dimension for at least Chicago and OSCOLA families.
- [ ] The spec separates note-start behavior from prose/integral repeated-cite
  behavior.
- [ ] The spec identifies the intended audit/fixture changes required after the
  model is settled.

## Changelog
- v0.1 (2026-03-11): Initial draft for the `note-start` follow-up.
