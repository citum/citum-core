---
# csl26-qqdt
title: 'schema-style: corpus-driven preset discovery for config concerns'
status: todo
type: task
priority: normal
tags:
    - registry
    - presets
    - citum-analyze
created_at: 2026-06-16T15:49:15Z
updated_at: 2026-06-17T11:21:23Z
---

The `--config-presets` mode (csl26-t56t) discovers per-concern config
shapes (contributors, dates, titles, locators) across the CSL corpus that do
not match any existing named preset in citum-schema-style.

This bean is an audit task, not a count-only implementation task. The analyzer
is reliable as an exact serialized-shape frequency report, but it is not an
automatic missing-preset detector. Review the report and classify recurring
candidates as `accept`, `defer`, or `reject`.

Current evidence from the tree:

- `2844` styles analyzed, `0` parse errors.
- `dates`: `770` matched, `0` unmatched. Do not add date presets; current
  date presets cover all extracted non-default date configs.
- `locators`: `120` unmatched, one recurring shape: the current author-date
  locator config plus `strip-label-periods: true`. Treat this as the only
  obvious implementation candidate, pending final preset naming.
- `titles`: `643` matched, `1697` unmatched across `29` recurring shapes.
  Treat these as taxonomy/design evidence first, especially `default.emph`,
  `default.text-case: title`, and mixed default/category overrides.
- `contributors`: `1931` matched, `909` unmatched across `64` recurring
  shapes. Treat these as style-family or convention evidence; require a
  recognizable family/convention cluster before adding new `ContributorPreset`
  variants.

Priority order: rank by `corpus_count`, style-family/convention coherence,
naming clarity, and expected authored YAML reduction. Do not promote a shape
only because it is frequent.

Suggested next step: build a short audit table for the top contributor and title
candidates with count, examples, likely family/convention, and classification
(`accept`, `defer`, or `reject`). Do not implement contributor or title presets
from this report until that audit identifies a nameable family or convention.

Run:

```bash
cargo run --bin citum-analyze -- styles-legacy --config-presets --json \
  | jq '.concerns[] | {concern, matched_style_count, unmatched_style_count, candidate_count: (.candidates | length), candidates: .candidates[:5]}'
```

Public API impact for this bean revision: none. If implementation is approved
later, expected API surface is limited to a possible new `LocatorPreset` variant
for the `strip-label-periods: true` shape. Add no new `DatePreset`, `TitlePreset`,
or `ContributorPreset` without a separate taxonomy decision.

For a later implementation bean, add schema parse/resolve tests for any new
preset, add analyzer reverse-match coverage so the accepted candidate no longer
appears unmatched, and run `just pre-commit` for Rust/schema changes.
