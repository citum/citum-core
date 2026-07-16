# Supplementary Reference Identifiers Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-07-15
**Related:** bean `csl26-8uxa`; [`MULTILINGUAL.md`](./MULTILINGUAL.md); [`EXTENSIBILITY_STRATEGY.md`](../architecture/EXTENSIBILITY_STRATEGY.md)

## Purpose

Citum references need a typed, renderable home for standardized identifiers
that do not yet justify dedicated schema fields. CSTR is the first supported
use case, while the same representation can carry identifiers such as ECLI
without adding an unrelated field for every registry.

## Scope

This specification defines supplementary identifier storage, validation, and
template rendering. Existing first-class fields such as DOI, ISBN, ISSN, URL,
PMID, and PMCID remain authoritative and are outside this map. Identifier
resolution, network lookup, and identity-based disambiguation are out of scope.

## Design

References may contain an `identifiers` mapping:

```yaml
class: monograph
type: book
title: Example
identifiers:
  cstr: "32012.36.1001024.2023.0328"
```

Keys must begin with an ASCII lowercase letter and contain only lowercase
letters, digits, and single kebab-case separators. Empty segments, uppercase
letters, underscores, whitespace, and punctuation other than `-` are invalid.

The following names are reserved because Citum already provides first-class
fields or accessors for them: `ads-bibcode`, `doi`, `docket-number`,
`eprint-id`, `isbn`, `issn`, `patent-number`, `pmcid`, `pmid`,
`report-number`, `standard-number`, and `url`. Inputs must use those dedicated
fields instead of duplicating them in `identifiers`.

Styles render a supplementary identifier with an identifier component:

```yaml
- identifier: cstr
  prefix: "CSTR: "
```

The component supports the standard rendering attributes. A missing or empty
identifier produces no component, including no prefix, suffix, or wrapping
punctuation. Normal group suppression therefore applies without a special
identifier rule.

CSL-M migration maps both `CSTR:` and `tex.cstr:` Zotero Extra entries to the
`cstr` key. An identical duplicate is harmless. When the values conflict, the
direct `CSTR:` entry takes precedence and migration emits a diagnostic.

Supplementary identifiers are typed renderable data. They are not
`custom.*` metadata, and engines must not interpret `custom.*` as a fallback
identifier source.

## Implementation Notes

The schema exposes a validated `IdentifierName` key type and a deterministic
ordered map. Adding a dedicated first-class field later requires reserving its
name and defining an explicit migration path for existing supplementary data.

## Acceptance Criteria

- [x] Valid supplementary identifiers round-trip through reference JSON and YAML.
- [x] Malformed and reserved identifier names fail during deserialization.
- [x] Generated reference schemas expose `identifiers` on every known reference class.
- [x] Identifier template components render present values and suppress missing values with their affixes.
- [x] CSL-M migration extracts CSTR from Zotero Extra and diagnoses conflicts.

## Changelog

- v1.0 (2026-07-15): Defined validated supplementary identifiers and template rendering.
- v1.1 (2026-07-15): Defined CSTR extraction precedence and stable conflict diagnostics.
