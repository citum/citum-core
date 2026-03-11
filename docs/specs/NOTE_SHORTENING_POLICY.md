# Note Shortening Policy Specification

**Status:** Active
**Version:** 1.1
**Date:** 2026-03-11
**Supersedes:** None
**Related:** `.beans/archive/csl26-t79d--spec-normative-note-shortening-policy.md`, `docs/specs/REPEATED_NOTE_CITATION_STATE_MODEL.md`, `docs/specs/NOTE_POSITION_AUDIT.md`, `docs/specs/NOTE_STYLE_DOCUMENT_NOTE_CONTEXT.md`

## Purpose
Define the normative contract for repeated-note and shortened-note behavior in
note styles so Citum can distinguish processor invariants from style-declared
formatting and can report shipped behavior separately from style-guide
conformance.

## Scope
In scope:
- note-style family modeling for repeated-note and shortened-note behavior
- the split between processor-managed note-position invariants and style-declared
  rendering behavior
- audit/report semantics for shipped regressions versus normative conformance
- settled versus unresolved areas for repeated-note testing

Out of scope:
- changing note-style output solely to match this spec
- committing copyrighted quotations or local research excerpts
- settling note-start, prose, or authored/integral repeated-cite distinctions
  without strong repo-backed evidence

## Design
### Processor Invariants
The processor owns note-position state and must remain style-agnostic in the
following areas:

1. Immediate-repeat detection is based on the immediately preceding citation or
   note context, not on style family.
2. For single-source repeats, locator comparison distinguishes:
   - same item and same locator context -> `Ibid`
   - same item and changed locator context -> `IbidWithLocator`
3. A previous note containing multiple sources invalidates lexical relative
   markers for the next cite.
4. When `citation.ibid` is absent, `Ibid` and `IbidWithLocator` fall back to
   `citation.subsequent` before falling back to the base citation spec.

These rules are processor invariants and must be enforced in Rust tests, not in
style-family YAML.

### Style-Declared Behavior
Styles declare rendering policy, not note-position state. The following remain
style-owned:

- whether a lexical relative marker exists
- the marker family, punctuation, capitalization, and locator joiner
- whether shortened-note behavior is expressed in `citation.subsequent` or in
  the base citation template
- the structure of the shortened-note form
- any explicit note-start versus prose/integral distinction, but only when that
  distinction is actually settled

### Normative Family Model
Shipped note styles fall into the following conformance families:

1. Chicago full-note family:
   immediate repeats use lexical `ibid`; later repeats use a distinct
   shortened-note form.
2. Chicago shortened-note family:
   immediate repeats use lexical `ibid`; the base citation is already a
   shortened-note form, so no distinct `subsequent` override is required.
3. MHRA full-note family:
   no lexical relative marker; immediate repeats reuse the shortened-note form
   used for later repeats.
4. MHRA shortened-note family:
   immediate repeats use lexical `ibid`; the base citation is already a
   shortened-note form.
5. New Hart’s full-note family:
   no lexical relative marker; immediate repeats reuse the shortened-note form
   used for later repeats.
6. OSCOLA family:
   immediate repeats use lexical `ibid`; later repeats use a distinct
   shortened-note form.
7. OSCOLA-no-ibid family:
   no lexical relative marker; immediate repeats reuse the subsequent short-note
   form.
8. Thomson Reuters legal short-note family:
   no lexical relative marker; immediate repeats reuse the subsequent short-note
   form, while locator punctuation and joiners remain style-declared.

### Audit and Reporting Contract
Repeated-note evaluation is split into two layers:

1. Regression layer:
   captures current shipped behavior and remains the hard failure gate for the
   audit command.
2. Conformance layer:
   captures normative family expectations and remains report-only in this wave.

The two layers must be reported separately in audit JSON and `report-core`
output. Conformance mismatches must not fail the audit command or the
core-quality gate in this change.

### Settled Versus Unresolved Areas
The following are settled enough for normative checks:

- whether a lexical relative marker exists
- whether immediate repeats use that marker or reuse a shortened-note form
- whether locator-sensitive immediate repeats preserve locator content
- whether shortened-note behavior is distinct from the first full note or is
  already encoded in the base citation form

The following remain unresolved and must not be turned into exact-output
assertions in this wave:

- note-start versus prose repeated-cite distinctions
- authored/integral repeated-cite wording beyond narrow invariants
- family-specific prose handling not already supported by settled repo behavior

## Implementation Notes
- Manual-derived rules must be paraphrased rather than quoted.
- Existing green repeated-note behavior is the stability baseline.
- If a normative dimension is not settled, represent it as unresolved in audit
  data rather than forcing a speculative pass/fail rule.

## Acceptance Criteria
- [x] The spec separates processor invariants from style-declared behavior.
- [x] The spec classifies shipped note styles into normative repeated-note
  families.
- [x] The spec defines the audit split between shipped regressions and
  normative conformance.
- [x] The spec records that multi-source previous notes invalidate lexical
  relative markers.
- [x] The spec preserves unresolved prose/integral repeated-cite behavior as
  unresolved rather than over-specifying it.

## Changelog
- v1.1 (2026-03-11): Activated with the layered audit model, settled family
  taxonomy, and explicit report-only normative conformance contract.
- v1.0 (2026-03-11): Initial draft.
