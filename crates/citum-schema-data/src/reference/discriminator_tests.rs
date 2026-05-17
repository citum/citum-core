/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Behaviour tests for the public class discriminator.
//!
//! Extracted from `mod.rs` (was `mod discriminator_tests { ... }`).

use super::{ClassExtension, InputReference, ReferenceClass};
use serde_json::{Value as JsonValue, json};

fn parse_reference(json: &str) -> Result<InputReference, serde_json::Error> {
    serde_json::from_str(json)
}

#[test]
fn public_discriminator_parses_representative_known_classes() {
    // given a representative input for each known top-level class,
    // when parsed via the flat-with-discriminator deserializer,
    // then the `class()` accessor must return the matching typed variant
    // (not Unknown, not a different known class).
    let cases: &[(&str, ReferenceClass)] = &[
        (
            r#"{ "class": "monograph", "type": "book", "title": "B", "issued": "2024" }"#,
            ReferenceClass::Monograph,
        ),
        (
            r#"{
                    "class": "legal-case",
                    "title": "Smith v. Jones",
                    "authority": "Supreme Court",
                    "issued": "2024"
                }"#,
            ReferenceClass::LegalCase,
        ),
        (
            r#"{ "class": "audio-visual", "type": "film", "title": "F", "issued": "2024" }"#,
            ReferenceClass::AudioVisual,
        ),
    ];

    for (json, expected_class) in cases {
        let reference = parse_reference(json).unwrap_or_else(|err| {
            panic!("expected `{expected_class:?}` to parse, got error: {err}\nJSON: {json}")
        });
        assert_eq!(
            &reference.class(),
            expected_class,
            "class() must equal the expected variant for JSON: {json}"
        );
    }
}

#[test]
fn public_discriminator_captures_unknown_field_on_known_class() {
    // when an unknown-for-this-class field is present on a known class,
    // forward-compat pattern silently captures it in unknown_fields
    // (SoftDegrade behavior per FORWARD_COMPATIBILITY.md row 07).
    let result = parse_reference(
        r#"{
                "class": "legal-case",
                "title": "Smith v. Jones",
                "monograph-type": "book"
            }"#,
    );

    assert!(
        result.is_ok(),
        "unknown field should be captured, not rejected"
    );
    let ref_obj = result.unwrap();
    if let ClassExtension::LegalCase(lc) = ref_obj.extension {
        assert!(
            lc.unknown_fields.contains_key("monograph-type"),
            "unknown field must be captured in unknown_fields map"
        );
    } else {
        panic!("expected LegalCase variant");
    }
}

#[test]
fn public_discriminator_captures_unknown_class_fields() {
    // when an unknown class string is encountered,
    // then the dispatcher must capture the class verbatim,
    // preserve non-shared fields under `UnknownClassData::fields`,
    // expose the shared `id` through the accessor (proves shared-fields
    // extraction works on the unknown path too), and surface the raw class
    // string via `ref_type()` per the documented soft-degrade contract.
    let reference = parse_reference(
        r#"{
                "class": "dance-performance",
                "id": "pina2011",
                "title": "Pina",
                "venue": "Berlin",
                "duration-minutes": 103
            }"#,
    )
    .unwrap();

    assert_eq!(
        reference.class(),
        ReferenceClass::Unknown("dance-performance".into())
    );

    let unknown = reference.unknown_class().unwrap();
    assert_eq!(unknown.class, "dance-performance");
    assert_eq!(
        unknown.fields.get("venue").and_then(JsonValue::as_str),
        Some("Berlin"),
        "captured non-shared field must be exactly the wire value"
    );
    assert_eq!(
        unknown
            .fields
            .get("duration-minutes")
            .and_then(JsonValue::as_u64),
        Some(103),
        "non-shared numeric field must preserve its JSON number type"
    );
    assert_eq!(reference.id().unwrap().as_str(), "pina2011");
    match reference.title().unwrap() {
        super::Title::Single(s) => assert_eq!(
            s, "Pina",
            "shared title must be the wire value for unknown-class refs"
        ),
        other => panic!("expected Title::Single, got {other:?}"),
    }
    assert_eq!(
        reference.ref_type(),
        "dance-performance",
        "ref_type for unknown class must return the raw class string (Layer-5 will replace)"
    );
    assert!(
        !ReferenceClass::KNOWN.contains(&reference.ref_type().as_str()),
        "ref_type sentinel for unknown class must not collide with any known class string"
    );
}

#[test]
fn public_discriminator_round_trips_flat_unknown_class() {
    // given an unknown-class reference parsed from flat JSON,
    // when re-serialized, the output must be structurally flat
    // (no UnknownClassData wrapper, no `fields` key, top-level discriminator),
    // and a second parse must yield a value equal to the first.
    let reference = parse_reference(
        r#"{
                "class": "dance-performance",
                "id": "pina2011",
                "title": "Pina",
                "venue": "Berlin"
            }"#,
    )
    .unwrap();

    let serialized: JsonValue = serde_json::to_value(&reference).unwrap();
    let serialized_obj = serialized
        .as_object()
        .expect("must serialize as a JSON object");

    assert_eq!(
        serialized_obj.get("class").and_then(JsonValue::as_str),
        Some("dance-performance"),
        "discriminator must round-trip at the top level"
    );
    assert_eq!(
        serialized_obj.get("venue").and_then(JsonValue::as_str),
        Some("Berlin"),
        "non-shared field must round-trip at the top level (flat structure)"
    );
    assert!(
        !serialized_obj.contains_key("fields"),
        "must not leak the internal UnknownClassData `fields` key, got: {serialized}"
    );

    let round_tripped: InputReference = serde_json::from_value(serialized).unwrap();
    assert_eq!(round_tripped, reference);
}

// ──────────────────────────────────────────────────────────────────────
// New tests covering the post-Copilot-review hardening paths.
// ──────────────────────────────────────────────────────────────────────

#[test]
fn duplicate_class_field_is_rejected_with_canonical_serde_shape() {
    // when the `class` discriminator appears twice on the wire,
    // then the dispatcher must reject it with the canonical
    // `duplicate field \`class\`` shape (matches serde's
    // `de::Error::duplicate_field` output for compatibility).
    let err = parse_reference(
        r#"{
                "class": "monograph",
                "class": "legal-case",
                "title": "X",
                "issued": "2024"
            }"#,
    )
    .unwrap_err()
    .to_string();

    assert!(
        err.contains("duplicate field `class`"),
        "must produce serde-canonical duplicate-field message, got: {err}"
    );
}

#[test]
fn duplicate_non_class_field_is_rejected_with_canonical_serde_shape() {
    // the non-class path was previously inconsistent (used `custom` with
    // a free-form string while the `class` path used `duplicate_field`).
    // both paths must now produce the identical canonical shape.
    let err = parse_reference(
        r#"{
                "class": "monograph",
                "title": "First",
                "title": "Second",
                "issued": "2024"
            }"#,
    )
    .unwrap_err()
    .to_string();

    assert!(
        err.contains("duplicate field `title`"),
        "non-class duplicate must mirror the serde-canonical shape, got: {err}"
    );
}

#[test]
fn missing_class_field_is_rejected() {
    let err = parse_reference(r#"{ "title": "Untyped", "issued": "2024" }"#)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("missing field `class`"),
        "absence of the discriminator must produce a canonical missing-field error, got: {err}"
    );
}

#[test]
fn non_object_body_is_rejected_with_schema_error_not_io_error() {
    // covers the `from_known` defensive branch: prior implementation
    // surfaced this as an IO-wrapped error, which was misleading for
    // schema bugs. The current path uses `de::Error::custom` so the
    // surfaced message must be a plain schema error.
    let err = serde_json::from_value::<InputReference>(json!(["not", "an", "object"]))
        .unwrap_err()
        .to_string();
    // The visitor `expecting` message describes a flat reference object;
    // serde's invalid-type machinery threads that through. Either the
    // visitor's `expecting` text or our defensive message is acceptable.
    assert!(
        err.contains("flat reference object")
            || err.contains("reference body must be a JSON object")
            || err.contains("invalid type"),
        "must produce a schema-shaped error, not an IO error, got: {err}"
    );
    assert!(
        !err.contains("InvalidData"),
        "must not leak the io::ErrorKind::InvalidData shape, got: {err}"
    );
}

#[test]
fn unknown_class_ref_type_does_not_collide_with_known_classes() {
    // regression guard for the Layer-5 soft-degrade contract: ref_type()
    // for an unknown class must never accidentally route to a known
    // CSL type-template branch.
    for unknown in [
        "dance-performance",
        "happening",
        "frobnicate",
        "x-future-class",
    ] {
        let json =
            format!(r#"{{ "class": "{unknown}", "id": "a", "title": "T", "issued": "2024" }}"#);
        let reference = parse_reference(&json).unwrap();
        assert!(
            matches!(reference.class(), ReferenceClass::Unknown(ref s) if s == unknown),
            "{unknown} must classify as Unknown",
        );
        assert!(
            !ReferenceClass::KNOWN.contains(&reference.ref_type().as_str()),
            "{unknown}: ref_type() returned {:?}, which is a KNOWN class — would mis-route at render",
            reference.ref_type()
        );
    }
}

#[test]
fn accessor_and_extension_class_agree_for_every_known_variant() {
    // forward-compat guard: every known class must parse such that
    // `class()` agrees with the resident `ClassExtension` variant.
    // Catches drift if a future change to the dispatcher routes a class
    // string into the wrong extension box.
    let cases: &[(&str, ReferenceClass, fn(&ClassExtension) -> bool)] = &[
        (
            r#"{ "class": "monograph", "type": "book", "title": "B", "issued": "2024" }"#,
            ReferenceClass::Monograph,
            |e| matches!(e, ClassExtension::Monograph(_)),
        ),
        (
            r#"{ "class": "legal-case", "title": "S v J", "authority": "SC", "issued": "2024" }"#,
            ReferenceClass::LegalCase,
            |e| matches!(e, ClassExtension::LegalCase(_)),
        ),
        (
            r#"{ "class": "audio-visual", "type": "film", "title": "F", "issued": "2024" }"#,
            ReferenceClass::AudioVisual,
            |e| matches!(e, ClassExtension::AudioVisual(_)),
        ),
        (
            r#"{ "class": "patent", "title": "P", "patent-number": "US123", "issued": "2024" }"#,
            ReferenceClass::Patent,
            |e| matches!(e, ClassExtension::Patent(_)),
        ),
        (
            r#"{ "class": "dataset", "title": "D", "issued": "2024" }"#,
            ReferenceClass::Dataset,
            |e| matches!(e, ClassExtension::Dataset(_)),
        ),
        (
            r#"{ "class": "software", "title": "S", "issued": "2024" }"#,
            ReferenceClass::Software,
            |e| matches!(e, ClassExtension::Software(_)),
        ),
    ];

    for (json, expected_class, extension_matches) in cases {
        let reference = parse_reference(json).expect(json);
        assert_eq!(
            &reference.class(),
            expected_class,
            "class() drift on {json}"
        );
        assert!(
            extension_matches(&reference.extension),
            "ClassExtension variant drift on {json}: class()={:?}",
            reference.class()
        );
    }
}

#[test]
fn set_id_updates_the_class_specific_extension_for_known_class() {
    // `set_id` writes only into the class-specific extension (the
    // duplicated top-level shared fields have been removed). Verify
    // both the public accessor and direct extension inspection agree.
    let mut reference = parse_reference(
        r#"{ "class": "monograph", "type": "book", "title": "B", "id": "orig", "issued": "2024" }"#,
    )
    .unwrap();

    reference.set_id(super::RefID::from("updated"));

    assert_eq!(reference.id().unwrap().as_str(), "updated");
    match &reference.extension {
        ClassExtension::Monograph(m) => assert_eq!(
            m.id.as_ref().map(|r| r.as_str()),
            Some("updated"),
            "set_id must update the class-specific extension copy"
        ),
        other => panic!("expected Monograph extension, got {other:?}"),
    }
}

#[test]
fn serialize_emits_flat_object_with_class_first_and_no_nesting() {
    // regression guard for the flatten-proxy serialize path. The wire
    // shape must be a flat object with `class` as a sibling of the
    // typed fields — never nested as `{ "monograph": {...} }` and
    // never reordered into the inner. Catches a future accidental
    // change away from `#[serde(flatten)]` on FlatClassProxy.
    let reference = parse_reference(
            r#"{ "class": "monograph", "type": "book", "title": "Pina", "id": "pina2011", "issued": "2024" }"#,
        )
        .unwrap();

    let value = serde_json::to_value(&reference).unwrap();
    let obj = value
        .as_object()
        .expect("InputReference must serialize to a top-level JSON object");

    assert_eq!(
        obj.get("class").and_then(JsonValue::as_str),
        Some("monograph"),
        "class discriminator must sit at the top level"
    );
    assert_eq!(
        obj.get("type").and_then(JsonValue::as_str),
        Some("book"),
        "typed fields must be flattened to the top level, not nested"
    );
    assert_eq!(
        obj.get("id").and_then(JsonValue::as_str),
        Some("pina2011"),
        "shared `id` must be flattened from the extension to the top level"
    );
    assert!(
        !obj.contains_key("monograph"),
        "must not nest the inner struct under a class-named key, got: {value}"
    );
    assert!(
        !obj.contains_key("extension"),
        "must not leak the internal `extension` field name, got: {value}"
    );
}

#[test]
fn round_trip_through_serde_value_preserves_every_known_class() {
    // belt-and-suspenders: serialize → from_value → equality. Exercises
    // the proxy serialize path for every known class via the existing
    // accessor_and_extension fixture set, plus Unknown.
    let cases = [
        r#"{ "class": "monograph", "type": "book", "title": "B", "issued": "2024" }"#,
        r#"{ "class": "legal-case", "title": "S v J", "authority": "SC", "issued": "2024" }"#,
        r#"{ "class": "audio-visual", "type": "film", "title": "F", "issued": "2024" }"#,
        r#"{ "class": "patent", "title": "P", "patent-number": "US123", "issued": "2024" }"#,
        r#"{ "class": "dataset", "title": "D", "issued": "2024" }"#,
        r#"{ "class": "software", "title": "S", "issued": "2024" }"#,
        r#"{ "class": "dance-performance", "id": "p", "title": "P", "venue": "B" }"#,
    ];
    for json in cases {
        let reference = parse_reference(json).expect(json);
        let value = serde_json::to_value(&reference).unwrap();
        let parsed: InputReference = serde_json::from_value(value).expect(json);
        assert_eq!(reference, parsed, "round-trip drift on: {json}");
    }
}

#[test]
fn set_id_keeps_unknown_class_fields_in_sync() {
    // unknown-class refs store id as a JsonValue::String in
    // UnknownClassData::fields; verify the documented behavior holds
    // and round-trips through the public `id()` accessor.
    let mut reference =
        parse_reference(r#"{ "class": "dance-performance", "id": "orig", "title": "P" }"#).unwrap();

    reference.set_id(super::RefID::from("updated"));

    assert_eq!(reference.id().unwrap().as_str(), "updated");
    let unknown = reference.unknown_class().unwrap();
    assert_eq!(
        unknown.fields.get("id").and_then(JsonValue::as_str),
        Some("updated"),
        "unknown-class set_id must update fields[\"id\"] as a JSON string"
    );
}

#[cfg(feature = "schema")]
#[test]
fn public_discriminator_schema_contains_class_branches_and_strict_root() {
    let schema = serde_json::to_value(schemars::schema_for!(InputReference)).unwrap();
    let schema_text = serde_json::to_string(&schema).unwrap();

    assert!(schema_text.contains("\"unevaluatedProperties\":false"));
    for class in ReferenceClass::KNOWN {
        assert!(
            schema_text.contains(&format!("\"const\":\"{class}\"")),
            "schema must contain a class branch for `{class}`"
        );
    }
    assert!(
        !schema_text.contains("\"const\":\"dance-performance\""),
        "producer-side schema must stay closed over known class strings"
    );
}

#[cfg(feature = "schema")]
#[test]
fn public_discriminator_schema_alignment_corpus_matches_dispatcher() {
    let schema = serde_json::to_value(schemars::schema_for!(InputReference)).unwrap();
    let schema_text = serde_json::to_string(&schema).unwrap();

    let known_valid = r#"{ "class": "monograph", "type": "book", "title": "B", "issued": "2024" }"#;
    let wrong_class_field = r#"{
            "class": "legal-case",
            "title": "Smith v. Jones",
            "monograph-type": "book"
        }"#;
    let unknown_class = r#"{
            "class": "dance-performance",
            "id": "pina2011",
            "title": "Pina",
            "venue": "Berlin"
        }"#;

    assert!(
        parse_reference(known_valid).is_ok(),
        "known-valid corpus row must parse through the dispatcher"
    );
    assert!(
        schema_text.contains("\"const\":\"monograph\""),
        "known-valid corpus row must have a matching schema branch"
    );
    // Cross-class field on a known class: the runtime captures it into
    // `unknown_fields` per the forward-compat SoftDegrade contract
    // (`docs/specs/FORWARD_COMPATIBILITY.md`, row 07). Producer-side
    // typo catching is the JSON-Schema's job — schema strictness
    // (`unevaluatedProperties: false`) is asserted separately below.
    let parsed_cross_class = parse_reference(wrong_class_field)
        .expect("wrong-class field must soft-degrade through the runtime path");
    assert!(
        matches!(parsed_cross_class.class(), ReferenceClass::LegalCase),
        "wrong-class field must not change the dispatched class"
    );
    assert!(
        schema_text.contains("\"const\":\"legal-case\"")
            && schema_text.contains("\"authority\"")
            && schema_text.contains("\"type\""),
        "schema must expose both relevant branches so unevaluatedProperties can reject cross-class leakage at the producer boundary"
    );

    let parsed_unknown = parse_reference(unknown_class)
        .expect("unknown class must parse through the consumer compatibility path");
    assert!(matches!(
        parsed_unknown.class(),
        ReferenceClass::Unknown(ref class) if class == "dance-performance"
    ));
    assert!(
        !schema_text.contains("\"const\":\"dance-performance\""),
        "schema intentionally rejects unknown producer-side class strings"
    );
}
