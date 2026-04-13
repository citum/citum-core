# Strong Domain Types Phase 1 Specification

**Status:** Active
**Date:** 2026-04-13
**Supersedes:** None
**Related:** `crates/citum-schema-data/src/reference/types/common.rs`, `scripts/audit-primitive-types.sh`

## Purpose

This specification defines the first schema-first pass for replacing primitive `String`
aliases with dedicated domain types in Citum. The goal is to strengthen the public
reference model without changing JSON/YAML wire formats or adding new parser
dependencies.

## Scope

In scope:

- Convert `RefID` from a `String` alias into a dedicated transparent newtype.
- Convert `LangID` from a `String` alias into a dedicated transparent newtype.
- Update direct schema/data consumers that rely on string-like behavior.
- Preserve serde, schema, and bindings output as scalar strings.
- Capture the cargo-dependency escalation path for stricter language-tag validation.

Out of scope:

- Validating BCP 47 syntax in `LangID` during this phase.
- Style, grouping, and registry identifier wrappers in `citum-schema-style`.
- CLI-only argument structs and engine-internal scratch structs.
- Free-form text wrappers for headings, titles, quotes, or general display strings.
- Treating the primitive-type audit script as a policy gate.

## Design

### Variant A: No `Cargo.toml` changes

Phase 1 uses transparent newtypes with no new dependencies.

- `RefID` is an opaque wrapper over `String`.
- `LangID` is an opaque wrapper over `String`.
- Both types expose string ergonomics needed by existing code:
  `Display`, `FromStr`, `From<String>`, `From<&str>`, `AsRef<str>`, `Borrow<str>`,
  and `Deref<Target = str>`.
- Both types derive serde/schema/bindings traits so their external wire shape
  remains a plain string.

This keeps the change as a type-safety refactor rather than a behavioral
validation change.

### Variant B: Strict language-tag validation

If a future phase adds BCP 47 validation to `LangID`, that work must:

- receive explicit confirmation before any `Cargo.toml` or `Cargo.lock` edit;
- add and justify a language-tag parsing dependency;
- define invalid-input behavior for serde/YAML/JSON ingestion;
- verify that schemas and TypeScript bindings remain string-shaped;
- be treated as a compatibility-sensitive follow-up PR.

### Audit Script Decision

The temporary root `types.sh` is promoted into `scripts/audit-primitive-types.sh`
as a maintained triage tool for future type-strengthening passes.

- Default scope is limited to Rust source under `crates/`.
- Test, bench, doc, and obvious helper paths are excluded.
- Output is grouped by public API and semantic field names first.
- The script is informational only and does not define a merge gate.

## Implementation Notes

- Favor trait-based string ergonomics over broad code churn in downstream crates.
- Preserve `HashMap<LangID, _>::get("en")` and `Option<RefID>::as_deref()` behavior.
- Keep `set_id` ergonomic by accepting `impl Into<RefID>`.
- Defer validated language-tag parsing until the repo explicitly opts into a
  new dependency and tighter runtime behavior.

## Acceptance Criteria

- [x] `RefID` is a dedicated transparent type rather than a type alias.
- [x] `LangID` is a dedicated transparent type rather than a type alias.
- [x] JSON/YAML serde output for both types remains a scalar string.
- [x] Schema generation still reports both types as strings.
- [x] Direct schema/data consumers compile without `Cargo.toml` changes.
- [x] The primitive-type audit script lives under `scripts/` and is documented
      as a triage tool, not a policy gate.

## Changelog

- 2026-04-13: Initial version and first implementation.
