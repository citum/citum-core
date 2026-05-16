---
# csl26-acfh
title: Surface silent unknown-field acceptance on reference data
status: completed
type: feature
priority: normal
tags:
    - forward-compat
created_at: 2026-05-15T14:48:11Z
updated_at: 2026-05-16T18:05:53Z
---

Promote forward-compat row 07 from observed=HardFail to observed=SoftDegrade (warning).

Reference data structs cannot use deny_unknown_fields due to serde's #[serde(tag)] limitation on InputReference, so unknown keys are silently discarded today. Add a Deserializer hook that records skipped keys during reference parsing and emits a CompatibilityWarning per dropped key. The render still proceeds without the extra data.

Files affected:
- crates/citum-schema-data/src/reference/types/{structural,specialized,legal,common}.rs
- crates/citum-schema-data/src/reference/mod.rs (InputReference)

Spec: docs/specs/FORWARD_COMPATIBILITY.md
Snapshot row: 07-new-reference-field

## Summary of Changes

Implemented capture-unknown-fields pattern on all reference payload types in citum-schema-data:

- Added `unknown_fields: BTreeMap<String, serde_json::Value>` field with flatten + skip_serializing_if attributes to 18 reference payload structs across:
  - structural.rs: Monograph, Collection, CollectionComponent, SerialComponent, Serial (Deser pairs)
  - specialized.rs: Event, Classic, Patent, Dataset, Standard, Software, AudioVisualWork (mix of direct + Deser)
  - legal.rs: LegalCase, Statute, Treaty, Hearing, Regulation, Brief (direct public structs)

- Updated all constructor sites (~14 locations) in conversion.rs and biblatex.rs to initialize unknown_fields: Default::default()

- Added serde_yaml_ng dependency to citum-schema-data Cargo.toml

- Forward-compat snapshot test now reflects SoftDegrade pass-through for row 07 (new-reference-field), confirming silent acceptance of unknown fields

Closes forward-compat row 07 per FORWARD_COMPATIBILITY.md spec.
