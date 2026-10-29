# CSL DOI-to-URL fallback migration specification

**Status:** Active
**Version:** 1.1
**Date:** 2026-10-29
**Supersedes:** None
**Related:** `docs/specs/GROUP_SELECT.md`,
`docs/specs/RENDER_WHEN_CONTRACT.md`, `csl26-zyic`

## Purpose

`citum-migrate` must preserve the common CSL rule "render the DOI when it is
present; otherwise render the URL." Citum represents this rule as an ordered
`select: first` group. Without this conversion, the migrator flattens both
branches into ordinary template components and can render the DOI and URL
together.

## Scope

This specification covers one CSL `<choose>` shape after macro expansion:

```xml
<choose>
  <if variable="DOI">...</if>
  <else-if variable="URL">...</else-if>
</choose>
```

The conversion applies in citation and bibliography templates. It does not
generalize to other variable chains, add migration diagnostics, or change the
schema or engine contract for `select: first`.

## Design

### Recognition

The migrator recognizes the fallback only when all of these conditions hold:

1. The `<choose>` has one `if` branch followed by one `else-if` branch and no
   `else` branch.
2. The `if` branch tests only `DOI`; the `else-if` branch tests only `URL`.
3. Each branch has no type, numeric, uncertain-date, locator, or position
   predicate.
4. Each branch uses an omitted `match`, `match="any"`, or `match="all"`.
   `match="none"` is not eligible.
5. Each expanded branch is non-empty and contains an output node for the
   variable it tests. The DOI branch must render `DOI`, and the URL branch must
   render `URL`.

A near-match follows the existing generic conditional migration path. The
migrator does not guess that a mixed or indirect condition has fallback
semantics.

### Output

The recognized `<choose>` becomes one `TemplateGroup` with `select: first`.
The DOI branch is the first candidate and the URL branch is the second.

A branch that compiles to one template component becomes a direct candidate.
A branch that compiles to multiple components becomes one nested `select: all`
group, so the outer selection treats the complete branch as one candidate.
Child order, affixes, wrapping, delimiters, and formatting remain attached to
the components or group that produced them.

The migrator carries the recognized fallback through an explicit internal IR
node. Generic condition handling remains unchanged for unrecognized trees.

## Implementation Notes

Recognition belongs in the upsampler before generic `<choose>` lowering. The
internal ordered-fallback node stores candidates as ordered node lists. The
compressor and other recursive IR consumers must visit every candidate.

The template compiler converts the internal node directly to the existing
`TemplateGroupSelect::First` schema value. No new public style field or engine
behavior is required.

## Acceptance Criteria

- [x] The exact DOI-then-URL shape emits one `select: first` group in source
      order.
- [x] A multi-component URL branch remains one atomic fallback candidate.
- [x] Citation and bibliography compilation preserve the same fallback
      structure.
- [x] Extra branches, an `else`, `match="none"`, mixed predicates, and branches
      that do not render their tested variable stay on the generic conditional
      path.
- [x] Targeted migration tests cover the accepted shape and every rejected
      near-match.
- [x] Migration checks for IEEE, APA, and Harvard Cite Them Right report their
      before-and-after fidelity results without committed style regeneration.

## Changelog

- v1.0 (2026-10-29): Initial draft.
- v1.1 (2026-10-29): Activated after implementing ordered fallback migration
  and its regression coverage.
