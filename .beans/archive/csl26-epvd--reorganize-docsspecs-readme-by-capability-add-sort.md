---
# csl26-epvd
title: Reorganize docs/specs/ README by capability + add SORTING.md
status: completed
type: task
priority: normal
created_at: 2026-05-31T20:08:57Z
updated_at: 2026-05-31T20:13:23Z
---

Rewrite docs/specs/README.md index with capability-grouped tables (columns: Spec | Status | Tests) so specs and tests are co-navigable by area (Citations, Bibliography, Sorting, Localization, Data Model, Text/Rendering, Document/Input, Migration, Style System, Platform). Add new SORTING.md umbrella spec documenting current sort behavior. Deliver via PR.

## Summary of Changes

- Rewrote  index: replaced status-grouped tables with 10 capability areas (Citations, Bibliography, Note Styles, Sorting, Localization, Data Model, Text/Rendering, Document/Input, Migration, Style System, Platform). Each row now includes Spec | Status | Tests columns for direct spec↔test traceability.
- Added new  umbrella spec documenting end-to-end sort behavior (bibliography vs citation separation, sort keys, presets, collation, tiebreaking). References narrower sub-specs.
- Fixed: added  and  which were present in the directory but missing from the old index.
