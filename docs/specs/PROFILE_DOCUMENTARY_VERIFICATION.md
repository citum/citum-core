# Profile Documentary Verification Specification

**Status:** Draft
**Date:** 2026-04-23
**Related:** `UNIFIED_SCOPED_OPTIONS.md`, `STYLE_TAXONOMY.md`, `../architecture/MULTI_AUTHORITY_STYLE_VERIFICATION_PLAN_2026-03-07.md`, bean `csl26-u1tq`

## Purpose

Define the verification model for true `profile + config-wrapper` styles when
publisher or journal documentation is a better primary authority than the
corresponding CSL renderer.

This specification introduces documentary-primary verification as an opt-in
policy for marked profile wrappers. The first implementation wave pilots the
model on `elsevier-harvard` and `elsevier-vancouver`.

## Scope

In scope:

- primary-authority verification for styles already classified as
  `profile + config-wrapper`
- policy metadata required to mark a profile as documentary-primary
- comparator behavior for documentary-primary reports and checks
- curated rendered fixture requirements for the Elsevier pilot
- release-gate semantics when documentary evidence and citeproc-js disagree

Out of scope:

- changing the global default authority away from `citeproc-js`
- applying documentary-primary verification to journals, structural wrappers, or
  standalone styles in this wave
- reclassifying `elsevier-with-titles` or non-Elsevier profile families
- changing style YAML schema or widening what profiles are allowed to override

## Design

### 1. Eligibility

This model applies only to styles that are already valid `profile +
config-wrapper` wrappers.

A style may be documentary-primary only when all of the following are true:

- it is semantically classified as a `profile`
- it is operationally a `config-wrapper`
- it keeps the config-wrapper contract:
  no local templates, no local `type-variants`, and no template-clearing `null`
- current publisher, society, standards, or journal guidance provides a better
  authority basis for the wrapper delta than the corresponding CSL renderer

If a style stops satisfying the config-wrapper contract, it must be reclassified
before it can use documentary-primary verification.

### 2. Authority Model

Documentary-primary verification is opt-in per style. It is not the new global
default.

For a marked documentary-primary profile:

- curated documentary fixtures are the primary release gate
- `citeproc-js` remains a secondary comparison source
- a citeproc mismatch is not a release failure by itself
- citeproc drift still appears in reports for adjudication and conflict review

This means fidelity is defined as fidelity to the declared primary authority,
not universal parity with the nearest CSL ancestor.

### 3. Verification Policy Metadata

The follow-up implementation must support the following policy shape in
`scripts/report-data/verification-policy.yaml` for marked pilot styles:

- `authority: documentary`
- `authority_id: <fixture-set-id>`
- `secondary: [citeproc-js]`
- `scopes: [citation, bibliography]`
- optional `note` explaining why documentary authority overrides CSL for that
  profile

The `authority_id` identifies the curated documentary fixture set used by the
documentary comparator.

### 4. Comparator Behavior

The reporting and oracle layer must add a documentary comparator path.

That path must:

- load curated expected citation and bibliography outputs from documentary
  snapshot or fixture files
- compare Citum output against those documentary expectations for the declared
  scopes
- report documentary-primary pass/fail separately from citeproc secondary drift
- keep citeproc mismatches visible in reports
- preserve adjusted-divergence handling for citeproc secondary comparisons
  without letting those adjustments redefine documentary pass/fail

Documentary-primary pass/fail and citeproc secondary-drift reporting must be
distinct outputs, not a single conflated fidelity result.

### 5. Fixture Model

Use curated rendered fixtures as the primary authority form.

For the Elsevier pilot, create one documentary fixture set for each profile:

- `elsevier-harvard`
- `elsevier-vancouver`

Each set must contain expected citation and bibliography outputs for the
guide-backed deltas that justify the wrapper.

Minimum coverage for the pilot:

- `elsevier-harvard`: bibliography cases that prove
  `bibliography.options.date-position`
- `elsevier-vancouver`: citation and bibliography cases that prove label-wrap
  and bibliography label-mode behavior

The pilot fixture sets must be explicit enough that a passing result means the
documented wrapper effect is actually being verified, not merely inferred from
unrelated baseline fixtures.

### 6. Elsevier Pilot Scope

The first implementation wave is limited to:

- `elsevier-harvard`
- `elsevier-vancouver`

Do not widen this wave to:

- `elsevier-with-titles`
- Springer profiles
- Taylor & Francis profiles
- journal wrappers
- a portfolio-wide documentary-primary rollout

The pilot exists to prove the policy and comparator model before broader
adoption.

## Implementation Notes

- The policy layer already accepts `documentary` as an authority kind; the main
  missing piece is the actual documentary comparator path in reporting/oracle
  tooling.
- Existing citeproc-based reports remain valuable as secondary drift evidence
  and should stay visible to maintainers.
- The pilot should favor small curated fixture sets over a broad but weak
  documentary corpus.

## Acceptance Criteria

- [ ] Marked documentary-primary profiles pass/fail against curated documentary
      fixtures.
- [ ] Reports show documentary as the primary authority and citeproc-js as a
      secondary source for the pilot styles.
- [ ] Citeproc mismatches remain visible in reports but do not fail CI when
      documentary authority passes.
- [ ] Styles not explicitly marked documentary-primary keep their current
      authority behavior.
- [ ] A profile that violates the config-wrapper contract cannot be
      documentary-primary without first being reclassified.
- [ ] The Elsevier pilot includes separate documentary fixture sets for
      `elsevier-harvard` and `elsevier-vancouver`.
- [ ] Pilot fixtures cover the exact scoped-option effects that justify the
      wrappers.

## Changelog

- 2026-04-23: Initial draft.
