# CSL 1.0.2 Type Conversion Contract

**Status:** Active
**Version:** 1.0
**Date:** 2026-07-02
**Supersedes:** None
**Related:** bean `csl26-cvfy`, bean `csl26-shco`, bean `csl26-1bdr`, [`INPUT_REFERENCE_CLASS_DISCRIMINATOR.md`](./INPUT_REFERENCE_CLASS_DISCRIMINATOR.md)

## Purpose

Legacy CSL-JSON references convert to `InputReference` through two
unvalidated hops: a `note:`-field `type: X` override
(`csl_legacy::csl_json::Reference::parse_note_field_hacks`) that
unconditionally rewrites the top-level `type`, and a hand-maintained
routing `match` (`citum_schema_data::reference::conversion`) that silently
falls through to a generic document/monograph shape for any CSL type
string it doesn't recognize. Neither hop is checked against the actual
CSL 1.0.2 type vocabulary, so a real CSL type with no routing arm — or a
typo in a `note:` override — reclassifies a reference with no error, no
warning, and no test failure. The only way the defect surfaces is a wrong
citation downstream, several pipeline layers away from the actual cause
(see the root-cause investigation logged in bean `csl26-cvfy`, root-caused
to a `chi-manuscript` fixture case in the `chicago-shared-corpus` suite,
tracked separately as bean `csl26-shco`).

This spec establishes the CSL 1.0.2 vocabulary as a source of truth, adds
a contract test that asserts every type in that vocabulary produces a
faithful (non-fallback) `ref_type()` round trip, closes every routing gap
the contract test found, and validates the `note:`-field override against
the same vocabulary.

## Scope

In scope:
- The canonical CSL 1.0.2 item-type vocabulary (`CSL_TYPES`) and the
  non-CSL-1.0.2 spellings the conversion layer already accepts
  (`CSL_TYPE_EXTENSIONS`), both in `crates/csl-legacy/src/csl_json.rs`.
- Closing every CSL 1.0.2 routing gap in
  `citum_schema_data::reference::conversion` so each type produces a
  `ref_type()` round trip that is not a silent collapse into the generic
  document/monograph fallback.
- Making the routing fallback loud in debug builds when it receives a
  known CSL 1.0.2 type (mirrors the existing `ClassExtension::Unknown`
  pattern in `accessors.rs`).
- Validating the `note:`-field `type: X` override against the vocabulary
  before applying it.
- The canonicalization/equivalence table: which `ref_type()` output counts
  as a faithful round trip for each CSL 1.0.2 input type, including the
  intentional divergences.

Out of scope (see [Non-goals](#non-goals)):
- `CompatibilityWarning` plumbing for the loud-fail path (owned by bean
  `csl26-1bdr` / `FORWARD_COMPATIBILITY.md`).
- Attributed fidelity reporting in `oracle.js`/`report-core.js` (Phase 5,
  tracked as a follow-on bean).
- Driving `chicago-shared-corpus` or any other style to 100% fidelity
  (bean `csl26-shco`'s scope, not this one).
- Any change to `InputReference`'s wire shape or the class discriminator
  design (`csl26-1bdr`'s settled scope).

## Design

### CSL 1.0.2 vocabulary as source of truth

`crates/csl-legacy/src/csl_json.rs` defines two public constants:

```rust
pub const CSL_TYPES: &[&str] = &[ /* 45 entries */ ];
pub const CSL_TYPE_EXTENSIONS: &[&str] = &[ /* 5 entries */ ];
```

`CSL_TYPES` is sourced from `schemas/styles/csl-types.rnc` at the tagged
`v1.0.2` release of the [CSL schema
repository](https://github.com/citation-style-language/schema/blob/v1.0.2/schemas/styles/csl-types.rnc)
(verified identical against `master` at the time of writing), and contains
exactly these 45 strings:

```text
article, article-journal, article-magazine, article-newspaper, bill,
book, broadcast, chapter, classic, collection, dataset, document, entry,
entry-dictionary, entry-encyclopedia, event, figure, graphic, hearing,
interview, legal_case, legislation, manuscript, map, motion_picture,
musical_score, pamphlet, paper-conference, patent, performance,
periodical, personal_communication, post, post-weblog, regulation,
report, review, review-book, software, song, speech, standard, thesis,
treaty, webpage
```

**`regulation` is confirmed CSL 1.0.2, not CSL-M.** The `v1.0.2`-tagged
schema file was fetched and diffed against `master`; both contain
`regulation` in the type list. It stays in `CSL_TYPES`, not a CSL-M
extensions list.

`CSL_TYPE_EXTENSIONS` audits the routing table for strings the conversion
layer has historically accepted that are **not** part of CSL 1.0.2:

| Extension string | Why it's accepted |
|---|---|
| `manual` | Zotero/BibLaTeX "manual" export; routed like a technical manual (`MonographType::Manual`). |
| `presentation` | Historical alias accepted alongside `speech`/`event`; routed to the event converter. |
| `personal-communication` | Hyphenated respelling of the CSL 1.0.2 `personal_communication` type. |
| `legal-case` | Hyphenated respelling of the CSL 1.0.2 `legal_case` type. |
| `statute` | Not part of CSL 1.0.2 (`legislation` is the closed-vocabulary equivalent); routed to the same converter and shares its canonical output. |

Both constants are consumed by:
1. The conversion-layer contract test
   (`citum_schema_data::reference::conversion::contract_tests`), which
   iterates `CSL_TYPES` and asserts a faithful round trip.
2. The routing fallback's `debug_assert!` (see [Loud-fail
   design](#loud-fail-design-for-the-routing-fallback)).
3. `Reference::parse_note_field_hacks`, which validates `note:`-field type
   overrides against `CSL_TYPES ∪ CSL_TYPE_EXTENSIONS` (see [Note-field
   override policy](#note-field-override-policy)).

### Canonicalization / equivalence table

`ref_type()` is not required to echo the input string verbatim — several
CSL 1.0.2 types have a **documented, intentional** canonical output that
differs from the input spelling. The contract test's `EXPECTATIONS` table
in `conversion/contract_tests.rs` encodes exactly these mappings; this
table is the authoritative list.

| CSL 1.0.2 input | Canonical `ref_type()` output | Rationale |
|---|---|---|
| `legal_case` | `legal-case` | Codebase-wide convention: underscore CSL spellings canonicalize to hyphens on output (pre-existing, not introduced by this spec). |
| `motion_picture` | `motion-picture` | Same convention. |
| `musical_score` | `musical-score` | Same convention (new arm added by this spec). |
| `personal_communication` | `personal-communication` | Same convention (pre-existing). |
| `legislation` | `statute` | `legislation` is the CSL 1.0.2 closed-vocabulary type; it routes to the same converter as the `statute` extension spelling and shares its canonical output — there is no separate `Legislation` class. |
| `article` (no `container-title`) | `preprint` | A container-less `article` has no journal context; the converter treats it as a standalone preprint (arXiv/SSRN-style), which matches how such exports actually occur in the wild. See `from_preprint_ref`. |
| `bill` (minimal shape: no `authority`, `chapter-number`, or `container-title`+`volume`+`page`) | `document` | `from_bill_ref` is deliberately polymorphic: `title`+`authority` → hearing; titleless+`authority`/`chapter-number` → `bill-proceeding` genre; titleless+`container-title`+`volume`+`page` → `bill-record` genre. A minimal bill with none of those signals is indistinguishable from a generic government document, and real CSL-JSON `bill` exports always carry one of the distinguishing shapes (see `reference/tests.rs`'s `test_parse_csl_bill_*` tests for the shapes that *do* round-trip distinctly). |

All other CSL 1.0.2 types round-trip to the identical string.

### Routing closure

Before this spec, `citum_schema_data::reference::conversion::mod`'s
top-level `match legacy.ref_type.as_str()` had no arm — direct or via
genre-seeding — for: `collection`, `entry`, `figure`, `graphic`,
`musical_score`, `pamphlet`, `performance`, `periodical`, `review`,
`review-book`. Three further types had an arm but did not round-trip
correctly under the contract test's minimal-fixture shape: `map` and
`document` reached the correct constructor only via the (soon-to-be-loud)
wildcard arm, and `speech` reached `from_event_ref` but `event_ref_type`
had no way to recover `"speech"` from an empty `genre` field (a real gap
found while writing the contract test, not anticipated at bean-filing
time). A twelfth gap, `post-weblog` silently collapsing to `"post"`, was
also found this way — `post-weblog` had a working routing arm but shared
`MonographType::Post` with plain `post` and nothing distinguished them on
the way back out.

Each gap closes with the same shape: route to an existing constructor,
and seed `genre` from `ref_type` only when the legacy reference didn't
already supply one — this is the established pattern
(`from_document_ref`'s pre-existing `"map"` handling) generalized to the
other genre-discriminated types:

| CSL type | Route | Round-trip mechanism |
|---|---|---|
| `collection` | `from_document_ref` (see the archival-semantics note below) | Genre seeded from `ref_type`; `monograph_ref_type`'s `Document` arm gains `collection`. |
| `document` | Explicit arm → `from_document_ref` (previously only reachable via the wildcard) | Already `"document"`. |
| `entry` | `from_collection_component_ref` | Genre seeded `"entry"`; `collection_component_ref_type` gains an `entry` arm. |
| `figure`, `graphic` | `from_document_ref` | Genre seeded from `ref_type`; `monograph_ref_type`'s `Document` arm gains `figure`/`graphic`. |
| `map` | Explicit arm → `from_document_ref` (genre logic pre-existing) | Unchanged behavior, now reachable without falling through the loud wildcard. |
| `musical_score`, `pamphlet` | `from_monograph_ref` (stays `MonographType::Book`) | Genre seeded from `ref_type`; `monograph_ref_type`'s `Book` arm checks genre first (`musical_score` → `"musical-score"`, `pamphlet` → `"pamphlet"`). |
| `performance` | `from_event_ref` | Genre seeded from `ref_type`; `event_ref_type` gains a `contains("performance")` arm. |
| `periodical` | `from_document_ref` (judgment call, see below) | Genre seeded from `ref_type`; `monograph_ref_type`'s `Document` arm gains `periodical`. |
| `review`, `review-book` | `from_serial_component_ref` | Genre seeded from `ref_type`; `serial_component_ref_type` checks genre before falling back to container-type inference. |
| `speech` | Unchanged route (`from_event_ref`) | Genre seeded from `ref_type` when absent — this is the fix; the routing arm already existed. |
| `post-weblog` | Unchanged route (`from_monograph_ref`) | Genre seeded `"post-weblog"`; `monograph_ref_type`'s `Post` arm checks genre before defaulting to `"post"`. |

**Genre-seeding never overwrites a user-supplied genre.** Every seeding
site uses `legacy.genre.clone().or_else(|| matches!(...).then(|| ...))` —
if the legacy reference already carries an explicit `genre`, that value
wins and the CSL-type-derived seed is not applied. This means a
`musical_score` (or any seeded type) with an explicit, unrelated `genre`
value degrades gracefully: it round-trips through whatever `ref_type()`
its actual genre implies, not through the seeded value. This is an
accepted, pre-existing trade-off (the same trade-off `from_document_ref`'s
original `"map"` handling already made) — the contract test only exercises
the minimal shape, so this edge case is not separately asserted, but it is
the same shape of divergence already accepted for `map`, `bill-proceeding`,
and `bill-record`.

**`collection` routing decision (archival, not editorial).** CSL 1.0.2's
`collection` denotes an **archival collection** — a body of manuscripts or
papers held by an archive, carrying an author/creator, an `archive`, an
`archive-place`, and often an `archive_collection` note override (this is
exactly the shape of the Chicago 18th fixtures that motivated this spec:
"Egmont Manuscripts, Phillipps Collection, University of Georgia
Library"). Citum's `ClassExtension::Collection` models the *editorial*
collection (anthology, proceedings, edited volume): it has `editor` but
**no `author` field and no archive fields**, so routing CSL `collection`
into it silently drops the author and the entire archival location — the
very data these references exist to cite. (`ref_type()` mapping
`ClassExtension::Collection` → `"collection"` predates this spec and is a
lossy legacy-output concession, not evidence the two concepts coincide.)
`collection` therefore routes through `from_document_ref` with a
genre-discriminated round trip, the same as the other archival/document
shapes, preserving `author`, `archive`, `archive_location`, and
`archive_info`. If archival collections ever warrant first-class
modeling, that is a schema addition (author + archive fields on a
dedicated class), which this spec's non-goals exclude.

**`periodical` routing decision.** `periodical` describes a standalone
reference to a serial publication itself (the publication, not an
article within it) — semantically closer to a generic bibliographic
document than to a `Serial` container, which in this codebase exists only
as an embedded parent for `SerialComponent`/`Collection`-style relations,
never as a freestanding top-level reference type. Building a dedicated
standalone-`Serial` constructor for a rarely-used CSL type would add a new
shape with no other caller, for a case the genre-tagged `Document`
pathway already serves adequately. `from_document_ref` + genre was
chosen for the same reason `figure`/`graphic`/`map` were: minimal new
surface area, consistent with the established genre-discrimination
pattern.

**Underscore/hyphen spellings.** New arms accept the CSL 1.0.2 spelling
only (`musical_score`, not `musical-score`) — this mirrors the
instruction to "accept the CSL spelling" rather than proactively
inventing new hyphen aliases; `legal_case`/`legal-case` and
`personal_communication`/`personal-communication` accepting both
spellings is pre-existing behavior this spec did not extend to the new
types.

### Loud-fail design for the routing fallback

The routing `match`'s final wildcard arm still falls through to
`from_document_ref` (unconditional fallback is required — an
unrecognized `type:` string, e.g. a producer-specific extension this
codebase has never seen, must still parse into *something* usable), but
it is now preceded by a `debug_assert!` that fires if the fallen-through
type is a **known** CSL 1.0.2 type:

```rust
debug_assert!(
    !csl_legacy::csl_json::CSL_TYPES.contains(&legacy.ref_type.as_str()),
    "unmapped CSL 1.0.2 type `{}` fell through to the document fallback; \
     add a routing arm in conversion/mod.rs",
    legacy.ref_type
);
```

This mirrors the `ClassExtension::Unknown` arm of `InputReference::ref_type()`
(`accessors.rs`, `TODO(csl26-1bdr)`) exactly in spirit: a `debug_assert!`
plus a `TODO(csl26-1bdr)` comment gesturing at the same future
`CompatibilityWarning` plumbing that bean owns. No new warning
infrastructure is built here — this spec deliberately reuses the existing
pattern rather than diverging into a second ad hoc mechanism.
`debug_assert!` was chosen over a hard `panic!`/`Result` because
`InputReference::from` is an infallible `From` impl consumed throughout
the codebase (CLI, migrate, engine); changing its signature to fallible is
out of scope, and a silent-in-release/loud-in-debug assertion is the
same trade-off the existing `Unknown`-class pattern already made.

### Note-field override policy

`Reference::parse_note_field_hacks` applies a `note:`-field `type: X`
line only when `X` is in `CSL_TYPES ∪ CSL_TYPE_EXTENSIONS`:

```rust
if key.eq_ignore_ascii_case("type") {
    if CSL_TYPES.contains(&value) || CSL_TYPE_EXTENSIONS.contains(&value) {
        self.ref_type = value.to_string();
        parsed_indices.insert(idx);
    }
    // else: leave ref_type and the note line untouched.
}
```

**Policy: ignore and keep, not apply-and-warn.** An unrecognized override
(a typo, e.g. `type: colection`, or a future vocabulary term this release
doesn't know) does **not** change `ref_type`, and the `note:` line is
**not** consumed — it stays in the rebuilt `note` field rather than being
silently swallowed. This was an explicit product decision (locked before
implementation): the alternative — apply the override anyway and flag a
`CompatibilityWarning` — was rejected because it still risks routing a
reference through a class-specific converter that has never seen that
`ref_type` string, hitting the *same* generic-fallback problem this spec
exists to close, just one layer earlier. Ignoring preserves the top-level
`type`, which is guaranteed to be routable (or itself loudly asserted, see
above), and leaves a human-readable trace (the unconsumed note line) for
whoever investigates the data quality issue.

This is also what makes the `chi-manuscript` regression fixable: the
reference's note carries `type: collection`, and `collection` is a real
CSL 1.0.2 type (already in `CSL_TYPES`), so the override is recognized
and applied — the reference correctly becomes a `collection`, not a
`manuscript` with a discarded override. The bug was never in the
override-validation policy; it was the missing `collection` routing arm
(see [Routing closure](#routing-closure)).

## Non-goals

- **`CompatibilityWarning` plumbing.** The loud-fail `debug_assert!` in
  the routing fallback is deliberately the same stopgap the
  `ClassExtension::Unknown` arm already uses. Wiring both into a real
  warning channel is bean `csl26-1bdr`'s Layer 5 scope, tracked by the
  `TODO(csl26-1bdr)` comments this spec adds.
- **Attributed fidelity reporting (Phase 5).** Exposing per-reference
  conversion diagnostics from `citum render refs --json` (e.g., "rendered
  via fallback/default type") so `scripts/oracle.js` and
  `scripts/report-core.js` can tag a failing fixture case as
  "conversion-layer suspect" before it's counted as a style-fidelity
  failure is out of scope for this spec. Tracked as a follow-on task
  under bean `csl26-cvfy` (see the epic's todo list) for a separate PR.
- **Driving any specific style to 100% fidelity.** `chicago-shared-corpus`
  and any other style's remaining fixture failures are bean `csl26-shco`'s
  scope. This spec closes the conversion-layer defect the `chi-manuscript`
  case exposed; it does not audit or fix other failures in that corpus.
- **`InputReference` wire-shape changes.** No change to the class
  discriminator design settled in
  [`INPUT_REFERENCE_CLASS_DISCRIMINATOR.md`](./INPUT_REFERENCE_CLASS_DISCRIMINATOR.md).

## Acceptance Criteria

- [x] `CSL_TYPES` and `CSL_TYPE_EXTENSIONS` defined in
      `crates/csl-legacy/src/csl_json.rs` with provenance documented in
      `///` comments.
- [x] `conversion::contract_tests::every_csl_1_0_2_type_round_trips_through_ref_type`
      asserts every `CSL_TYPES` entry against the canonicalization table
      above, with zero allowlisted exceptions.
- [x] `conversion::contract_tests::manuscript_with_recognized_collection_note_override_converts_to_collection`
      reproduces and fixes the `chi-manuscript` regression.
- [x] Every routing gap in the table above has an explicit arm; the
      wildcard fallback carries the `debug_assert!` loud-fail.
- [ ] `Reference::parse_note_field_hacks` validates `type:` overrides
      against `CSL_TYPES ∪ CSL_TYPE_EXTENSIONS`; unrecognized overrides
      are ignored and the note line is preserved (tests in
      `crates/csl-legacy/src/csl_json.rs` and
      `crates/citum-schema-data/src/reference/tests.rs`).
- [ ] Phase 5 (attributed fidelity reporting) — tracked as a follow-on
      bean, not required for this spec's Active status.
- [ ] `node scripts/report-core.js --style chicago-notes-18th` re-run to
      confirm `chi-manuscript` improves — tracked in bean `csl26-cvfy`'s
      todo list, not part of this implementation PR.

## Changelog

- v1.0 (2026-07-02): Initial version; spec flips to Active in the same
  commit that lands the routing closure.
