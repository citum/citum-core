# Forward-Compatibility Specification

**Status:** Draft
**Version:** 0.1
**Date:** 2026-05-15
**Related:** bean `csl26-2a0b`, bean `csl26-fuw7`, `docs/architecture/DESIGN_PRINCIPLES.md`, `docs/reference/SCHEMA_VERSIONING.md`, `docs/policies/ENUM_VOCABULARY_POLICY.md`, `docs/architecture/EXTENSIBILITY_STRATEGY_2026-03-14.md`

## Purpose

Define the contract that an older Citum engine must honor when it parses a
newer style, reference, or locale that uses features it does not yet
understand. The goal is to make most non-template feature additions safe to
ship as a `minor` schema bump: producers can ship styles or data using new
options, attribute enum values, or locale terms without breaking older
engines. Older engines surface a single, consistent warning channel instead
of raw serde errors.

Template grammar changes and brand-new top-level reference classes are
explicitly out of scope: those remain `major`-level changes that older
engines may reject.

## Scope

**In scope** (must soft-degrade with a warning):

- Additive variants on attribute enums referenced inside templates or
  inside already-dispatched reference structs:
  `ContributorRole`, `MonographType`, `SerialComponentType`,
  `CollectionType`, `MonographComponentType`, `TermForm`, `DateForm`,
  `NumberingType`, `GrammaticalGender`.
- New optional fields on style option structs (anywhere
  `deny_unknown_fields` is currently applied in
  `crates/citum-schema-style/`).
- New optional fields on reference data types
  (`crates/citum-schema-data/src/reference/types/`).
- New optional top-level sections in a `Style` document.
- New locale term keys that a style references but the engine's vocabulary
  does not yet enumerate.
- New `custom.<namespace>.*` keys (the existing inert-metadata escape
  hatch from [`EXTENSIBILITY_STRATEGY_2026-03-14.md`](../architecture/EXTENSIBILITY_STRATEGY_2026-03-14.md)).

**Out of scope** (older engines may hard-fail; producers must bump major):

- Changes to template grammar — new required variants of
  `TemplateComponent`, new required fields on an existing template
  component, changed semantics of an existing variant.
- Unknown values of the top-level `InputReference::class` discriminator.
  See [§ InputReference discriminator](#inputreference-discriminator).
- Renames or removals of any field, variant, or term key. See
  [`ENUM_VOCABULARY_POLICY.md` §Backward Compatibility](../policies/ENUM_VOCABULARY_POLICY.md).

This spec governs **forward** compatibility (old engine reads new data).
**Backward** compatibility (new engine reads old data) is delivered by
`#[serde(default)]` + `Option<T>` on every additive field; that part of
the contract is already enforced by code review and is not restated here.

## Outcome classes

Every loadable artifact (style, reference, locale) lands in exactly one
outcome class when an older engine reads it:

| Class | Meaning |
|---|---|
| `Pass` | The new feature was silently accepted and the artifact behaves as if the feature were absent. No warning emitted. Used for inert metadata (`custom.*`) and for new fields the older engine simply ignores. |
| `SoftDegrade` | The new feature was acknowledged but its effect was dropped or replaced with a documented fallback. A warning is emitted through the documented channel. The render still produces output. |
| `HardFail` | Parse or load returned an error. No render. Used only for grammar-level changes the engine cannot reason about and for unknown top-level reference classes. |

`SoftDegrade` is the dominant target. `Pass` is acceptable only when the
older engine genuinely cannot infer the producer's intent (e.g.
namespaced custom keys). `HardFail` is reserved for the out-of-scope
categories above.

## Warning channel

All `SoftDegrade` outcomes surface through one channel so consumers — the
CLI, `citum-hub`, the language bindings — present them uniformly. The
channel already exists in skeletal form at
`crates/citum-cli/src/commands.rs:1716`, where `citum check` compares
`style.version` against `SchemaVersion::default()` and emits a warning
when minor > supported minor.

The channel must be extended so that:

1. Engine load APIs (`Style::from_yaml_str`, `Locale::from_yaml_str`,
   reference loaders in `citum-io`) accumulate `CompatibilityWarning`
   records during parse instead of returning early with a serde error
   for additive cases.
2. `citum check` reports each warning with the field path, the offending
   value, the fallback applied, and the schema version that introduced
   the feature (when known).
3. Bindings expose the warning list on the returned style / reference /
   locale handle.

The shape of `CompatibilityWarning` is intentionally left to the
follow-up implementation beans; this spec only fixes the contract.

## Scope table

Each row is anchored to one or more cases in
`crates/citum-engine/tests/forward_compatibility.rs`. The "Currently
observed" column reflects what the engine does **today**, not what the
contract demands.

| # | Category | Example | Desired | Currently observed | Follow-up |
|---|---|---|---|---|---|
| 1 | Attribute enum in template | `contributor: producer` (new `ContributorRole`) | `SoftDegrade` | `HardFail` (raw serde error) | Tolerant enum deserializer |
| 2 | Attribute enum in data | `class: monograph, type: dance-performance` | `SoftDegrade` | `HardFail` | Tolerant enum deserializer |
| 2b | Top-level `class` value | `class: dance-performance` | `HardFail` (opt-out, major bump) **or** `SoftDegrade` via catch-all variant | `HardFail` | InputReference discriminator design |
| 3 | Locale form | `form: vocative` | `SoftDegrade` (fall back to `Long`) | `HardFail` | Tolerant enum deserializer |
| 4 | Date form | `form: month-and-day` (new `DateForm`) | `SoftDegrade` | `HardFail` | Tolerant enum deserializer |
| 5 | New style option key | `options.contributors.future-key: true` | `SoftDegrade` (ignore field, warn) | `HardFail` (`deny_unknown_fields`) | Capture-unknown-fields wrapper |
| 6 | New top-level style section | `experiments: { ... }` | `SoftDegrade` | `HardFail` (`deny_unknown_fields` on `Style`) | Capture-unknown-fields wrapper |
| 7 | New reference field | `audience: scholarly` on `Monograph` | `Pass` (silent ignore) **or** `SoftDegrade` (warn) | `Pass` (silent — known gap) | Reference-data silent-acceptance fix |
| 8 | New locale term key | term key not in current vocabulary | `SoftDegrade` at lookup, render the key as fallback | depends on lookup path — to be measured | Tolerant locale lookup |
| 9 | Custom namespace | `custom.publisher-x.foo: true` | `Pass` | `Pass` (control case) | none |
| 10 | Version-only signal | `version: "99.0"` on otherwise valid style | `SoftDegrade` via `citum check` | `SoftDegrade` (control case) | none |
| 11 | Template grammar add | hypothetical `loop:` variant | `HardFail` (opt-out, major) | `HardFail` (control case) | none |
| 12 | Required template field add | new required field on `TemplateVariable` | `HardFail` (opt-out, major) | `HardFail` (control case) | none |

The truth-of-record for these outcomes is
`crates/citum-engine/tests/snapshots/forward_compat_gaps.snap`. The
test corpus runs every row, captures `(declared, observed)` pairs, and
asserts the snapshot. Documentation rows that drift from the snapshot
must be updated in the same PR that changes engine behavior.

## InputReference discriminator

`InputReference` uses `#[serde(tag = "class")]` at
`crates/citum-schema-data/src/reference/mod.rs:74`. The tag value
determines which concrete struct (`Monograph`, `SerialComponent`,
`LegalCase`, …) the rest of the payload deserializes into. An unknown
`class` value has no struct shape to fall into; serde cannot type the
payload at all.

Two viable shapes (one must be picked before this spec graduates to
`Active`):

**Option A — Catch-all variant.** Add
`InputReference::Unknown(UnknownReference { class: String, fields:
serde_json::Map<String, Value> })`. A brand-new class round-trips its
data, emits a `SoftDegrade` warning, and degrades to a generic
rendering path (likely "render title + author + year, drop everything
else"). Preserves the spec's headline rule that new features warn
rather than break.

**Option B — Major-bump category.** Declare new top-level classes as a
second opt-out category alongside template grammar. Producers must bump
major and accept that older engines hard-fail. Simpler to implement;
worse user experience.

This spec recommends Option A and tracks the design in a follow-up
bean. Until then row 2b stays `HardFail` in the snapshot.

## Producer obligations

Style and tool authors:

1. **Bump `style.version`** whenever the style uses a feature added in a
   newer schema minor. The `version` is the primary signal `citum check`
   uses to emit the global "this style targets a newer engine" warning.
2. **Prefer `custom.<namespace>.*`** for genuinely experimental or
   institution-specific metadata. The portable schema is not the right
   incubation surface; see
   [`EXTENSIBILITY_STRATEGY_2026-03-14.md`](../architecture/EXTENSIBILITY_STRATEGY_2026-03-14.md).
3. **Treat template grammar changes as `major`-only.** New
   `TemplateComponent` variants and changes to existing-component
   shapes are not forward-compatible and must not ship as `minor`.

## Consumer obligations

Engine and binding authors:

1. **Never lose data silently.** Any feature that the older engine
   cannot honor must produce a `CompatibilityWarning`. Silent `Pass` is
   reserved for namespaced `custom.*` and for cases where the data
   simply has no effect on rendering.
2. **Render must still produce output** in every `SoftDegrade` case.
   The user should see what the older engine *could* do, plus a
   warning.
3. **Surface warnings through the documented channel** rather than
   inventing a new one.

## Promotion path

Borrowed from
[`EXTENSIBILITY_STRATEGY_2026-03-14.md`](../architecture/EXTENSIBILITY_STRATEGY_2026-03-14.md)
and applied here:

1. New behavior starts as `custom.<namespace>.*` if it is exploratory or
   non-portable.
2. If multiple styles need it, it graduates to a typed schema addition
   (`Option<T>` field, new attribute-enum variant).
3. The schema addition lands as a `minor` bump. Older engines see it as
   `SoftDegrade`. Newer engines honor it.
4. Stable promoted features may eventually become required — that
   transition is the `major` bump and is governed by the deprecation
   policy in [`ENUM_VOCABULARY_POLICY.md`](../policies/ENUM_VOCABULARY_POLICY.md).

## Acceptance checks

This spec is `Active` once the snapshot in
`crates/citum-engine/tests/snapshots/forward_compat_gaps.snap` shows
`declared == observed` for every row whose desired class is `Pass` or
`SoftDegrade`. Rows whose desired class is `HardFail` are already
aligned and do not block promotion.

## Worked scenarios

### A new contributor role ships in v0.52

A style needs `producer` for a film-credits template:

```yaml
bibliography:
  template:
    - contributor: producer
      form: long
```

Engine 0.51 reads this style. Today: serde rejects the unknown variant
with a parse error, the user sees "unknown variant `producer`". Under
this spec: `SoftDegrade` — the template component renders as empty
(no producer field on the data), and a `CompatibilityWarning` is
emitted naming `contributor: producer` and the schema version that
introduced it.

### A style adds a new top-level section

A style ships an `experiments` block:

```yaml
version: "0.52"
experiments:
  inline-author-disambiguation: true
bibliography: ...
```

Engine 0.51 reads this. Today: `Style` has `deny_unknown_fields`,
parse fails. Under this spec: `SoftDegrade` — the `experiments` block
is captured into an opaque `unknown_sections` map (or similar), a
warning is emitted, and the rest of the style renders normally.

### A reference adds a new field

```yaml
- id: smith2026
  class: monograph
  title: ...
  audience: scholarly
```

Engine 0.51 reads this. Today: `Monograph` cannot deny unknown fields
(serde+`#[serde(tag)]` limitation), so the `audience` key is silently
discarded with no warning. Under this spec: the silent drop becomes a
`SoftDegrade` warning so users know data was not honored. Rendering is
otherwise unchanged.

### A reference uses a brand-new class

```yaml
- id: perf2026
  class: dance-performance
  title: ...
```

Today: parse fails. Under Option A (recommended): the reference lands
in `InputReference::Unknown { class: "dance-performance", fields: ... }`
and degrades to a generic render. Under Option B: hard fail; the
producer must bump major and accept the break.

## Non-goals

- Defining the exact Rust shape of `CompatibilityWarning`.
- Specifying which `style.version` value corresponds to which feature
  (the schema changelog already records that).
- Adding any tolerant deserializer code in this spec's PR.
- Changing the contract for `backward` compatibility (new engine, old
  data) — that path is already handled by `Option<T>` + `#[serde(default)]`.
