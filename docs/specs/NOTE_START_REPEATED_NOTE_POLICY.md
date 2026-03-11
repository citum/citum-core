# Note-Start Repeated-Note Policy Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-03-11
**Supersedes:** None
**Related:** `.beans/archive/csl26-nts1--spec-note-start-repeated-note-policy.md`, `docs/specs/NOTE_SHORTENING_POLICY.md`, `docs/specs/NOTE_STYLE_DOCUMENT_NOTE_CONTEXT.md`

## Purpose
Define a narrow normative policy for repeated-note behavior at note start so
Citum can separate processor-owned repeated-note state from style-owned
note-initial rendering policy.

## Scope
In scope:
- whether note-initial immediate repeats differ from the same repeated cite in
  other note positions
- how a settled note-start distinction should be modeled
- Chicago and OSCOLA families only
- the minimum fixture and audit follow-up required after settlement

Out of scope:
- prose/integral repeated-cite semantics
- changing shipped style output in this spec wave
- widening this policy to MHRA, New Hart's, Thomson Reuters, or OSCOLA-no-ibid
- broad restatement of the full repeated-note policy

## Design
### Decision
`note-start` is a style-declared repeated-note rendering dimension, not a new
processor-managed repeated-note state.

Repeated-note state remains:

- `First`
- `Subsequent`
- `Ibid`
- `IbidWithLocator`

`note-start` is not a fifth repeated-note position. If code follows later, it
must be modeled as an orthogonal render context or override layered on top of
existing repeated-note state, not as a new `Position` variant.

The processor may later expose note-start context to rendering, but it must not
impose capitalization, lexical-marker spelling, or note-initial wording as a
universal rule. Those remain style-owned.

### Settled Families
This wave settles only the families with clear repo-backed and manual-backed
evidence of a note-start distinction:

1. Chicago full-note and shortened-note families
2. OSCOLA family

The following remain unresolved in this wave and must not be widened here:

1. OSCOLA-no-ibid
2. MHRA full-note and shortened-note families
3. New Hart's family
4. Thomson Reuters legal short-note family
5. prose/integral repeated-cite wording

### Family Rules
Chicago note families treat a note-initial immediate-repeat marker as
sentence-initial. The note-start lexical marker is therefore capitalized.

OSCOLA permits `ibid` for an immediately preceding footnote, but its note-start
marker remains lowercase rather than switching to initial capital.

These opposing family rules mean note-start behavior cannot be hard-coded as a
processor rule. The processor can detect repeated-note state, but the rendered
note-start marker form must remain style-declared.

### Audit and Fixture Contract
The existing repeated-note regression/conformance split remains unchanged:

1. Regression remains the hard gate.
2. Conformance remains report-only in this wave.

Minimum follow-up requirements after this settlement:

1. No new fixture is required if the conformance layer treats the current
   standalone repeated-note fixture as note-start output for note styles.
2. A future conformance field may record note-start lexical-marker policy for
   families that use `ibid`.
3. `note-start` should be removed from `unresolved` only for:
   - `chicago-full-note`
   - `chicago-shortened-note`
   - `oscola`
4. `prose-integral` remains unresolved everywhere.
5. MHRA, New Hart's, Thomson Reuters, and `oscola-no-ibid` remain untouched in
   this wave.

Minimum future test updates:

1. One Chicago conformance case that expects note-start initial capital on the
   immediate-repeat marker.
2. One OSCOLA conformance case that expects lowercase note-start `ibid`.
3. Report tests showing settled families no longer list `note-start` as
   unresolved.

## Implementation Notes
- Prefer paraphrased style-manual summaries only.
- Reuse the two-layer audit approach from `NOTE_SHORTENING_POLICY.md`.
- Avoid widening this work into authored-note prose handling.
- If a schema/API follow-up is needed, model note-start as an orthogonal render
  context layered over `citation.ibid` / `citation.subsequent`, not as a new
  repeated-note position.

## Sources
- Chicago Manual of Style FAQ on sentence-initial abbreviations.
- Chicago Manual of Style FAQ on immediate-note `ibid`.
- OSCOLA 4th edition guidance on `ibid`.

## Acceptance Criteria
- [x] The spec decides that note-start is style-declared rather than
  processor-managed state.
- [x] The spec keeps repeated-note state limited to `First`, `Subsequent`,
  `Ibid`, and `IbidWithLocator`.
- [x] The spec settles note-start behavior for Chicago and OSCOLA families
  only.
- [x] The spec keeps prose/integral repeated-cite behavior out of scope.
- [x] The spec identifies the minimum future audit and test changes without
  changing the regression/conformance split.

## Changelog
- v1.0 (2026-03-11): Activated with a style-declared note-start model and a
  narrow Chicago/OSCOLA settlement.
- v0.1 (2026-03-11): Initial draft for the `note-start` follow-up.
