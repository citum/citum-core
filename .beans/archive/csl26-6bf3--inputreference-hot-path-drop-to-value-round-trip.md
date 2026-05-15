---
# csl26-6bf3
title: 'InputReference hot-path: drop to_value round-trip'
status: completed
type: task
priority: normal
created_at: 2026-05-16T00:15:35Z
updated_at: 2026-05-16T00:24:59Z
parent: csl26-1bdr
---

Resolve Copilot review comments #3251089405, #3251089411 (residual), #3251089475, #3251089491 on PR #717 — the three are the same architectural shape (`serde_json::to_value` round-trip on the hot path).

## Background

The discriminator cutover that landed in #717 took a serialize-then-extract shortcut to ship Layer 2–4 quickly:

- **Construction** (`boxed_reference_constructor!` / `from_known`): `serde_json::to_value(inner)` → `from_value::<SharedReferenceFields>`. One JSON-tree allocation + reparse per known-class construction.
- **Visitor / Deserialize** (`InputReference::deserialize` visit_map): buffers the entire object body into a `serde_json::Map` before dispatching to the typed inner deserializer. For YAML/TOML inputs this materializes a JSON-shaped intermediate per reference.
- **Serialize**: `serde_json::to_value(&extension)` → reflatten through SerializeMap. One JSON-tree allocation per serialization (hits BibLaTeX, RIS, CSL-JSON exporters).

For a bibliography with thousands of references, this is a measurable cost. Copilot is right that the previous tagged-enum design avoided it.

## Approach (recommended)

The root cause is the duplicated shared fields on `InputReference` (now pub(crate)). Resolving cleanly means:

1. Drop the 17 pub(crate) shared fields from `InputReference` — read everything through `self.extension` lazily via the existing accessors.
2. Drop `SharedReferenceFields` (or keep only as a deserialize-time aid).
3. Rewrite the `Deserialize` visitor to consume `class` first, then forward remaining keys to the typed inner deserializer via `MapAccessDeserializer` — no JSON intermediate.
4. Rewrite `Serialize` to wrap the inner struct's serializer and inject `class` at the top — no `to_value` round-trip.
5. Update `with_extension` / `from_known` to take only the extension.
6. Update `set_id` and other setters to update only the extension (one path).

## Acceptance criteria

- [ ] All current discriminator tests (13) still pass.
- [ ] `docs/schemas/bib.json` regen is wire-identical.
- [ ] A microbench (criterion) on construction + ser + deser of a 1000-reference bibliography shows the expected allocation reduction. Add the bench under `crates/citum-schema-data/benches/`.
- [ ] No `serde_json::to_value` in the hot path of either Serialize or Deserialize.
- [ ] Forward-compat unknown-class round-trip still works (since Unknown still needs a JsonMap for its captured fields, that path can keep the intermediate).

## Out of scope

- Layer 5 CompatibilityWarning plumbing (tracked separately).
- Transitional PascalCase constructor cleanup (block-level TODO remains; replace when snake_case factories are added).

## Summary of Changes

Resolved in the follow-up commit on PR #717.

- Deleted SharedReferenceFields struct + its from_body / from_serializable helpers entirely.
- Deleted the 17 duplicated pub(crate) shared fields on InputReference. The struct now contains only extension: ClassExtension. Cross-crate grep confirmed the fields were write-only (only set_id wrote self.id, no readers anywhere).
- Simplified boxed_reference_constructor! to a one-liner: no JSON work at all on the construction path (Copilot #3251089405).
- Simplified from_known to skip the SharedReferenceFields::from_body round-trip; only one serde_json::from_value per deserialization now (Copilot #3251089405 deserialize-side).
- Removed reference_class_and_body + serialize_known_reference helpers. Replaced with a FlatClassProxy struct that uses #[serde(flatten)] to splat the typed inner directly through the parent serializer (Copilot #3251089491). No to_value round-trip on serialize.
- The visitor's JsonMap buffering in visit_map is unchanged — it's intrinsic to a custom dispatcher with an Unknown fallback (#[serde(other)] only catches unit variants, not data-carrying ones). The downstream from_known no longer adds a second round-trip on top, which is the practical win on the deserialize side (Copilot #3251089475 partially addressed; the buffering itself is structurally required).
- set_id simplified to a single write (no more dual-storage sync).
- 2 new tests on top of the existing 13: serialize_emits_flat_object_with_class_first_and_no_nesting locks the flatten-proxy wire shape; round_trip_through_serde_value_preserves_every_known_class exercises Serialize -> from_value -> equality for 6 known classes + Unknown.

Gate: 1284 passed (up from 1282), clippy clean, fmt clean. Schema regen is wire-identical (jq -S diff confirms).
