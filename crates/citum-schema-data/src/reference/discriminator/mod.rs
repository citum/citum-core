/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Layer-1 scaffolding for `csl26-1bdr` / `csl26-odgh`.
//!
//! Implements the design from `docs/specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md`
//! (v0.3) on a representative subset of classes — Monograph, LegalCase,
//! AudioVisual — so the serde mechanics, schemars wiring, and error UX are
//! locked in at production scale before the cutover.
//!
//! Once the remaining 15 classes are added, this module replaces the legacy
//! `pub enum InputReference` in `super::mod.rs`. The legacy types stay alive
//! in parallel during the refactor; nothing in this module depends on them
//! and nothing else in the crate depends on this module yet.

#![allow(
    clippy::too_many_lines,
    reason = "scaffolding will be split when expanded to all 18 classes"
)]

use std::collections::BTreeSet;
use std::fmt;

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map as JsonMap, Value as JsonValue};

#[cfg(feature = "schema")]
use schemars::JsonSchema;

use super::DateValue;
use super::contributor::ContributorList;
use super::types::common::{RefID, Title};

// ---------------- Public surface ----------------

/// Top-level bibliographic-data type with shared base + class-specific overlay.
///
/// See `docs/specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md` §Wire format for
/// the YAML/JSON contract and §Rust API for the accessor surface.
#[derive(Debug, Clone, PartialEq)]
pub struct InputReference {
    /// Reference identifier.
    pub id: RefID,
    /// Title of the work.
    pub title: Option<Title>,
    /// Unified contributor list.
    pub contributors: Option<ContributorList>,
    /// Publication date.
    pub issued: Option<DateValue>,
    /// Creation or origination date.
    pub created: Option<DateValue>,
    /// Note field.
    pub note: Option<String>,
    /// Class-specific overlay. Internal; consumers go through accessors.
    pub(crate) extension: ClassExtension,
}

impl InputReference {
    /// Return the typed class discriminator.
    #[must_use]
    pub fn class(&self) -> ReferenceClass {
        match &self.extension {
            ClassExtension::Monograph(_) => ReferenceClass::Monograph,
            ClassExtension::LegalCase(_) => ReferenceClass::LegalCase,
            ClassExtension::AudioVisual(_) => ReferenceClass::AudioVisual,
            ClassExtension::Unknown(u) => ReferenceClass::Unknown(u.class.clone()),
        }
    }

    /// Return the class-specific overlay for pattern-matching.
    #[must_use]
    pub fn extension(&self) -> &ClassExtension {
        &self.extension
    }

    /// Return monograph-specific fields when the class is `monograph`.
    #[must_use]
    pub fn as_monograph(&self) -> Option<&MonographFields> {
        match &self.extension {
            ClassExtension::Monograph(m) => Some(m),
            _ => None,
        }
    }

    /// Return legal-case-specific fields when the class is `legal-case`.
    #[must_use]
    pub fn as_legal_case(&self) -> Option<&LegalCaseFields> {
        match &self.extension {
            ClassExtension::LegalCase(c) => Some(c),
            _ => None,
        }
    }

    /// Return audio-visual-specific fields when the class is `audio-visual`.
    #[must_use]
    pub fn as_audio_visual(&self) -> Option<&AudioVisualFields> {
        match &self.extension {
            ClassExtension::AudioVisual(a) => Some(a),
            _ => None,
        }
    }

    /// Return unknown-class data when the class is not in the known vocabulary.
    #[must_use]
    pub fn unknown_class(&self) -> Option<&UnknownClassData> {
        match &self.extension {
            ClassExtension::Unknown(u) => Some(u),
            _ => None,
        }
    }
}

/// Typed class discriminator returned by [`InputReference::class`].
///
/// The wire form for `class:` is always the raw kebab-case string. The
/// `Unknown` variant is `#[serde(skip)]` so it never appears literally on
/// the wire — the underlying string round-trips through
/// [`UnknownClassData::class`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceClass {
    /// A monograph (book, report, thesis, etc.).
    Monograph,
    /// A legal case.
    LegalCase,
    /// An audio-visual work (film, recording, broadcast).
    AudioVisual,
    /// Forward-compat: a class string the engine does not recognize.
    #[serde(skip)]
    Unknown(String),
}

impl ReferenceClass {
    /// Names of all variants in the **known** vocabulary, kebab-case as on
    /// the wire. Used by the dispatcher and by the JSON-Schema generator.
    pub const KNOWN: &'static [&'static str] = &["monograph", "legal-case", "audio-visual"];
}

/// Class-specific overlay; one variant per known class plus an `Unknown`
/// arm for the forward-compat path.
#[derive(Debug, Clone, PartialEq)]
pub enum ClassExtension {
    /// Monograph-specific fields.
    Monograph(MonographFields),
    /// Legal-case-specific fields.
    LegalCase(LegalCaseFields),
    /// Audio-visual-specific fields.
    AudioVisual(AudioVisualFields),
    /// Class not in the known vocabulary. Round-trips via the dispatcher
    /// and the custom Serialize impl on [`InputReference`].
    Unknown(UnknownClassData),
}

/// Monograph-specific fields.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct MonographFields {
    /// Monograph subtype (book, report, thesis, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monograph_type: Option<String>,
    /// Edition designation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    /// Volume number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// ISBN identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isbn: Option<String>,
}

/// Legal-case-specific fields.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct LegalCaseFields {
    /// Court name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub court: Option<String>,
    /// Docket or case number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docket_number: Option<String>,
    /// Legal reporter series.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reporter: Option<String>,
}

/// Audio-visual-specific fields.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct AudioVisualFields {
    /// Runtime in minutes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_minutes: Option<u32>,
    /// Medium descriptor (film, television, recording, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medium: Option<String>,
}

/// Unknown-class payload captured by the dispatcher.
#[derive(Debug, Clone, PartialEq)]
pub struct UnknownClassData {
    /// Raw class string from the input.
    pub class: String,
    /// Class-specific fields captured verbatim.
    pub fields: JsonMap<String, JsonValue>,
}

// ---------------- Static field tables ----------------
//
// The dispatcher uses these to split the incoming map into shared vs
// class-specific buckets and to produce scope-correct error messages.
// Must stay in sync with the struct definitions above; a future macro can
// derive them.

const SHARED_KEYS: &[&str] = &["id", "title", "contributors", "issued", "created", "note"];

const MONOGRAPH_KEYS: &[&str] = &["monograph-type", "edition", "volume", "isbn"];
const LEGAL_CASE_KEYS: &[&str] = &["court", "docket-number", "reporter"];
const AUDIO_VISUAL_KEYS: &[&str] = &["runtime-minutes", "medium"];

fn class_keys(class: &str) -> Option<&'static [&'static str]> {
    match class {
        "monograph" => Some(MONOGRAPH_KEYS),
        "legal-case" => Some(LEGAL_CASE_KEYS),
        "audio-visual" => Some(AUDIO_VISUAL_KEYS),
        _ => None,
    }
}

// ---------------- Shared-fields helper ----------------
//
// Strict deserializer for the shared bucket. Separate from `InputReference`
// itself because the outer type needs a hand-written impl for the dispatch
// logic.

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct SharedFields {
    id: RefID,
    #[serde(default)]
    title: Option<Title>,
    #[serde(default)]
    contributors: Option<ContributorList>,
    #[serde(default)]
    issued: Option<DateValue>,
    #[serde(default)]
    created: Option<DateValue>,
    #[serde(default)]
    note: Option<String>,
}

// ---------------- Hand-written Deserialize ----------------

impl<'de> Deserialize<'de> for InputReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RefVisitor;

        impl<'de> Visitor<'de> for RefVisitor {
            type Value = InputReference;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a reference object with a `class` discriminator")
            }

            fn visit_map<M>(self, mut map: M) -> Result<InputReference, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut entries: Vec<(String, JsonValue)> = Vec::new();
                let mut class: Option<String> = None;
                let mut seen: BTreeSet<String> = BTreeSet::new();

                while let Some(key) = map.next_key::<String>()? {
                    if !seen.insert(key.clone()) {
                        return Err(de::Error::custom(format!("duplicate field `{key}`")));
                    }
                    if key == "class" {
                        class = Some(map.next_value()?);
                    } else {
                        let v: JsonValue = map.next_value()?;
                        entries.push((key, v));
                    }
                }

                let class = class.ok_or_else(|| de::Error::missing_field("class"))?;
                let known = class_keys(&class);

                let mut shared_map: JsonMap<String, JsonValue> = JsonMap::new();
                let mut class_map: JsonMap<String, JsonValue> = JsonMap::new();

                for (k, v) in entries {
                    if SHARED_KEYS.contains(&k.as_str()) {
                        shared_map.insert(k, v);
                    } else if let Some(keys) = known {
                        if keys.contains(&k.as_str()) {
                            class_map.insert(k, v);
                        } else {
                            return Err(de::Error::custom(format!(
                                "unknown field `{k}` for class `{class}`; \
                                 known shared fields: {SHARED_KEYS:?}, \
                                 known fields for this class: {keys:?}"
                            )));
                        }
                    } else {
                        // Unknown class — capture verbatim.
                        class_map.insert(k, v);
                    }
                }

                let shared: SharedFields = serde_json::from_value(JsonValue::Object(shared_map))
                    .map_err(de::Error::custom)?;

                let extension = match class.as_str() {
                    "monograph" => {
                        let f: MonographFields =
                            serde_json::from_value(JsonValue::Object(class_map)).map_err(|e| {
                                de::Error::custom(format!("class `monograph`: {e}"))
                            })?;
                        ClassExtension::Monograph(f)
                    }
                    "legal-case" => {
                        let f: LegalCaseFields =
                            serde_json::from_value(JsonValue::Object(class_map)).map_err(|e| {
                                de::Error::custom(format!("class `legal-case`: {e}"))
                            })?;
                        ClassExtension::LegalCase(f)
                    }
                    "audio-visual" => {
                        let f: AudioVisualFields =
                            serde_json::from_value(JsonValue::Object(class_map)).map_err(|e| {
                                de::Error::custom(format!("class `audio-visual`: {e}"))
                            })?;
                        ClassExtension::AudioVisual(f)
                    }
                    other => ClassExtension::Unknown(UnknownClassData {
                        class: other.to_string(),
                        fields: class_map,
                    }),
                };

                Ok(InputReference {
                    id: shared.id,
                    title: shared.title,
                    contributors: shared.contributors,
                    issued: shared.issued,
                    created: shared.created,
                    note: shared.note,
                    extension,
                })
            }
        }

        deserializer.deserialize_map(RefVisitor)
    }
}

// ---------------- Hand-written Serialize ----------------

impl Serialize for InputReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let class_str = match &self.extension {
            ClassExtension::Monograph(_) => "monograph",
            ClassExtension::LegalCase(_) => "legal-case",
            ClassExtension::AudioVisual(_) => "audio-visual",
            ClassExtension::Unknown(u) => &u.class,
        };

        // Materialize the class-specific fields as a JSON object so we can
        // iterate without committing to an entry count up front.
        let class_value = match &self.extension {
            ClassExtension::Monograph(f) => serde_json::to_value(f),
            ClassExtension::LegalCase(f) => serde_json::to_value(f),
            ClassExtension::AudioVisual(f) => serde_json::to_value(f),
            ClassExtension::Unknown(u) => Ok(JsonValue::Object(u.fields.clone())),
        }
        .map_err(serde::ser::Error::custom)?;
        let class_map = match class_value {
            JsonValue::Object(m) => m,
            JsonValue::Null => JsonMap::new(),
            _ => {
                return Err(serde::ser::Error::custom(
                    "class-specific fields did not serialize as an object",
                ));
            }
        };

        let mut total = 2 + class_map.len();
        if self.title.is_some() {
            total += 1;
        }
        if self.contributors.is_some() {
            total += 1;
        }
        if self.issued.is_some() {
            total += 1;
        }
        if self.created.is_some() {
            total += 1;
        }
        if self.note.is_some() {
            total += 1;
        }

        let mut out = serializer.serialize_map(Some(total))?;
        out.serialize_entry("id", &self.id)?;
        out.serialize_entry("class", class_str)?;
        if let Some(t) = &self.title {
            out.serialize_entry("title", t)?;
        }
        if let Some(c) = &self.contributors {
            out.serialize_entry("contributors", c)?;
        }
        if let Some(i) = &self.issued {
            out.serialize_entry("issued", i)?;
        }
        if let Some(c) = &self.created {
            out.serialize_entry("created", c)?;
        }
        if let Some(n) = &self.note {
            out.serialize_entry("note", n)?;
        }
        for (k, v) in &class_map {
            out.serialize_entry(k, v)?;
        }
        out.end()
    }
}

// ---------------- Tests ----------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "Panicking is acceptable in tests."
)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> Result<InputReference, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    #[test]
    fn valid_monograph_parses() {
        let yaml = r#"
id: smith2026
class: monograph
title: A Book
issued: "2026"
monograph-type: book
volume: "2"
"#;
        let r = parse(yaml).unwrap();
        assert_eq!(r.id.as_str(), "smith2026");
        assert!(matches!(r.class(), ReferenceClass::Monograph));
        let m = r.as_monograph().unwrap();
        assert_eq!(m.monograph_type.as_deref(), Some("book"));
        assert_eq!(m.volume.as_deref(), Some("2"));
    }

    #[test]
    fn known_class_with_no_class_specific_fields() {
        let yaml = r#"
id: smith2026
class: legal-case
title: Smith v. Jones
"#;
        let r = parse(yaml).unwrap();
        assert!(matches!(r.class(), ReferenceClass::LegalCase));
        let c = r.as_legal_case().unwrap();
        assert!(c.court.is_none());
        assert!(c.docket_number.is_none());
    }

    #[test]
    fn typo_in_shared_field_rejected() {
        let yaml = r#"
id: smith2026
class: monograph
titel: A Book
"#;
        let err = parse(yaml).unwrap_err().to_string();
        assert!(
            err.contains("titel"),
            "expected unknown-field error mentioning `titel`, got: {err}"
        );
    }

    #[test]
    fn typo_in_class_specific_field_rejected() {
        let yaml = r#"
id: smith2026
class: monograph
title: A Book
monogarph-type: book
"#;
        let err = parse(yaml).unwrap_err().to_string();
        assert!(
            err.contains("monogarph") && err.contains("monograph"),
            "expected scope-correct unknown-field error, got: {err}"
        );
    }

    #[test]
    fn class_specific_field_under_wrong_class_rejected() {
        let yaml = r#"
id: smith2026
class: legal-case
title: Smith v. Jones
monograph-type: book
"#;
        let err = parse(yaml).unwrap_err().to_string();
        assert!(
            err.contains("monograph-type") && err.contains("legal-case"),
            "expected wrong-class error, got: {err}"
        );
    }

    #[test]
    fn unknown_class_round_trips() {
        let yaml = r#"
id: perf2026
class: dance-performance
title: Pina
issued: "2011"
venue: Berlin
duration-minutes: 103
"#;
        let r = parse(yaml).unwrap();
        let u = r.unknown_class().unwrap();
        assert_eq!(u.class, "dance-performance");
        assert_eq!(u.fields.len(), 2);
        assert!(u.fields.contains_key("venue"));

        let reserialized = serde_yaml::to_string(&r).unwrap();
        let r2 = parse(&reserialized).unwrap();
        assert_eq!(r, r2);
    }

    #[test]
    fn typo_in_class_value_captured_as_unknown() {
        let yaml = r#"
id: smith2026
class: monogarph
title: A Book
monograph-type: book
"#;
        let r = parse(yaml).unwrap();
        let u = r.unknown_class().unwrap();
        assert_eq!(u.class, "monogarph");
    }

    #[test]
    fn known_class_round_trip() {
        let original = InputReference {
            id: RefID("av2026".to_string()),
            title: Some(Title::Single("Pina".to_string())),
            contributors: None,
            issued: Some(DateValue::new("2011".to_string())),
            created: None,
            note: None,
            extension: ClassExtension::AudioVisual(AudioVisualFields {
                runtime_minutes: Some(103),
                medium: Some("film".into()),
            }),
        };
        let yaml = serde_yaml::to_string(&original).unwrap();
        assert!(
            !yaml.contains("class_data"),
            "wire shape must be flat: {yaml}"
        );
        assert!(
            !yaml.contains("extension:"),
            "wire shape must be flat: {yaml}"
        );
        let round = parse(&yaml).unwrap();
        assert_eq!(original, round);
    }
}
