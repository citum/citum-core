# Schema Split and Convert Namespace Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-03-10
**Supersedes:** None
**Related:** docs/policies/TYPE_ADDITION_POLICY.md

## Purpose
Define a crate-level schema split and a new CLI conversion namespace that cleanly separates style and data models while expanding bibliography conversion across native and legacy formats.

## Scope
In scope: `citum-schema` split into `citum-schema-data` and `citum-schema-style` with a compatibility facade; new `citum convert` subcommand layout; bibliography format conversion for native formats plus CSL-JSON, BibLaTeX, and RIS; lossless-first conversion behavior; test and migration updates.

Out of scope: rendering behavior changes, style semantics changes, non-bibliography import pipelines, and network services.

## Design
- Add crate `citum-schema-data` containing citation/reference/input bibliography models and data conversion hooks.
- Add crate `citum-schema-style` containing style/options/template/grouping/locale/preset/embedded models.
- Retain crate `citum-schema` as facade that re-exports both crates for import compatibility.
- Replace legacy `citum convert <input> -o <output>` syntax with explicit subcommands:
  - `citum convert refs <input> -o <output> [--from <fmt>] [--to <fmt>]`
  - `citum convert style <input> -o <output>`
  - `citum convert citations <input> -o <output>`
  - `citum convert locale <input> -o <output>`
- `convert refs` supports native (`citum-yaml`, `citum-json`, `citum-cbor`) and legacy (`csl-json`, `biblatex`, `ris`) formats.
- Default format behavior infers from extensions; `--from`/`--to` override inference.
- Conversion is lossless-first: unmapped fields are preserved in `custom` metadata for round-trip compatibility.
- Model additions are allowed only when they satisfy type policy constraints and this spec's guardrails.

### Model Addition Guardrails
Additions to native model are accepted only when all are true:
1. Fidelity gap is demonstrated using real conversion examples.
2. Existing native fields plus `custom` cannot represent semantics cleanly.
3. Addition is explicit, additive, optional, and serde-driven.
4. Prior-art mapping is documented (prefer biblatex alignment).
5. Change includes docs and tests and passes active type policy.

When these criteria are not met, fields remain in `custom`.

## Implementation Notes
- Reuse existing BibLaTeX parser path in engine.
- Reuse existing CSL-JSON conversion path via legacy conversion support.
- Add internal RIS adapter with deterministic mapping and unknown-field preservation.
- Mark this spec `Active` in the first implementation commit.

## Acceptance Criteria
- [ ] New schema crates compile and facade preserves existing import patterns.
- [ ] New `convert` namespace replaces old syntax and command help reflects it.
- [ ] `convert refs` supports native + CSL-JSON + BibLaTeX + RIS formats.
- [ ] Conversion matrix tests and lossless round-trip tests pass.
- [ ] Type additions, if any, are documented as fidelity-driven deltas.

## Changelog
- v1.0 (2026-03-10): Initial draft.
