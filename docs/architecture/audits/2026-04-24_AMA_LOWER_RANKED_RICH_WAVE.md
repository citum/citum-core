# AMA Lower-Ranked Rich Wave

**Date:** 2026-04-24
**Status:** Completed

## Summary

This style-evolve wave kept the lower-ranked work scoped to the American Medical
Association family and paired it with the first official AMA rich-input
benchmark.

## Changes

- `american-medical-association` now has the supplemental
  `ama-zotero-bibliography` benchmark using
  `tests/fixtures/test-items-library/ama-11th.json`.
- The AMA 11 rich bibliography benchmark is fully clean at `71/71`.
- Existing lower-ranked AMA variants now retain their accepted citation
  behavior and reach full bibliography parity:
  - `american-medical-association-alphabetical`: `17/18` citations, `34/34`
    bibliography
  - `american-medical-association-no-et-al`: `18/18` citations, `34/34`
    bibliography
  - `american-medical-association-no-url`: `18/18` citations, `34/34`
    bibliography

## Decisions

- `american-medical-association-no-url-alphabetical` was not added in this wave.
  Its current legacy comparison remains below the existing maintained variants
  at `17/18` citations and `32/34` bibliography.
- Bracket and parenthesis AMA variants remain out of scope because the alias
  report shows citation-shape differences from the base AMA style.
- The remaining `american-medical-association-alphabetical` citation mismatch is
  a numeric citation-number ordering issue for the `webpage-single` scenario.
  Bibliography output is clean, so this wave preserves the existing citation
  behavior rather than forcing a broad processor change into a style PR.
