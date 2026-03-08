---
# csl26-kafu
title: Re-run and retune affected numeric alphabetical styles after semantic sort fallback
status: completed
type: task
priority: high
created_at: 2026-03-08T00:33:31Z
updated_at: 2026-03-08T16:15:00Z
---

The missing-name sort fallback change is global enough to improve one class of
alphabetical styles while subtly perturbing others. After the engine-side fix,
the affected numeric alphabetical styles needed a fresh oracle pass to confirm
we did not trade one ordering bug for another family-specific regression.

Re-ran targeted oracle verification for:

- `springer-basic-brackets-no-et-al-alphabetical`
- `american-medical-association-alphabetical`
- `annual-reviews-alphabetical`
- `american-mathematical-society-label`

## Summary of Changes

Verified that the alphabetical bibliography cohort shows `orderingIssues: 0`
across the targeted oracle runs after the missing-name title fallback change.
No additional style or engine retuning was warranted for anonymous or
missing-contributor bibliography ordering.

Remaining oracle mismatches were unrelated to the missing-name sort fallback:

- `springer-basic-brackets-no-et-al-alphabetical`: citation-number offsets
- `american-medical-association-alphabetical`: one patent bibliography year gap
- `annual-reviews-alphabetical`: one extra container-title output
- `american-mathematical-society-label`: label-generation/citation mismatches

Closed as completed with no code changes required from this follow-up.
