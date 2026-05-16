---
# csl26-0ksu
title: Capture-unknown-fields wrapper for style schemas
status: completed
type: feature
priority: high
tags:
    - forward-compat
created_at: 2026-05-15T14:48:07Z
updated_at: 2026-05-16T18:30:57Z
---

Promote forward-compat rows 05 and 06 from observed=HardFail to observed=SoftDegrade.

Replace deny_unknown_fields on style option / template / top-level Style structs with a wrapper that captures unknown keys into an opaque map and emits a CompatibilityWarning per skipped key. Strict mode (e.g. 'citum check --strict') preserves the current typo-catching behavior.

Files affected (audit, not exhaustive):
- crates/citum-schema-style/src/options/*.rs
- crates/citum-schema-style/src/lib.rs (Style)
- crates/citum-schema-style/src/template.rs

Spec: docs/specs/FORWARD_COMPATIBILITY.md
Snapshot rows: 05-new-option-key, 06-new-top-level-section


## Summary of Changes

Implemented capture-unknown-fields pattern on style schema structs in citum-schema-style:

- Removed `deny_unknown_fields` from `Style`, `CitationSpec`, `BibliographySpec`, `LocalizedTemplateSpec`, `Config`, `ContributorConfig`, `LocatorConfig`, `LocatorKindConfig`, `LocatorPattern`, `Substitute`, `TitlesConfig` (and its inner kind structs).
- Added `unknown_fields: BTreeMap<String, serde_yaml::Value>` capture field with `#[serde(flatten, default, skip_serializing_if = ...)]` and `#[schemars(skip)]` on each affected struct.
- Threaded `unknown_fields` through manual `Default`, constructor literals, and the custom `Config` deserializer's wire→Self mapping.
- Updated the legacy `old_section_extends_key_is_rejected` test (renamed to `_is_captured_for_forward_compat`) to assert the new SoftDegrade behavior.

Closes forward-compat rows 05 (new style option key) and 06 (new top-level style section) per `docs/specs/FORWARD_COMPATIBILITY.md`. Snapshot at `crates/citum-engine/tests/snapshots/forward_compat_gaps.snap` now shows `declared == observed` for both rows.

Template-grammar opt-out (rows 11, 12) is unchanged — `template.rs` structs intentionally retain `deny_unknown_fields`.

The remaining nested option structs (`CitationOptions`, `BibliographyOptions`, `DateConfig`, `BibliographyConfig`, et al.) still hold `deny_unknown_fields`; closing the snapshot did not require them. A follow-up bean covers full strict→capture conversion for those structs plus the `citum check --strict` CLI surface.
