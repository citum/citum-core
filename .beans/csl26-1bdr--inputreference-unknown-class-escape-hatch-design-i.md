---
# csl26-1bdr
title: InputReference unknown-class escape hatch (design + impl)
status: todo
type: feature
priority: normal
created_at: 2026-05-15T14:48:20Z
updated_at: 2026-05-15T14:48:20Z
blocked_by:
    - csl26-2a0b
---

Decide and implement how forward-compat row 02b is handled.

InputReference uses #[serde(tag = "class")] at crates/citum-schema-data/src/reference/mod.rs:74. An unknown class has no struct shape to deserialize into, so the standard tolerant-enum trick does not apply. Two viable shapes (see docs/specs/FORWARD_COMPATIBILITY.md § InputReference discriminator):

- **Option A — Catch-all variant**: add InputReference::Unknown(UnknownReference { class: String, fields: serde_json::Map<String, Value> }). Brand-new classes round-trip data, emit a SoftDegrade warning, and degrade to a generic rendering path. Recommended in the spec.
- **Option B — Major-bump category**: declare new top-level classes as a second opt-out category alongside template grammar.

Pick one, implement, and update the snapshot. Until decided, row 02b correctly observes HardFail.

Spec: docs/specs/FORWARD_COMPATIBILITY.md
Snapshot row: 02b-discriminator-class
