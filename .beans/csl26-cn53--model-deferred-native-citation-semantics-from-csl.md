---
# csl26-cn53
title: Model deferred native citation semantics from CSL Wave 1
status: completed
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

## Decisions Made

### 1. disambiguate_BasedOnEtAlSubsequent - PROMOTED
- **Decision**: Add native `subsequent_min` and `subsequent_use_first` fields to `ShortenListOptions`.
- **Rationale**: This is a useful, generalizable feature. Authors often use shorter forms on repeat cites (standard practice in academic writing).
- **Implementation**:
  - Added `subsequent_min` and `subsequent_use_first` to schema
  - Modified contributor renderer to check citation position (via `ProcHints.position`)
  - Apply subsequent-specific settings when `position != First`
  - Test passes: `test_disambiguate_basedonetalsubsequent` verifies first vs subsequent et-al shortening

### 2. disambiguate_ByCiteMinimalGivennameExpandMinimalNames - DEFERRED FURTHER
- **Decision**: Not implemented in this round.
- **Reason**: This is fundamentally a "by-cite" disambiguation mechanism mixed with "minimal names" logic. It's CSL-specific machinery that doesn't fit Citum's declarative model cleanly. Deferred for future research.

### 3. disambiguate_PrimaryNameWithInitialsLimitedToPrimary - DEFERRED FURTHER
- **Decision**: Not implemented in this round.
- **Reason**: This is a per-position rule (`primary-name-with-initials` logic tied to a specific position in list). It's a CSL holdover that would require position-level name formatting overrides. Deferred for future design.

## Summary of Changes

- **Schema**: `ShortenListOptions` extended with `subsequent_min` and `subsequent_use_first` (both `Option<u8>`)
- **Engine**:
  - Added `position` field to `ProcHints` to track citation position (First/Subsequent/Ibid/etc.)
  - Modified render functions to pass `position` from `Citation` through to `ProcHints`
  - Updated contributor rendering logic to apply subsequent settings when rendering
- **Tests**: Existing test `test_disambiguate_basedonetalsubsequent` now passes (was skipped, now validates subsequent et-al shortening)
- **Migrate**: Updated CSL→Citum template compiler to include new fields with `None` defaults
