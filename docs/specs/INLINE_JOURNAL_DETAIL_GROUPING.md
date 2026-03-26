# Inline Journal Detail Grouping Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-03-25
**Supersedes:** (none)
**Related:** csl26-myrk, [TEMPLATE_V2.md](./TEMPLATE_V2.md)

## Purpose

Specify how Citum should represent inline article-journal detail blocks that
need mixed delimiter structure under Template V2. The concrete target is
patterns such as `volume, issue (Mon Year), pages`, where the outer detail
sequence remains comma-delimited but `issue` and `date` must stay in a nested
space-delimited group.

## Scope

In scope:
- Bibliography template generation for article-journal detail patterns that
  place `volume`, `issue`, and `issued` adjacently.
- Template V2 composition using existing `group` nodes only.
- Style output updates for touched styles that currently use the older
  flattened workaround.

Out of scope:
- New schema fields or a new template component kind.
- Reintroducing template-level overrides.
- Repo-wide `items` to `group` cleanup outside touched files.

## Design

Template V2 already provides the required composition surface:

```yaml
- group:
    - number: volume
    - group:
        - number: issue
        - date: issued
          form: year-month
          wrap: parentheses
      delimiter: " "
    - number: pages
      prefix: "pp. "
  delimiter: ", "
```

This shape is normative for migrated article-journal detail blocks when the
legacy CSL pattern is structurally:

- `volume`
- followed by `issue`
- followed by `issued` rendered as `year-month`
- all within a comma-delimited journal detail sequence

Migration should prefer this nested `group` shape over flattened sibling
components because the flattened form cannot preserve the inner spacing without
hardcoded punctuation on adjacent siblings.

`type-variants` remains the correct scope for article-journal-specific
structure. This spec does not add any new override surface, and all examples
use `group`, not legacy `items`.

## Implementation Notes

- The migrate pass may use a structural heuristic rather than source-provenance
  tracking as long as it only rewrites the `volume` + `issue` + adjacent
  `issued(year-month)` pattern.
- Touched style YAML should be normalized to `group:` even though serde still
  accepts `items:`.
- Runtime rendering already supports nested groups and empty-group suppression;
  the main implementation work is in migrate output shape and style cleanup.

## Acceptance Criteria

- [ ] Migration rewrites adjacent article-journal `volume` + `issue` +
      `issued(year-month)` detail blocks into nested Template V2 `group`
      composition.
- [ ] The rendered bibliography output preserves comma-delimited outer detail
      flow and space-delimited `issue` + parenthesized date flow.
- [ ] Touched docs and YAML use Template V2 terminology (`group`, no template
      override language).

## Changelog

- v1.0 (2026-03-25): Initial version.
