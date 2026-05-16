---
# csl26-acfh
title: Surface silent unknown-field acceptance on reference data
status: todo
type: feature
priority: normal
tags:
    - forward-compat
created_at: 2026-05-15T14:48:11Z
updated_at: 2026-05-16T14:15:58Z
---

Promote forward-compat row 07 from observed=HardFail to observed=SoftDegrade (warning).

Reference data structs cannot use deny_unknown_fields due to serde's #[serde(tag)] limitation on InputReference, so unknown keys are silently discarded today. Add a Deserializer hook that records skipped keys during reference parsing and emits a CompatibilityWarning per dropped key. The render still proceeds without the extra data.

Files affected:
- crates/citum-schema-data/src/reference/types/{structural,specialized,legal,common}.rs
- crates/citum-schema-data/src/reference/mod.rs (InputReference)

Spec: docs/specs/FORWARD_COMPATIBILITY.md
Snapshot row: 07-new-reference-field
