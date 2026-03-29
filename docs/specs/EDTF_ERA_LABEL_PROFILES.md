# EDTF Era Label Profiles Specification

**Status:** Draft
**Date:** 2026-03-29
**Related:** `.beans/csl26-v8n2--edtf-historical-era-rendering-follow-ups.md`, `docs/specs/EDTF_HISTORICAL_ERA_RENDERING.md`, `docs/architecture/PRIOR_ART.md`

## Purpose

Define the next display-layer follow-on to historical EDTF rendering: optional positive-era labeling, profile-driven era term selection, and end-user-safe rendering for EDTF years with unspecified digits. This specification extends the shipped negative-year behavior without changing EDTF parsing or broadening accepted input syntax.

This draft is purely descriptive. It does not change runtime behavior in this PR; it defines the intended next implementation slice for review.

## Scope

In scope: proposed `DateConfig` API for era-label profiles, default-positive suppression policy, display-only normalization of unspecified years, and a default rendering policy for negative EDTF years with unspecified digits.

Out of scope: changes to EDTF parsing, normalization of literal strings such as `500 BC` or `500 BCE`, locale-override expansion for date terms, broader message-system refactors, and a fully authored prose taxonomy for fuzzy historical labels.

## Design

### Existing baseline

The active historical-era slice remains the source of truth for shipped behavior:

- valid negative EDTF years render as historical years
- negative years use locale-backed `before-era`
- positive years remain unlabeled by default
- examples/docs use valid astronomical EDTF input

This follow-on spec must preserve that baseline when no new option is configured.

### Canonical input model

EDTF remains the canonical input format. This slice is display-only.

- Parsed EDTF years continue to drive rendering.
- Astronomical-to-historical conversion remains unchanged for fully specified negative years.
- Non-EDTF literal values such as `500 BC` remain out of scope for parsing and normalization.

### Proposed public API

Add two new `options.dates` fields:

```yaml
options:
  dates:
    era-labels: default | bc-ad | bce-ce
    negative-unspecified-years: range | fuzzy
```

`era-labels` controls which era suffix set, if any, is shown:

- `default`: preserve current behavior; negative years use locale `before-era`, positive years are unlabeled
- `bc-ad`: negative years use locale `bc`; positive years use locale `ad`
- `bce-ce`: negative years use locale `bce`; positive years use locale `ce`

`negative-unspecified-years` controls how negative EDTF years with unspecified digits are rendered:

- `range`: render explicit historical ranges; this is the default and the only normative mode in this spec
- `fuzzy`: reserved for later prose-oriented output such as century buckets; semantics remain intentionally TBD

### Era term sourcing

Era labels should come from semantic locale terms rather than hard-coded English strings:

- `ad`
- `bc`
- `bce`
- `ce`

`before-era` remains the fallback/default negative-era source for `era-labels: default`, preserving the shipped slice and allowing locales that currently expose only a generic negative-era term.

This belongs in `DateConfig`, not `LocaleOverride`, because locale overrides currently patch messages, grammar options, and legacy aliases, not date-term fields.

Locales provide strings such as `ad`, `bc`, `bce`, and `ce`; `DateConfig` decides when and whether those terms are used.

### Rendering policy

#### Fully specified years

- Positive years render with no suffix by default.
- `bc-ad` opt-in renders positive years with `AD`.
- `bce-ce` opt-in renders positive years with `CE`.
- Negative years continue to convert from astronomical numbering to historical numbering before any era suffix is applied.

Tiny policy table:

| EDTF input | `default` | `bc-ad` | `bce-ce` |
|------------|-----------|---------|----------|
| `0054` | `54` | `54 AD` | `54 CE` |
| `-0043` | `44 BC` | `44 BC` | `44 BCE` |

Examples:

- `-0099` -> `100 BC`
- `0054` -> `54` under `default`
- `0054` -> `54 AD` under `bc-ad`
- `0054` -> `54 CE` under `bce-ce`

#### Positive years with unspecified digits

Positive unspecified years should be normalized for display instead of exposing raw EDTF `u` markers.

Examples:

- `199u` -> `199X`
- `19uu` -> `19XX`

This is a display normalization only; stored EDTF remains unchanged.

#### Negative years with unspecified digits

Negative unspecified years must never expose raw EDTF or astronomical notation to end users.

Default policy: explicit historical ranges.

The range endpoints should be derived from the EDTF uncertainty window in astronomical numbering, then converted into historical-year output and presented in ascending user-facing order.

Examples:

- `-009u` -> `100–91 BC`
- `-00uu` -> `100–1 BC`

### Deferred fuzzy mode

`negative-unspecified-years: fuzzy` is part of the proposed config surface so reviewers can evaluate the long-term API shape, but this spec does not fully define the prose taxonomy yet.

If implemented later, fuzzy mode may produce outputs such as:

- `early 1st century BC`
- `1st century BC`

That future slice must specify:

- bucketing rules
- locale responsibilities
- how prose labels interact with style conventions

## Implementation Notes

- Reuse parsed EDTF `Year` values; do not add alternate parsing paths.
- Apply one shared display helper across single dates and interval endpoints.
- Preserve backwards compatibility for styles that rely on the current negative-era default.
- Treat `fuzzy` as reserved unless and until a later implementation spec defines it fully.

## Acceptance Criteria

- [ ] The spec defines a minimal `DateConfig` API for era label selection and negative unspecified-year rendering.
- [ ] The spec preserves current shipped behavior under the default configuration.
- [ ] The spec states that EDTF remains the canonical input model.
- [ ] The spec defines default positive-era suppression and opt-in `BC/AD` and `BCE/CE` profiles.
- [ ] The spec requires display normalization for positive unspecified years using `X` masks.
- [ ] The spec requires negative unspecified years to avoid raw EDTF and astronomical notation, with explicit ranges as the default.
- [ ] The spec states that `fuzzy` is reserved for future work and is not fully normative in this draft.
- [ ] The spec explains why this design belongs in `DateConfig` rather than locale overrides.

## Changelog

- 2026-03-29: Initial draft for review.
