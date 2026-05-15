---
# csl26-0ksu
title: Capture-unknown-fields wrapper for style schemas
status: todo
type: feature
priority: high
created_at: 2026-05-15T14:48:07Z
updated_at: 2026-05-15T14:48:07Z
blocked_by:
    - csl26-2a0b
---

Promote forward-compat rows 05 and 06 from observed=HardFail to observed=SoftDegrade.

Replace deny_unknown_fields on style option / template / top-level Style structs with a wrapper that captures unknown keys into an opaque map and emits a CompatibilityWarning per skipped key. Strict mode (e.g. 'citum check --strict') preserves the current typo-catching behavior.

Files affected (audit, not exhaustive):
- crates/citum-schema-style/src/options/*.rs
- crates/citum-schema-style/src/lib.rs (Style)
- crates/citum-schema-style/src/template.rs

Spec: docs/specs/FORWARD_COMPATIBILITY.md
Snapshot rows: 05-new-option-key, 06-new-top-level-section
