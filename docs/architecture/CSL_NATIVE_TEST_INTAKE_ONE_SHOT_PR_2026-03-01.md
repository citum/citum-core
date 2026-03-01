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

## Planned follow-through after this PR

The next implementation PR should convert Wave 1 into traceable native tests.
That work should:

1. keep the upstream fixture ID in each Rust test
2. use minimal Citum-native fixtures rather than automatic XML-to-YAML
   translation
3. place cases in the existing test targets when possible:
   - `citations.rs`
   - `bibliography.rs`
   - `metadata.rs`
   - a new `rendering.rs` target for punctuation and flip-flop semantics if
     needed
4. run Typst alongside plain output for renderer-sensitive cases only

## Acceptance Criteria

This one-shot PR is successful if it leaves the project with:

- a repeatable report over the Hayagriva issue
- a machine-readable, Citum-owned Wave 1 intake list
- a documented policy that de-centers Hayagriva while still using it
- a clean handoff for the next PR that actually imports native regressions

## Current PR Status

This PR now includes real native regressions, not just intake scaffolding.

Native regressions landed so far:

- `disambiguate_YearSuffixAtTwoLevels`
- `date_SortEmptyDatesBibliography`
- `date_SortEmptyDatesCitation`
- `bugreports_ContainerTitleShort`
- `variables_ContainerTitleShort`
- `flipflop_LeadingSingleQuote`

Adjacent renderer coverage also landed for quote and punctuation movement:

- `punctuation_FullMontyQuotesIn` (partial, low-level renderer regression)
- `bugreports_MovePunctuationInsideQuotesForLocator` (partial, low-level renderer regression)

## Commands

Generate the external-signal report:

```bash
node scripts/report-csl-intake.js
```

Emit JSON for scripting:

```bash
node scripts/report-csl-intake.js --json
```
