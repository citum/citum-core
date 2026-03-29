---
# csl26-v8n2
title: EDTF historical era rendering follow-ups
status: completed
type: feature
priority: high
created_at: 2026-03-29T14:10:00Z
updated_at: 2026-03-29T15:30:48Z
---

Track the remaining EDTF historical/date rendering work around the new negative-year era rendering slice. Current shipped spec: `docs/specs/EDTF_HISTORICAL_ERA_RENDERING.md`. Draft review spec: `docs/specs/EDTF_ERA_LABEL_PROFILES.md`.

This bean covers:
- the implemented locale-backed rendering of valid negative EDTF years
- follow-up design questions that remain intentionally out of scope for the current PR
- documentation/roadmap cleanup when EDTF behavior claims drift ahead of implementation
- review and approval of the follow-on draft spec before implementation begins

## Follow-up inventory

- Define a minimal `DateConfig` API for optional positive-era output and era-label profiles.
- Define how negative years with unspecified digits (`-009u`, `-00uu`) should render without leaking raw astronomical notation.
- Normalize positive unspecified years for display so end users do not see raw EDTF `u` markers.
- Decide whether literal historical strings such as `500 BCE` should remain literal-only inputs or gain a normalized parsing path elsewhere.
- Keep docs/examples claims aligned with what is shipped versus what is still draft-only.

## Current slice summary

- Added locale-backed `before-era` support with English default `BC`.
- Added astronomical-to-historical year conversion for parsed EDTF negative years.
- Corrected the archival example data to valid EDTF `-0099` for 100 BC.
- Switched the published examples page to a checked-in demo style as the verification source for the historical archival output.

## Draft review focus

- Is the proposed config surface minimal and style-appropriate?
- Are the defaults backwards-compatible with the shipped historical-era slice?
- Are unspecified BCE years rendered without exposing EDTF internals?
- Is the locale/style boundary clear enough to implement in a follow-on PR?

## Summary of Changes

- Added `EraLabels` (default/bc-ad/bce-ce) and `NegativeUnspecifiedYears` (range/fuzzy) enums to `DateConfig`
- Added `ad`, `bc`, `bce`, `ce` era suffix fields to `DateTerms` and `RawDateTerms`
- Engine `format_display_year` now renders era-label profiles, normalizes positive unspecified years (u→X), and computes historical ranges for negative unspecified years
- Spec `docs/specs/EDTF_ERA_LABEL_PROFILES.md` updated to Active
- JSON schemas regenerated (patch bump)
- Commit: ecaf3dc4
