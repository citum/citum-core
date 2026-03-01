---
# csl26-cn53
title: Model deferred native citation semantics from CSL Wave 1
status: todo
type: feature
priority: deferred
tags:
    - csl
    - testing
    - architecture
created_at: 2026-03-01T16:45:18Z
updated_at: 2026-03-01T16:45:18Z
---

This bean tracks the Wave 1 cases that were deferred because they appear to represent reusable native citation semantics that Citum may eventually want to model cleanly, rather than CSL-1.0-only mechanics.

Why this exists:
- PR #258 completed the high-fit Wave 1 intake and intentionally reclassified several remaining cases.
- Some of those cases should not be forced into ad hoc fixes, but they may still justify first-class native modeling later.
- We want an explicit place to revisit them without re-auditing the whole Hayagriva or CSL corpus.

Candidate deferred-core-feature cases:
- disambiguate_ByCiteMinimalGivennameExpandMinimalNames
- disambiguate_PrimaryNameWithInitialsLimitedToPrimary
- possibly disambiguate_BasedOnEtAlSubsequent, if Citum decides et-al-subsequent controls belong in the native contributor model

Non-goals for this bean:
- Do not port CSL display-block or second-field-align behavior literally.
- Do not add a CSL-compat layer for features that are not a good fit for Citum's architecture.
- Do not mix this work with broad renderer-markup normalization.

Expected output when this bean is picked up:
- decide which deferred Wave 1 cases are true native feature candidates vs CSL-specific holdovers
- propose the minimal schema/engine additions, if any
- add traceable native regressions for any promoted cases
- update tests/fixtures/csl-native-intake-wave1.json and docs/architecture/CSL_NATIVE_TEST_INTAKE_ONE_SHOT_PR_2026-03-01.md accordingly
