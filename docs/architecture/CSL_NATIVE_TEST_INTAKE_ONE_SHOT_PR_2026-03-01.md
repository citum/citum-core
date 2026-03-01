# CSL Native Test Intake One-Shot PR

## Summary

This PR establishes a durable intake path for CSL 1.0 processor tests into
Citum's native regression suite.

The immediate trigger was the recent Typst integration plus Hayagriva's
[Tracking Issue: CSL Spec Compliance](https://github.com/typst/hayagriva/issues/327).
That issue is useful, but it should not become the project's definition of
scope.

Instead, this PR treats Hayagriva as one external signal alongside:

- current native test gaps in `crates/citum-engine/tests/`
- current oracle fidelity risk and top-style impact
- renderer sensitivity introduced by Typst output
- architectural fit with Citum's model

## Ground Truth

As of March 1, 2026:

- Hayagriva issue `#327` contains **238** checklist fixtures
- the local vendored `tests/csl-test-suite` snapshot contains **all 238**
  corresponding human and machine fixtures
- current native coverage is still selective and uneven
- `tests/fixtures/convert_csl_tests.sh` remains a scaffold rather than a
  general conversion workflow
- `crates/citum-engine/tests/citations.rs` still contains at least one
  placeholder-style upstream mapping, which confirms that intake work is still
  incomplete

This means the main bottleneck is no longer corpus availability. It is
selection, traceability, and native harness design.

## Decision

We should do a comprehensive review of the CSL suite, but not by mirroring the
Hayagriva issue.

The correct approach is:

1. Use the full local CSL test suite as the candidate corpus.
2. Use Hayagriva issue `#327` as a secondary prioritization input.
3. Curate a Wave 1 native intake set that reflects Citum's own engine and style
   priorities.
4. Preserve upstream fixture IDs in native tests for traceability.

## What This PR Adds

### 1. Intake reporting script

`scripts/report-csl-intake.js` parses Hayagriva issue `#327`, checks fixture
presence under `tests/csl-test-suite/processor-tests`, and reports section and
state counts.

This gives the project a repeatable way to answer:

- how large the external issue currently is
- which fixture groups it emphasizes
- whether the local corpus contains those fixtures

### 2. Curated Wave 1 intake manifest

`tests/fixtures/csl-native-intake-wave1.json` is a machine-readable list of the
first intake candidates.

The manifest is intentionally curated rather than exhaustive. It includes:

- direct Hayagriva-linked cases that are clearly relevant to Citum
- local native-gap cases that matter even when they are not the same fixture ID
- explicit defer/exclude decisions for unsupported or poorly fitting areas

### 3. Architecture record

This document records the policy decision so future intake work does not drift
into "track whatever Hayagriva tracks" behavior.

## Intake Policy

### High-priority signals

- **Citum relevance**: behavior maps cleanly to current processor capabilities
- **Oracle or style risk**: likely to affect top styles or known output
  fidelity
- **Current native gap**: under-tested in `citum-engine` integration tests

### Medium-priority signal

- **Renderer sensitivity**: punctuation, quoting, flip-flop, or decoration
  cases that can diverge in Typst versus plain output

### Secondary signal

- **Hayagriva issue inclusion**: useful because it reflects another processor's
  pain points, but not sufficient by itself

## Wave 1 shape

Wave 1 focuses on cases that provide the most value per test:

- year-suffix and given-name disambiguation edge cases
- container-title short and related variable behavior
- empty-date sorting
- subsequent-author-substitute semantics
- non-dropping particle and name-particle behavior
- punctuation, quotes, and flip-flop behavior that can regress in Typst output
- author-as-heading behavior that aligns with existing grouping/document logic

Wave 1 explicitly does **not** let second-field-align semantics dominate the
plan. Those cases are tracked in the manifest as `adapt-later`.

## Wave 1 completion

This PR now completes the original Wave 1 review set in the only way that
actually scales:

- land native regressions for the cases that fit Citum's current engine and
  data model
- adapt a few CSL cases into native equivalents when Citum's explicit model is
  better than the CSL 1.0 fixture shape
- reclassify the cases that depend on unsupported CSL-specific behavior instead
  of forcing brittle one-off implementations into this PR

That means Wave 1 is no longer "a handoff for the next PR." The implementation
and the curation happened together here.

## Acceptance Criteria

This one-shot PR is successful because it now leaves the project with:

- a repeatable report over the Hayagriva issue
- a machine-readable, Citum-owned Wave 1 intake list
- a documented policy that de-centers Hayagriva while still using it
- real native regressions and engine fixes for the high-fit Wave 1 cases
- explicit reclassification for the cases that do not fit the current Citum
  architecture

## Current PR Status

This PR now includes real native regressions, not just intake scaffolding.

Native regressions landed so far:

- `disambiguate_YearSuffixAndSort`
- `disambiguate_YearSuffixAtTwoLevels`
- `date_SortEmptyDatesBibliography`
- `date_SortEmptyDatesCitation`
- `bugreports_ContainerTitleShort`
- `variables_ContainerTitleShort`
- `magic_SubsequentAuthorSubstitute`
- `name_HyphenatedNonDroppingParticle1`
- `name_HyphenatedNonDroppingParticle2`
- `flipflop_LeadingSingleQuote`
- `flipflop_StartingApostrophe`
- `punctuation_FullMontyQuotesOut`

Adjacent renderer coverage also landed for quote and punctuation movement:

- `punctuation_FullMontyQuotesIn` (partial, low-level renderer regression)
- `bugreports_MovePunctuationInsideQuotesForLocator` (partial, low-level renderer regression)
- `flipflop_Apostrophes` (adapted title-level apostrophe regression)

Wave 1 entries deliberately reclassified out of `integrate-now` fall into two
different buckets.

Deferred core-feature candidates:

- `disambiguate_ByCiteMinimalGivennameExpandMinimalNames`
- `disambiguate_PrimaryNameWithInitialsLimitedToPrimary`
- `disambiguate_BasedOnEtAlSubsequent`

These are not currently modeled cleanly, but they may justify first-class
native support later if we decide the semantics belong in Citum. Follow-up is
tracked in [.beans/csl26-cn53--model-deferred-native-citation-semantics-from-csl.md](/Users/brucedarcus/Code/citum/citum-core/.beans/csl26-cn53--model-deferred-native-citation-semantics-from-csl.md).

Deferred CSL-specific or out-of-model cases:

- `textcase_SkipNameParticlesInTitleCase`
- `display_AuthorAsHeading`

These are deferred because they currently look more like CSL-specific mechanics
or layout conventions than reusable native Citum semantics.

Taken together, the reclassified cases depend on one of:

- per-name minimal given-name expansion not modeled by current disambiguation hints
- CSL-specific disambiguation rules such as `primary-name-with-initials`
- `et-al-subsequent-*` controls not present in Citum contributor options
- title-case transformation features not present in the current rendering model
- CSL display-block layout semantics that do not map directly to current Citum document rendering

## Commands

Generate the external-signal report:

```bash
bun scripts/report-csl-intake.js
```

Emit JSON for scripting:

```bash
bun scripts/report-csl-intake.js --json
```
