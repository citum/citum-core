# EDTF Historical Era Rendering Specification

**Status:** Active
**Date:** 2026-03-29
**Related:** `.beans/csl26-v8n2--edtf-historical-era-rendering-follow-ups.md`, `docs/architecture/PRIOR_ART.md`

## Purpose

Define how Citum renders valid EDTF historical years so negative astronomical years do not leak into user-facing citations and bibliography output. This specification covers locale-backed era suffixes for negative years, valid EDTF historical-year semantics, and the checked-in documentation/demo path used to verify the rendered examples.

## Scope

In scope: valid EDTF negative-year rendering, locale-backed era suffix lookup, astronomical-to-historical year conversion, documentation/examples updates for valid historical EDTF, and explicit tracking of adjacent EDTF date rendering follow-up work.

Out of scope: non-standard shorthand parsing such as `-100`, automatic `AD`/`CE` rendering for positive years, broader locale-message integration for date formatting, and unresolved semantics for negative years with unspecified digits.

## Design

### Historical EDTF semantics

Citum keeps strict EDTF parsing. Historical years must therefore use valid astronomical year numbering:

- `-0099` = `100 BC`
- `-0100` = `101 BC`
- `0000` = `1 BC`
- `0001` = `1`

Documentation and examples must use valid EDTF input, not shorthand approximations.

### Locale-backed era suffix

`DateTerms` and `RawDateTerms` gain a neutral `before-era` field for year zero and negative years. English defaults set this to `BC`.

This slice intentionally adds only the negative-era suffix. Positive-era output remains unchanged and is tracked as follow-up work.

### Rendering behavior

When the processor renders a parsed EDTF year:

- if the year is greater than `0`, render the year number unchanged
- if the year is `0` or negative, convert from astronomical numbering to the historical year number using `1 - year`
- append the locale `before-era` suffix when available

This conversion applies consistently to:

- single dates
- date range starts
- date range ends

Existing month/day/range/time/uncertainty behavior remains unchanged.

### Documentation verification path

The checked-in archival examples use the dedicated `examples/archive-eprint-demo-style.yaml` demo style as the source of truth for the rendered examples shown in `docs/examples.html`.

The historical archival example uses valid EDTF `-0099` so the rendered output is `100 BC`.

## Implementation Notes

- Use the parsed `citum_edtf::Year` when converting negative years so interval endpoints and simple dates share one conversion path.
- For negative years with unspecified digits, keep current raw EDTF-style rendering until a separate policy is specified.
- The examples page should explicitly note that EDTF uses astronomical year numbering.

## Acceptance Criteria

- [x] Valid EDTF `-0099` renders as `100 BC` with the default English locale.
- [x] A negative EDTF full date renders the converted historical year with the era suffix.
- [x] A negative EDTF interval renders converted historical years at both endpoints.
- [x] `DateTerms::en_us()` provides a default `before-era` value.
- [x] The checked-in archival demo style renders the historical manuscript output shown in `docs/examples.html`.
- [x] Docs/examples use valid EDTF historical-year input and explain the astronomical-year convention.

## Changelog

- 2026-03-29: Initial version.
