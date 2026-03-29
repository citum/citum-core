---
# csl26-v8n2
title: EDTF historical era rendering follow-ups
status: in-progress
type: feature
priority: high
created_at: 2026-03-29T14:10:00Z
updated_at: 2026-03-29T14:10:00Z
---

Track the remaining EDTF historical/date rendering work around the new negative-year era rendering slice. Spec: docs/specs/EDTF_HISTORICAL_ERA_RENDERING.md

This bean covers:
- the implemented locale-backed rendering of valid negative EDTF years
- follow-up design questions that remain intentionally out of scope for the current PR
- documentation/roadmap cleanup when EDTF behavior claims drift ahead of implementation

## Follow-up inventory

- Decide whether positive-era output should remain suppressed or become locale-backed (`AD`/`CE`) in a later slice.
- Decide how to render negative years with unspecified digits (`-009u`, `-00uu`) without leaking raw astronomical notation.
- Decide whether literal historical strings such as `500 BCE` should remain literal-only inputs or gain a normalized parsing path elsewhere.
- Audit remaining docs and examples for any other shorthand or incomplete EDTF historical-date claims.

## Current slice summary

- Added locale-backed `before-era` support with English default `BC`.
- Added astronomical-to-historical year conversion for parsed EDTF negative years.
- Corrected the archival example data to valid EDTF `-0099` for 100 BC.
- Switched the published examples page to a checked-in demo style as the verification source for the historical archival output.
