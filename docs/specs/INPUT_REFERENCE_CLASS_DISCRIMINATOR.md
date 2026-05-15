# InputReference Class Discriminator Design

**Status:** Draft
**Version:** 0.3
**Date:** 2026-05-15
**Related:** bean `csl26-1bdr`, [`FORWARD_COMPATIBILITY.md`](./FORWARD_COMPATIBILITY.md), [`../architecture/EXTENSIBILITY_STRATEGY_2026-03-14.md`](../architecture/EXTENSIBILITY_STRATEGY_2026-03-14.md), [`../policies/TYPE_ADDITION_POLICY.md`](../policies/TYPE_ADDITION_POLICY.md), [`../policies/ENUM_VOCABULARY_POLICY.md`](../policies/ENUM_VOCABULARY_POLICY.md)

## Purpose

Specifies the data shape and deserialization mechanics of
`InputReference`, the top-level bibliographic-data type in
`citum-schema-data`. The chosen shape is **a shared base struct + a
class-specific overlay dispatched by a hand-written `Deserialize`
impl**. This document is the design contract for the follow-up
implementation bean.

## Background

`InputReference` is currently a closed
`#[serde(tag = "class", rename_all = "kebab-case")]` enum at
`crates/citum-schema-data/src/reference/mod.rs:74` with 18 typed
variants (`Monograph`, `SerialComponent`, `LegalCase`, …). The shape
has two practical problems:

1. It hard-fails on any unknown `class:` value.
2. Serde's tag-replay behavior for newtype-variant payloads is
   incompatible with `#[serde(deny_unknown_fields)]` on the inner
   structs. The project has documented this at 16 sites across
   `crates/citum-schema-data/src/reference/types/legal.rs`,
   `crates/citum-schema-data/src/reference/types/specialized.rs`, and
   `crates/citum-schema-data/src/reference/types/structural.rs` with
   the comment `// deny_unknown_fields removed: incompatible with
   #[serde(tag)] on InputReference`. The consequence is forward-compat
   row 07 (`csl26-acfh`): a misspelled or future field on a reference
   is silently dropped instead of producing a parse error or warning.

Pre-1.0 is the right time to reshape the data model; we can change
the wire shape without backward-compatibility shims. The shape below
restores strictness end-to-end, makes unknown classes a non-event,
and gives hand-authors a single predictable outer object.

## Wire format

The on-wire object — YAML, JSON, CBOR — is a **flat map**. Shared
fields, the `class` discriminator, and class-specific fields all sit
at the top level. There is **no wrapper key** for class-specific data.

```yaml
- id: smith2026
  class: monograph        # discriminator
  title: A Book           # shared
  contributors: [...]     # shared
  issued: "2026"          # shared
  monograph-type: book    # class-specific (only when class: monograph)
  volume: 2               # class-specific
```

This is the contract producers and hand-authors target. **No `class_data:`,
`extension:`, or other wrapper key appears on the wire.** The Rust
struct holds class-specific data in a `pub(crate)` field whose name is
not part of the public contract and never appears in serialized output;
see [§Rust API](#rust-api) for the accessors that surface it instead.

Round-trip is symmetric: `parse(yaml) → serialize → yaml` is identical
modulo key ordering for both known and unknown classes. (Validated
empirically in the spike that fed this spec: a `class: audio-visual`
reference round-trips through the custom `Deserialize`/`Serialize` pair
to YAML with no wrapper key on output.)

## Rust API

Consumers interact with `InputReference` through this surface; the
internal layout that makes the dispatcher work is private to the
crate.

**Shared fields are public struct fields.**

```rust
reference.id           // RefID
reference.title        // Option<Title>
reference.contributors // Option<ContributorList>
reference.issued       // Option<EdtfString>
// ... etc.
```

**The class is reached through a method, not a field.**

```rust
reference.class() -> ReferenceClass
```

`ReferenceClass` is a typed enum with one variant per known class plus
an `Unknown(String)` variant for the forward-compat path. `class()`
returns by value (variants are cheap; the `Unknown` arm clones the
underlying string). The Rust call site never reaches for the field name.

**Class-specific data is reached through typed accessors.**

```rust
// Pattern-match form: gives access to the typed extension enum.
match reference.extension() {
    ClassExtension::Monograph(m)   => use_monograph(m),
    ClassExtension::LegalCase(c)   => use_legal_case(c),
    // ... 18 known-class arms ...
    ClassExtension::Unknown(u)     => use_generic(u),
}

// Direct-typed form: one accessor per known class.
if let Some(m) = reference.as_monograph() {
    use_volume(m.volume);
}

// Unknown-class accessor for the SoftDegrade path.
if let Some(u) = reference.unknown_class() {
    // u.class: String, u.fields: serde_json::Map
}
```

**Internal storage is `pub(crate)`.** The struct has one private field
holding the active `ClassExtension`. Crate-internal call sites and the
custom `Serialize`/`Deserialize` impls touch it directly; downstream
consumers cannot. The name of the private field is not part of the API
contract.

## Design

### Shape

The Rust data model has four pieces: an outer struct (`InputReference`),
a typed class discriminator (`ReferenceClass`), an inner enum
dispatching by class (`ClassExtension`), and per-class field structs
(`*Fields`).

```rust
pub struct InputReference {
    pub id: RefID,
    pub title: Option<Title>,
    pub contributors: Option<ContributorList>,
    pub issued: Option<EdtfString>,
    pub abstract_: Option<RichText>,
    pub custom: Option<CustomMap>,
    // ... every field that is genuinely shared across all classes ...

    pub(crate) extension: ClassExtension,
}

impl InputReference {
    pub fn class(&self) -> ReferenceClass { /* derived from `self.extension` */ }
    pub fn extension(&self) -> &ClassExtension { &self.extension }
    pub fn as_monograph(&self) -> Option<&MonographFields> { /* ... */ }
    pub fn as_legal_case(&self) -> Option<&LegalCaseFields> { /* ... */ }
    // ... one accessor per known class ...
    pub fn unknown_class(&self) -> Option<&UnknownClassData> { /* ... */ }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceClass {
    Monograph,
    CollectionComponent,
    SerialComponent,
    Collection,
    Serial,
    LegalCase,
    Statute,
    Treaty,
    Hearing,
    Regulation,
    Brief,
    Classic,
    Patent,
    Dataset,
    Standard,
    Software,
    Event,
    AudioVisual,
    /// Forward-compat: a class string the engine does not recognize.
    /// Not serialized as `unknown`; the raw string round-trips via the
    /// `class:` field on the wire and through `UnknownClassData::class`.
    #[serde(skip)]
    Unknown(String),
}

pub enum ClassExtension {
    Monograph(MonographFields),
    CollectionComponent(CollectionComponentFields),
    SerialComponent(SerialComponentFields),
    Collection(CollectionFields),
    Serial(SerialFields),
    LegalCase(LegalCaseFields),
    Statute(StatuteFields),
    Treaty(TreatyFields),
    Hearing(HearingFields),
    Regulation(RegulationFields),
    Brief(BriefFields),
    Classic(ClassicFields),
    Patent(PatentFields),
    Dataset(DatasetFields),
    Standard(StandardFields),
    Software(SoftwareFields),
    Event(EventFields),
    AudioVisual(AudioVisualFields),
    Unknown(UnknownClassData),
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct MonographFields {
    pub monograph_type: Option<MonographType>,
    pub volume: Option<u32>,
    pub edition: Option<String>,
    // ... other class-specific fields, all strictly typed ...
}

pub struct UnknownClassData {
    pub class: String,
    pub fields: serde_json::Map<String, serde_json::Value>,
}
```

`ClassExtension` is **not** annotated with `#[serde(tag)]`. The
dispatch happens in a hand-written `Deserialize` impl on
`InputReference` (sketched below).

**Notes on the shape:**

- `extension` is `pub(crate)`. The field name is a Rust internal and
  never appears on the wire (see [§Wire format](#wire-format)).
- `ReferenceClass` is the typed discriminator returned by
  `InputReference::class()`. It exists for two reasons: it gives
  schemars an `enum` schema for the `class:` property, and it gives
  Rust consumers a typed match on the class without descending into
  `ClassExtension`. The `Unknown(String)` variant is `#[serde(skip)]`
  because the wire form is always the raw `class:` string — never the
  literal `unknown`.
- `UnknownClassData::fields` is `serde_json::Map<String, serde_json::Value>`.
  `citum-schema-data` already depends on `serde_json` as a primary
  dependency (`crates/citum-schema-data/Cargo.toml:12`); `serde_yaml`
  is dev-only. The JSON value tree is format-agnostic — YAML input
  transcodes through serde to the same `Value` shape the JSON loader
  produces. Round-trip fidelity for YAML-specific layout features
  (anchors, tagged scalars) is not a goal.

### Why this shape works

The current shape (`#[serde(tag = "class")]` on
`InputReference` itself) cannot apply `#[serde(deny_unknown_fields)]`
to the per-class struct: when serde-derive deserializes
`InputReference::Monograph(Box<Monograph>)`, it replays the entire
input map — *including* the `class` tag — into `Monograph::deserialize`,
which then rejects `class` as an unknown field. Moving `#[serde(tag)]`
to an inner enum does **not** fix this: newtype-variant payloads
inside a `#[serde(tag)]` enum see the same replay.

A hand-written outer dispatcher removes the tag from the input before
it ever reaches a `*Fields` struct. Each `*Fields` struct sees only
its own keys, so `deny_unknown_fields` applies. The dispatcher also
emits scope-correct error messages because it knows which keys are
shared and which are class-specific for the active class — something
serde-derive cannot do across the tag boundary.

### Dispatcher algorithm

The outer `Deserialize` impl is a `Visitor` over a map. The algorithm
(verified by the spike at Phase 1 of the implementation plan; tests
in this section quote real spike output):

```text
1. Collect every (key, value) entry from the input map.
2. Extract `class` (required; missing → error).
3. For each remaining entry, look up the key:
     - in the static SHARED_KEYS list → shared bucket;
     - in the static class-specific keys for the active class
       (e.g. MONOGRAPH_KEYS) → class-specific bucket;
     - otherwise → error: "unknown field `<key>` for class `<class>`;
       known shared: [...], known fields for this class: [...]".
4. If the active class is not in KNOWN_CLASSES, do not enforce a
   class-specific key list; capture the class-specific bucket
   verbatim into UnknownClassData.fields. (Forward-compat path.)
5. Deserialize the shared bucket strictly into a SharedFields helper
   (deny_unknown_fields). Deserialize the class-specific bucket
   strictly into the matching *Fields struct (also deny_unknown_fields).
6. Construct InputReference { ...shared..., extension }.
```

The Rust skeleton, lifted verbatim from the spike:

```rust
impl<'de> Deserialize<'de> for InputReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        struct RefVisitor;

        impl<'de> Visitor<'de> for RefVisitor {
            type Value = InputReference;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a reference object with a `class` discriminator")
            }
            fn visit_map<M>(self, mut map: M) -> Result<InputReference, M::Error>
            where M: MapAccess<'de> {
                // collect, split, dispatch — see algorithm above
            }
        }
        deserializer.deserialize_map(RefVisitor)
    }
}
```

Companion `Serialize` impl flattens shared + class-specific back into
a single map, emitting `class:` after `id:` and the class-specific
fields in their natural order. The spike implements this and
round-trips both known and unknown classes (test 6 and test 8).

## Error UX

The dispatcher's error messages are quoted verbatim from the spike's
test output. The implementation PR will preserve these contracts.

### (a) Typo in a shared field

Input:

```yaml
id: smith2026
class: monograph
titel: A Book
```

Output (`HardFail`):

```text
unknown field `titel` for class `monograph`;
known shared fields: ["id", "title", "issued"],
known fields for this class: ["monograph-type", "volume", "edition"]
  at line 2 column 1
```

The error correctly attributes the typo to *neither* shared nor
class-specific scope and lists both, so the user can see whether they
meant `title` (shared) or some class-specific field.

### (b) Typo in a class-specific field on a known class

Input:

```yaml
id: smith2026
class: monograph
title: A Book
monogarph-type: book
```

Output (`HardFail`):

```text
unknown field `monogarph-type` for class `monograph`;
known shared fields: ["id", "title", "issued"],
known fields for this class: ["monograph-type", "volume", "edition"]
  at line 2 column 1
```

Same error shape as (a); the user sees `monograph-type` in the
known-for-this-class list and corrects the typo.

### (c) Class-specific field used under the wrong class

Input:

```yaml
id: smith2026
class: legal-case
title: Smith v. Jones
monograph-type: book
```

Output (`HardFail`):

```text
unknown field `monograph-type` for class `legal-case`;
known shared fields: ["id", "title", "issued"],
known fields for this class: ["court", "docket-number"]
  at line 2 column 1
```

The dispatcher routes each field by *active class*, so a
class-specific field used under the wrong class is caught at the
dispatcher rather than swallowed by a loose inner deserializer.

### (d) Unknown `class:` value

Input:

```yaml
id: perf2026
class: dance-performance
title: Pina
issued: "2011"
venue: Berlin
duration-minutes: 103
```

Outcome (parse-time, `SoftDegrade`):

- Parse succeeds.
- `reference.extension()` returns
  `ClassExtension::Unknown(UnknownClassData {
   class: "dance-performance",
   fields: { venue: "Berlin", duration-minutes: 103 } })`.
- `reference.class()` returns `ReferenceClass::Unknown("dance-performance".into())`.
- A `CompatibilityWarning` is emitted through the channel defined in
  [`FORWARD_COMPATIBILITY.md`](./FORWARD_COMPATIBILITY.md) §Warning channel.
- The warning channel is responsible for the user-facing message; the
  dispatcher's job is to ensure the parse succeeds and the data
  round-trips. Did-you-mean suggestions over the known
  `ReferenceClass` variants are an implementation detail of the
  warning emitter, not of this spec.

A genuine typo on `class:` (e.g. `class: monogarph`) follows the same
path — the engine captures it as Unknown rather than hard-failing,
because the dispatcher cannot tell a typo apart from a future class
the engine version simply doesn't know yet. The warning channel
surfaces both as `SoftDegrade`.

**Shared-field strictness is independent of class knowledge.** A typo
on a shared field (e.g. `titel:`) produces the same parse error as
case (a) regardless of whether the active class is known or unknown.
The dispatcher checks shared keys before consulting any class-specific
key list.

## JSON Schema

`docs/schemas/bib.json` declares
`"$schema": "https://json-schema.org/draft/2020-12/schema"`. Draft
2020-12 provides `unevaluatedProperties`, which is the right primitive
for "shared keys + class-specific keys, both strict, no leakage."

Sketch:

```jsonc
{
  "$ref": "#/$defs/InputReference",
  "$defs": {
    "InputReference": {
      "type": "object",
      "required": ["id", "class"],
      "properties": {
        "id":           { "type": "string" },
        "title":        { "$ref": "#/$defs/Title" },
        "contributors": { "$ref": "#/$defs/ContributorList" },
        "issued":       { "$ref": "#/$defs/EdtfString" },
        // ... other shared fields ...
        "class": {
          "type": "string",
          "enum": [
            "monograph", "collection-component", "serial-component",
            "collection", "serial", "legal-case", "statute", "treaty",
            "hearing", "regulation", "brief", "classic", "patent",
            "dataset", "standard", "software", "event", "audio-visual"
          ]
        }
      },
      "allOf": [
        {
          "if":   { "properties": { "class": { "const": "monograph" } } },
          "then": { "properties": {
            "monograph-type": { "$ref": "#/$defs/MonographType" },
            "volume":         { "type": "integer" },
            "edition":        { "type": "string" }
          }}
        },
        {
          "if":   { "properties": { "class": { "const": "legal-case" } } },
          "then": { "properties": {
            "court":         { "type": "string" },
            "docket-number": { "type": "string" }
          }}
        }
        // ... one branch per known class ...
      ],
      "unevaluatedProperties": false
    }
  }
}
```

Why `unevaluatedProperties: false` is the right primitive here:

- A naive `additionalProperties: false` on the root rejects every
  class-specific key before any `if`/`then` branch can permit it.
- A naive `additionalProperties: false` on the `*Fields` schema
  rejects every shared key inside the branch.
- `unevaluatedProperties: false` (draft 2019-09+) considers a
  property "evaluated" if it is validated by **any** subschema —
  including a `then` branch — and rejects only the leftovers. The
  shared properties are evaluated by the root, the active class's
  properties are evaluated by the matching `then`, and a typo
  ("evaluated by neither") fails validation.

This delivers the same per-class strictness the engine enforces at
parse time, at the JSON Schema layer that hand-authors get via editor
tooling and the hub.

For unknown-class forward-compat: the JSON Schema is a *producer-side*
guardrail and intentionally rejects unknown values of `class:`. The
engine's `SoftDegrade` path is for data that has already been
produced and is now being consumed by an older engine; that
asymmetry is intentional. Producers should bump `style.version` and
adopt the new class deliberately; validators should not silently
sanction unrecognized vocabulary on the producing side.

### Generation via schemars

Default `#[derive(JsonSchema)]` on `InputReference` will **not** emit
the composition shown above. Schemars derives schemas from the
visible struct shape; with a custom `Deserialize` driving a private
`extension` field, schemars has no way to discover the conditional
per-class field sets from the type system. The default derivation
would emit a schema for the shared fields plus an `extension`
property pointing at the `ClassExtension` enum — a shape that does
not match the wire format and would not validate hand-authored YAML.

The implementation provides a hand-written generation function and
applies it via the schemars attribute:

```rust
#[derive(JsonSchema)]
#[schemars(schema_with = "input_reference_schema")]
pub struct InputReference { /* ... */ }

fn input_reference_schema(generator: &mut SchemaGenerator) -> Schema {
    // Emit the JSON Schema sketched above:
    //   - root object with shared properties and `class` as a closed
    //     enum derived from ReferenceClass::variants()
    //   - allOf with one if/then branch per known class, each branch
    //     embedding the schemars-derived schema for the matching
    //     *Fields struct
    //   - unevaluatedProperties: false
}
```

`ReferenceClass`, `ClassExtension`, and each `*Fields` struct derive
`JsonSchema` normally; only the outer `InputReference` needs the
custom function. The function composes the parts the derive macro
already produces — it does not reinvent per-class schemas.

### Alignment invariant

For any input that names a known class, the engine's dispatcher and
the generated JSON Schema **MUST** agree on accept-vs-reject:

- Any input the dispatcher rejects (unknown shared field, unknown
  class-specific field for the active class, class-specific field
  under the wrong class) must also fail JSON Schema validation
  against the generated schema.
- Any input that passes JSON Schema validation must parse
  successfully through the dispatcher.

Inputs with **unknown** `class:` values are treated asymmetrically by
design: rejected by the schema (producer-side guardrail; the `class`
enum is closed) and accepted with `SoftDegrade` by the engine
(consumer-side forward-compat). This is the only intentional
divergence between the two layers.

The implementation PR ships a corpus-driven alignment test
([§Acceptance criteria](#acceptance-criteria), bullet 4).

## Migration sites

The implementation PR touches:

- **`crates/citum-schema-data/src/reference/mod.rs:74`** — replace the
  closed tagged enum with the `InputReference` struct described in
  [§Design](#design) (shared fields + `pub(crate) extension:
  ClassExtension`) plus the hand-written `Deserialize`/`Serialize`,
  the `ReferenceClass` enum, and the typed accessors documented in
  [§Rust API](#rust-api).
- **`crates/citum-schema-data/src/reference/types/legal.rs`**,
  **`crates/citum-schema-data/src/reference/types/specialized.rs`**,
  and **`crates/citum-schema-data/src/reference/types/structural.rs`**
  — each of the 16 structs currently carrying the comment
  `// deny_unknown_fields removed: incompatible with #[serde(tag)] on
  InputReference (serde limitation - tag field is replayed into inner struct)`
  becomes a strict `*Fields` extension. The fields themselves carry
  over unchanged; only the wrapper changes. Restore
  `#[serde(deny_unknown_fields)]` and delete the exemption comment.
- **`crates/citum-engine/src/ffi/mod.rs:108`** and `:120` —
  `serde_yaml::from_str::<Vec<InputReference>>` and
  `IndexMap<String, InputReference>` continue to compile against the
  new shape with no signature change. The C ABI does not change.
- **`crates/citum-io/src/biblatex.rs:25`** — `BibRefContext<'a>` is
  the *biblatex parser-side* context that constructs `InputReference`
  values from a biblatex AST. Constructor sites (`build_inbook_reference`,
  `build_article_reference`, etc.) update to construct the new
  `InputReference` shape with the appropriate `ClassExtension::*`
  variant in the private `extension` field rather than producing the
  legacy newtype variants.
- **`crates/citum-migrate/src/`** — CSL→Citum migrator constructors
  update analogously.
- **`crates/citum-engine/tests/forward_compatibility.rs`** —
  `case_discriminator_class` (row 02b) flips from `HardFail` to
  `SoftDegrade`. The snapshot row in
  `crates/citum-engine/tests/snapshots/forward_compat_gaps.snap`
  updates. Row 07 (`case_new_reference_field`, `csl26-acfh`) also
  closes because the restored `deny_unknown_fields` on `*Fields`
  catches the silent-drop case.
- **Oracle and fixtures.** YAML fixtures keep their hand-authored
  shape: `id`, `class`, shared and class-specific fields at the same
  nesting level. The migrator produces this shape directly; no
  fixture YAML changes for valid references.

## Migration patterns

The follow-up implementation PR refactors every site that matches on
`InputReference`. Three patterns cover the bulk of consumer code.

### Pattern 1 — match on the class

Before:

```rust
match reference {
    InputReference::Monograph(m)       => render_monograph(m),
    InputReference::LegalCase(c)       => render_legal_case(c),
    InputReference::SerialComponent(s) => render_serial_component(s),
    // ... 18 arms ...
}
```

After:

```rust
match reference.extension() {
    ClassExtension::Monograph(m)       => render_monograph(reference, m),
    ClassExtension::LegalCase(c)       => render_legal_case(reference, c),
    ClassExtension::SerialComponent(s) => render_serial_component(reference, s),
    // ... 18 known-class arms ...
    ClassExtension::Unknown(u)         => render_generic(reference, u),
}
```

Render functions take the outer `&InputReference` (for shared fields)
plus the typed extension (for class-specific fields). Consumers reach
the extension through the `extension()` accessor; the private field
name is never visible at the call site.

### Pattern 2 — access a shared field

Before (one of many variations across variants):

```rust
let title = match reference {
    InputReference::Monograph(m)       => m.title.as_ref(),
    InputReference::SerialComponent(s) => s.title.as_ref(),
    // ... repeat for every variant that has a title ...
};
```

After:

```rust
let title = reference.title.as_ref();
```

The shared-field accessor collapses. This is the largest source of
deletion in the implementation PR.

### Pattern 3 — access a class-specific field

Before:

```rust
if let InputReference::Monograph(m) = reference {
    if let Some(et) = &m.monograph_type { ... }
}
```

After (two equivalent forms — direct-typed is shorter for the common
case of a single class):

```rust
// Direct-typed: one accessor per known class.
if let Some(m) = reference.as_monograph() {
    if let Some(et) = &m.monograph_type { ... }
}

// Pattern-match: useful when handling multiple classes in one block.
if let ClassExtension::Monograph(m) = reference.extension() {
    if let Some(et) = &m.monograph_type { ... }
}
```

### Mechanical migration aids

- A short codemod (sed/jq one-liners, or a `syn`-based rewriter) can
  perform Pattern 1 across the crate. The implementation PR ships
  the codemod alongside the schema-data refactor so reviewers can
  audit the bulk transformation as a deterministic rewrite.
- The biblatex parser at `crates/citum-io/src/biblatex.rs` updates
  its `build_*_reference` constructors. The internal
  `BibRefContext<'a>` abstraction stays put; only the constructed
  shape changes.

## Acceptance criteria

This spec moves from Draft to Active when:

1. A child implementation bean is filed under `csl26-1bdr` covering:
   - schema-data refactor to the shape above;
   - migrate path update;
   - biblatex parser update;
   - oracle and fixture regeneration as needed;
   - schema artifact regeneration in `docs/schemas/`;
   - forward-compat rows 02b and 07 snapshot updates;
   - `CompatibilityWarning` plumbing for the unknown-class
     soft-degrade (shape owned by [`FORWARD_COMPATIBILITY.md`](./FORWARD_COMPATIBILITY.md));
   - the eight test cases enumerated in this spec's §"Error UX" and
     in the spike's test suite, translated into citum-schema-data
     unit tests.
2. The schema bump implied by the implementation is recorded in
   [`../reference/SCHEMA_VERSIONING.md`](../reference/SCHEMA_VERSIONING.md)
   operational history (pre-1.0 cap: minor).
3. The implementation PR is merged and the snapshot reflects the new
   outcomes.
4. The implementation PR ships **schema-alignment tests**: a corpus
   of valid and invalid YAML fixtures (at minimum one per error case
   in [§Error UX](#error-ux)) is validated against the generated JSON
   Schema *and* parsed through the dispatcher; for every known-class
   case, the two layers must agree on accept-vs-reject. The single
   intentional divergence (unknown `class:` → schema rejects, engine
   `SoftDegrade`s) is asserted explicitly.

## Worked example

A reference for a `dance-performance` class that engine 0.51 does
not know. Engine 0.52 has added the typed class.

Hand-authored YAML:

```yaml
- id: perf2026
  class: dance-performance
  title: Pina
  contributors:
    director: [{ name: Wim Wenders }]
  issued: "2011"
  venue: Berlin
```

Under engine 0.51 (does not know the class):

- `reference.id == "perf2026"`, `reference.title == Some("Pina")`,
  `reference.contributors == Some(...)`, `reference.issued ==
  Some("2011")` — shared fields populate normally.
- `reference.class()` returns `ReferenceClass::Unknown("dance-performance".into())`.
- `reference.extension()` returns `ClassExtension::Unknown(UnknownClassData
  { class: "dance-performance", fields: { "venue": "Berlin" } })`.
- The dispatcher recognized `id`, `title`, `contributors`, `issued`
  as shared (strictly validated) and captured `venue` verbatim in
  `UnknownClassData::fields` because the class is unknown.
- A typo on `titel:` here would still fail at parse time — the
  shared-field strictness is independent of class knowledge.
- A `CompatibilityWarning` is emitted; the engine renders the shared
  fields via the generic path and produces degraded but readable
  output.

Under engine 0.52 (knows the class):

- `reference.class()` returns `ReferenceClass::DancePerformance`.
- `reference.as_dance_performance()` returns
  `Some(&DancePerformanceFields { venue: Some("Berlin"), ... })`.
- `reference.extension()` returns
  `&ClassExtension::DancePerformance(...)`.
- All fields typed; `deny_unknown_fields` on `DancePerformanceFields`
  catches a typo on `venuw:`.
- Full rendering.

The producer's authoring effort is identical in both cases — same
flat YAML, no wrapper key (see [§Wire format](#wire-format)).

## Non-goals

- No commitment to a specific `CompatibilityWarning` shape — that
  contract lives in [`FORWARD_COMPATIBILITY.md`](./FORWARD_COMPATIBILITY.md).
- No reshaping of `Style`; this spec's scope is `InputReference`
  only.
- No commitment to a JSON Schema version number — the release
  workflow infers from conventional commits.
- No commitment to a derive-macro for the dispatcher. The
  implementation PR may hand-write each `*Fields` key list or
  generate it via a macro; either is acceptable so long as the key
  lists stay in sync with the struct definitions.
