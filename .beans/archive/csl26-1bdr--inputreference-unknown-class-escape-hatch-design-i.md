---
# csl26-1bdr
title: InputReference unknown-class escape hatch (design + impl)
status: completed
type: feature
priority: deferred
tags:
    - forward-compat
created_at: 2026-05-15T14:48:20Z
updated_at: 2026-05-16T13:10:33Z
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



## Update 2026-05-15 — deferred per pre-1.0 stance

Per author feedback on PR #715: pre-public-release, there is no concern for backward-compatibility breakage, so the headline forward-compat rule does not need to hold for top-level reference classes yet. The spec now declares new `InputReference` classes as a major-bump opt-out (alongside template grammar). No design work needed today.

Revisit post-1.0 only if real-world evidence shows brand-new classes are common enough to justify the catch-all variant engineering. Snapshot row 02b stays `declared=HardFail observed=HardFail` (correct opt-out) until then.

## Update 2026-05-15 — architecture spec opened

Reopened the deferred discriminator question with a wider lens. Pre-1.0
freedom + the existing `deny_unknown_fields` suppression at 16 sites (caused
by the same `#[serde(tag)]` collision) made the wider review worth doing
now rather than after 1.0.

Spec: `docs/specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md` (Draft).
Weighs five options (status quo, catch-all variant, flat struct, open class
+ registry, shared base + class-specific overlay). Recommends Option E
(shared base + class-specific overlay) as the only option that restores
strictness end-to-end while keeping the hybrid type model intact.

Implementation lands in a follow-up child bean once the spec is Active.

## Update 2026-05-15 (b) — spec rewritten per review

Copilot review on PR #715 (review run #716) surfaced three serde
flaws in the v0.1 draft's Option E sketch plus one FFI path error.
Validated the corrected shape with a throwaway Rust spike
(8 tests, all green), then rewrote the spec to v0.2:

- single-design framing (Option E only); comparative analysis dropped
- hand-written `Deserialize` dispatcher on the outer struct; no
  `#[serde(tag)]` on `ClassExtension`; restores `deny_unknown_fields`
  end-to-end
- `UnknownClassData { class, fields }` populated by the dispatcher
  (not by `#[serde(other)]`, which can't carry data)
- JSON Schema uses `unevaluatedProperties: false` (draft 2020-12,
  already our target) for the per-class strictness composition
- migration sites corrected: `BibRefContext` lives in citum-io,
  not engine FFI

Error-UX examples in §"Error UX" are quoted verbatim from the
spike's test output rather than invented.

## Update 2026-05-15 (c) — v0.3 review pass

Resolves Perplexity's additional review points on v0.2:

- Wire format gets its own up-front section showing flat YAML with
  an explicit "no wrapper key" callout (`class_data:` does not appear
  on the wire — Rust internal only).
- Rust API documented as a first-class section: shared fields as
  public struct fields, `class()` accessor returning a typed
  `ReferenceClass`, `extension()` + per-class `as_<class>()`
  accessors. Internal field is `pub(crate)`, name not part of the
  public contract.
- `ReferenceClass` added as a typed enum so schemars emits a closed
  `enum` schema for `class:` automatically; `Unknown(String)` variant
  for the SoftDegrade path.
- JSON Schema section adds a schemars-alignment invariant
  (dispatcher and generated schema agree on accept-vs-reject for
  known classes; asymmetry on unknown classes is intentional) and
  names the custom `#[schemars(schema_with = ...)]` requirement on
  `InputReference`.
- `UnknownClassData::fields` type settled as
  `serde_json::Map<String, serde_json::Value>`.
- Acceptance criteria adds bullet 4: schema-alignment tests in the
  implementation PR.
