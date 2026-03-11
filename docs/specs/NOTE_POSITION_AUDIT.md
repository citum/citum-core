# Note Position Audit Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-03-11
**Supersedes:** None
**Related:** `csl26-vnci`, `docs/specs/REPEATED_NOTE_CITATION_STATE_MODEL.md`, `docs/specs/MIXED_CONDITION_NOTE_POSITION_TREES.md`

## Purpose
Define a committed audit layer for note-style repeated-citation behavior so Citum can verify, report, and gap-track `first`, `subsequent`, and `ibid` rendering across shipped note styles instead of relying on ad hoc spot checks.

## Scope
In scope: committed repeated-note fixtures, a note-style expectation manifest, a scriptable audit over all shipped `processing: note` styles, report-core integration for core-style visibility, and automated tests for the audit plumbing. Out of scope: changing style-guide intent, broad style fidelity cleanup unrelated to repeated-note behavior, and rebuilding the published `docs/compat.html` artifact in this change.

## Design
The audit introduces a dedicated repeated-note citation fixture that models one source cited first, cited again immediately without a locator, cited again immediately with a locator, interrupted by another source, and then cited subsequently with a locator. The paired reference fixture must use stable references that expose note shortening clearly and are valid across Chicago-, MHRA-, legal-, and New Hart’s-style families.

A new manifest under `scripts/report-data/` must declare repeated-note expectations per shipped note style. At minimum each entry records whether the style is expected to support repeated-note overrides, whether immediate repeats should render lexical `ibid`, and whether non-immediate repeats should use a distinct subsequent form. Styles without expected repeated-note specialization remain auditable, but must be classified explicitly rather than inferred from missing YAML blocks.

The audit script must enumerate all shipped note styles in `styles/`, render the repeated-note fixture through the Citum processor, and evaluate the outputs against the manifest. It must emit structured JSON and a human-readable summary that distinguishes:
- pass
- configuration gap (expected override missing from style YAML)
- rendering gap (style declares the behavior but the rendered output does not match the expectation)

Report integration must be additive. `scripts/report-core.js` should retain current fidelity scoring, but core note styles must surface repeated-note audit status in the generated JSON and HTML detail view so compatibility reporting can identify note-position coverage gaps separately from generic citation fidelity. This integration is informational; it must not silently alter the existing fidelity numerator/denominator.

The expectation model should stay minimal and decision-complete for current note families:
- Chicago note families: lexical `ibid` plus distinct subsequent short form.
- MHRA and New Hart’s note families: distinct subsequent short form, no lexical `ibid`.
- OSCOLA: lexical `ibid` plus distinct subsequent short form.
- OSCOLA no-ibid and Thomson Reuters legal: no lexical `ibid`, but immediate repeats must reuse the subsequent short form.
- Styles intentionally lacking repeated-note specialization must be declared explicitly so they appear as known gaps rather than hidden omissions.

## Implementation Notes
Prefer implementing the audit in Node alongside `report-core.js` and `style-verification.js` so fixture policy, YAML parsing, and reporting metadata stay in one stack. The report integration should reuse existing style metadata plumbing instead of duplicating style discovery logic.

The initial coverage target is the 19 shipped note styles under `styles/`. If the audit finds a style that cannot be classified with the current expectation fields, split that into a narrow follow-on rather than expanding the manifest semantics mid-change.

## Acceptance Criteria
- [x] A committed repeated-note fixture pair exists under `tests/fixtures/` and is suitable for note-style audit runs.
- [x] A committed expectation manifest covers every shipped `processing: note` style in `styles/`.
- [x] A repeat-note audit command reports pass/gap status for every shipped note style and exits non-zero when an expected repeated-note behavior is missing or misrendered.
- [x] `scripts/report-core.js` includes repeated-note audit status for applicable core note styles without changing the existing fidelity scoring contract.
- [x] Automated tests cover manifest loading and at least one passing and one failing repeated-note audit scenario.

## Changelog
- v1.0 (2026-03-11): Activated with committed fixtures, audit command, and report integration.
