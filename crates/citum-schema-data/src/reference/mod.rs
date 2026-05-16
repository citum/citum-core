/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! A reference is a bibliographic item, such as a book, article, or web page.
//! It is the basic unit of bibliographic data.

pub mod contributor;
#[cfg(feature = "legacy-convert")]
pub mod conversion;
pub mod date;
pub mod types;

use std::sync::LazyLock;

#[cfg(all(test, feature = "legacy-convert"))]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests;

pub use self::contributor::{
    Contributor, ContributorEntry, ContributorGender, ContributorList, ContributorRole, FlatName,
    SimpleName, StructuredName,
};
pub use self::date::EdtfString;
use self::types::common::HasNumbering;
pub use self::types::common::{
    FieldLanguageMap, LangID, MultilingualString, NumOrStr, Numbering, NumberingType, Place,
    Publisher, RefID, RichText, Title,
};
pub use self::types::legal::{Brief, Hearing, LegalCase, Regulation, Statute, Treaty};
pub use self::types::specialized::{
    AudioVisualType, AudioVisualWork, Classic, Dataset, Event, Patent, Software, Standard, WorkCore,
};
pub use self::types::structural::{
    Collection, CollectionComponent, CollectionType, Monograph, MonographComponentType,
    MonographType, Serial, SerialComponent, SerialComponentType, SerialType,
};

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map as JsonMap, Value as JsonValue};
#[cfg(feature = "bindings")]
use specta::Type;
use url::Url;

/// Empty field-language map returned by accessors on unknown-class references.
///
/// `FieldLanguageMap` is a `HashMap`, whose `::new()` is not `const`, so a
/// `LazyLock` is required. The map is constructed once for the process and
/// reused by every unknown-class reference.
static EMPTY_FIELD_LANGUAGES: LazyLock<FieldLanguageMap> = LazyLock::new(FieldLanguageMap::new);

/// A relation to another bibliographic entity.
///
/// Untagged in serde to allow either an inline object or a string ID reference.
/// Used for both hierarchical (`container`) and associative (`original`, `reviewed`, `series`) links.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(untagged)]
pub enum WorkRelation {
    /// The target work is referenced by its ID (resolved at render time).
    Id(RefID),
    /// The target work is embedded inline.
    Embedded(Box<InputReference>),
}

/// The Reference model: a class-specific overlay reachable through accessor methods.
///
/// All shared bibliographic data (id, title, contributors, dates, publisher, ...)
/// lives inside the class-specific payload in `extension`. The accessor methods
/// (`id()`, `title()`, etc.) dispatch through the extension and are the public
/// read path; the typed setters (`set_id`, ...) are the public mutation path.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bindings", derive(Type))]
pub struct InputReference {
    pub(crate) extension: ClassExtension,
}

/// Unknown reference-class payload captured by the discriminator dispatcher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
pub struct UnknownClassData {
    /// Raw `class:` string from the input object.
    pub class: String,
    /// Non-shared fields captured verbatim for round-trip preservation.
    #[cfg_attr(feature = "bindings", specta(type = serde_json::Value))]
    pub fields: JsonMap<String, JsonValue>,
}

/// Typed class discriminator returned by [`InputReference::class`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceClass {
    /// A monograph, such as a book or report.
    Monograph,
    /// A component of a larger monographic collection.
    CollectionComponent,
    /// A component of a larger serial publication.
    SerialComponent,
    /// A collection of works, such as an anthology or proceedings.
    Collection,
    /// A serial publication, such as a journal or newspaper.
    Serial,
    /// A legal case.
    LegalCase,
    /// A statute or legislative act.
    Statute,
    /// An international treaty or agreement.
    Treaty,
    /// A legislative or administrative hearing.
    Hearing,
    /// An administrative regulation.
    Regulation,
    /// A legal brief or filing.
    Brief,
    /// A classic work with standard citation forms.
    Classic,
    /// A patent.
    Patent,
    /// A research dataset.
    Dataset,
    /// A technical standard or specification.
    Standard,
    /// Software or source code.
    Software,
    /// An event such as a conference, performance, or broadcast.
    Event,
    /// An audio-visual work.
    AudioVisual,
    /// A class string not known by this version.
    ///
    /// `#[serde(skip)]`: an `Unknown(...)` variant cannot serialize directly
    /// through the derived `Serialize` impl on `ReferenceClass`. The class
    /// string is preserved on the wire via [`UnknownClassData::class`] inside
    /// the surrounding `InputReference`, not via this enum standalone.
    #[serde(skip)]
    Unknown(String),
}

impl ReferenceClass {
    /// Known class names in their wire-format spelling.
    pub const KNOWN: &'static [&'static str] = &[
        "monograph",
        "collection-component",
        "serial-component",
        "collection",
        "serial",
        "legal-case",
        "statute",
        "treaty",
        "hearing",
        "regulation",
        "brief",
        "classic",
        "patent",
        "dataset",
        "standard",
        "software",
        "event",
        "audio-visual",
    ];
}

/// Class-specific overlay stored inside [`InputReference`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bindings", derive(Type))]
pub enum ClassExtension {
    /// Monograph-specific payload.
    Monograph(Box<Monograph>),
    /// Collection-component-specific payload.
    CollectionComponent(Box<CollectionComponent>),
    /// Serial-component-specific payload.
    SerialComponent(Box<SerialComponent>),
    /// Collection-specific payload.
    Collection(Box<Collection>),
    /// Serial-specific payload.
    Serial(Box<Serial>),
    /// Legal-case-specific payload.
    LegalCase(Box<LegalCase>),
    /// Statute-specific payload.
    Statute(Box<Statute>),
    /// Treaty-specific payload.
    Treaty(Box<Treaty>),
    /// Hearing-specific payload.
    Hearing(Box<Hearing>),
    /// Regulation-specific payload.
    Regulation(Box<Regulation>),
    /// Brief-specific payload.
    Brief(Box<Brief>),
    /// Classic-work-specific payload.
    Classic(Box<Classic>),
    /// Patent-specific payload.
    Patent(Box<Patent>),
    /// Dataset-specific payload.
    Dataset(Box<Dataset>),
    /// Standard-specific payload.
    Standard(Box<Standard>),
    /// Software-specific payload.
    Software(Box<Software>),
    /// Event-specific payload.
    Event(Box<Event>),
    /// Audio-visual-specific payload.
    AudioVisual(Box<AudioVisualWork>),
    /// Unknown-class payload.
    Unknown(Box<UnknownClassData>),
}

macro_rules! boxed_reference_constructor {
    ($fn_name:ident, $ty:ty, $variant:ident) => {
        fn $fn_name(reference: Box<$ty>) -> Self {
            Self {
                extension: ClassExtension::$variant(reference),
            }
        }
    };
}

impl InputReference {
    fn from_known<T>(
        class_extension: impl FnOnce(Box<T>) -> ClassExtension,
        value: JsonValue,
    ) -> Result<Self, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Defensive: the dispatcher in `deserialize_reference_body` only ever
        // hands us a `JsonValue::Object`; reject anything else with a schema
        // error rather than letting `from_value` produce an opaque type error.
        if !matches!(&value, JsonValue::Object(_)) {
            return Err(<serde_json::Error as serde::de::Error>::custom(
                "reference body must be a JSON object",
            ));
        }
        let inner = serde_json::from_value(value)?;
        Ok(Self {
            extension: class_extension(Box::new(inner)),
        })
    }

    // ────────────────────────────────────────────────────────────────────
    // Transitional PascalCase constructors.
    //
    // These exist to keep call sites that used to construct an `InputReference`
    // enum variant working unchanged through the discriminator cutover
    // (bean: csl26-1bdr). They are NOT the long-term public surface and
    // SHOULD NOT be relied upon by new code — use a `Box::new(<Class> { … })`
    // value plus a future snake_case factory once one is exposed.
    //
    // TODO(csl26-1bdr): replace these 19 functions with idiomatic snake_case
    // constructors and drop the transitional set. Tracked by parent epic.
    // ────────────────────────────────────────────────────────────────────

    /// Construct a monograph reference (transitional; see block note above).
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Monograph(reference: Box<Monograph>) -> Self {
        Self::from_boxed_monograph(reference)
    }

    /// Construct a collection-component reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn CollectionComponent(reference: Box<CollectionComponent>) -> Self {
        Self::from_boxed_collection_component(reference)
    }

    /// Construct a serial-component reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn SerialComponent(reference: Box<SerialComponent>) -> Self {
        Self::from_boxed_serial_component(reference)
    }

    /// Construct a collection reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Collection(reference: Box<Collection>) -> Self {
        Self::from_boxed_collection(reference)
    }

    /// Construct a serial reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Serial(reference: Box<Serial>) -> Self {
        Self::from_boxed_serial(reference)
    }

    /// Construct a legal-case reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn LegalCase(reference: Box<LegalCase>) -> Self {
        Self::from_boxed_legal_case(reference)
    }

    /// Construct a statute reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Statute(reference: Box<Statute>) -> Self {
        Self::from_boxed_statute(reference)
    }

    /// Construct a treaty reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Treaty(reference: Box<Treaty>) -> Self {
        Self::from_boxed_treaty(reference)
    }

    /// Construct a hearing reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Hearing(reference: Box<Hearing>) -> Self {
        Self::from_boxed_hearing(reference)
    }

    /// Construct a regulation reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Regulation(reference: Box<Regulation>) -> Self {
        Self::from_boxed_regulation(reference)
    }

    /// Construct a brief reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Brief(reference: Box<Brief>) -> Self {
        Self::from_boxed_brief(reference)
    }

    /// Construct a classic reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Classic(reference: Box<Classic>) -> Self {
        Self::from_boxed_classic(reference)
    }

    /// Construct a patent reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Patent(reference: Box<Patent>) -> Self {
        Self::from_boxed_patent(reference)
    }

    /// Construct a dataset reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Dataset(reference: Box<Dataset>) -> Self {
        Self::from_boxed_dataset(reference)
    }

    /// Construct a standard reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Standard(reference: Box<Standard>) -> Self {
        Self::from_boxed_standard(reference)
    }

    /// Construct a software reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Software(reference: Box<Software>) -> Self {
        Self::from_boxed_software(reference)
    }

    /// Construct an event reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Event(reference: Box<Event>) -> Self {
        Self::from_boxed_event(reference)
    }

    /// Construct an audio-visual reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn AudioVisual(reference: Box<AudioVisualWork>) -> Self {
        Self::from_boxed_audio_visual(reference)
    }

    /// Construct an unknown-class reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Unknown(reference: Box<UnknownClassData>) -> Self {
        Self {
            extension: ClassExtension::Unknown(reference),
        }
    }

    boxed_reference_constructor!(from_boxed_monograph, Monograph, Monograph);
    boxed_reference_constructor!(
        from_boxed_collection_component,
        CollectionComponent,
        CollectionComponent
    );
    boxed_reference_constructor!(
        from_boxed_serial_component,
        SerialComponent,
        SerialComponent
    );
    boxed_reference_constructor!(from_boxed_collection, Collection, Collection);
    boxed_reference_constructor!(from_boxed_serial, Serial, Serial);
    boxed_reference_constructor!(from_boxed_legal_case, LegalCase, LegalCase);
    boxed_reference_constructor!(from_boxed_statute, Statute, Statute);
    boxed_reference_constructor!(from_boxed_treaty, Treaty, Treaty);
    boxed_reference_constructor!(from_boxed_hearing, Hearing, Hearing);
    boxed_reference_constructor!(from_boxed_regulation, Regulation, Regulation);
    boxed_reference_constructor!(from_boxed_brief, Brief, Brief);
    boxed_reference_constructor!(from_boxed_classic, Classic, Classic);
    boxed_reference_constructor!(from_boxed_patent, Patent, Patent);
    boxed_reference_constructor!(from_boxed_dataset, Dataset, Dataset);
    boxed_reference_constructor!(from_boxed_standard, Standard, Standard);
    boxed_reference_constructor!(from_boxed_software, Software, Software);
    boxed_reference_constructor!(from_boxed_event, Event, Event);
    boxed_reference_constructor!(from_boxed_audio_visual, AudioVisualWork, AudioVisual);
}

/// Produce a serde duplicate-field error with the canonical
/// `duplicate field \`<name>\`` shape.
///
/// `serde::de::Error::duplicate_field` requires `&'static str`; our keys are
/// dynamic, so we route through `custom` while preserving the exact message
/// format that `duplicate_field` would emit. Downstream consumers matching
/// on the `Display` output see identical text.
fn duplicate_field_error<E: de::Error>(field: &str) -> E {
    de::Error::custom(format_args!("duplicate field `{field}`"))
}

impl<'de> Deserialize<'de> for InputReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ReferenceVisitor;

        impl<'de> Visitor<'de> for ReferenceVisitor {
            type Value = InputReference;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a flat reference object with a `class` discriminator")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut class = None;
                let mut body = JsonMap::new();

                while let Some(key) = map.next_key::<String>()? {
                    if key == "class" {
                        if class.is_some() {
                            // Canonical serde duplicate-field error; matches the
                            // shape that `de::Error::duplicate_field` would emit
                            // for the dynamic-name path below.
                            return Err(duplicate_field_error::<M::Error>("class"));
                        }
                        class = Some(map.next_value::<String>()?);
                    } else {
                        let value = map.next_value::<JsonValue>()?;
                        if body.insert(key.clone(), value).is_some() {
                            return Err(duplicate_field_error::<M::Error>(&key));
                        }
                    }
                }

                let class = class.ok_or_else(|| de::Error::missing_field("class"))?;
                deserialize_reference_body(&class, body).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_map(ReferenceVisitor)
    }
}

/// Flat-with-class serialization proxy: prepends a `class` field and flattens
/// the typed payload directly through the serializer. Avoids the
/// `serde_json::to_value` round-trip that the previous implementation used
/// to compute the body map.
#[derive(Serialize)]
struct FlatClassProxy<'a, T: Serialize + ?Sized> {
    class: &'a str,
    #[serde(flatten)]
    inner: &'a T,
}

impl Serialize for InputReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // For known classes we let serde's `flatten` machinery splat the
        // typed inner struct directly into the parent serializer — no
        // intermediate `serde_json::Value` allocation per reference. For
        // `Unknown`, the payload is already a `JsonMap`, so we walk it
        // directly.
        match &self.extension {
            ClassExtension::Monograph(inner) => FlatClassProxy {
                class: "monograph",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::CollectionComponent(inner) => FlatClassProxy {
                class: "collection-component",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::SerialComponent(inner) => FlatClassProxy {
                class: "serial-component",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Collection(inner) => FlatClassProxy {
                class: "collection",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Serial(inner) => FlatClassProxy {
                class: "serial",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::LegalCase(inner) => FlatClassProxy {
                class: "legal-case",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Statute(inner) => FlatClassProxy {
                class: "statute",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Treaty(inner) => FlatClassProxy {
                class: "treaty",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Hearing(inner) => FlatClassProxy {
                class: "hearing",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Regulation(inner) => FlatClassProxy {
                class: "regulation",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Brief(inner) => FlatClassProxy {
                class: "brief",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Classic(inner) => FlatClassProxy {
                class: "classic",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Patent(inner) => FlatClassProxy {
                class: "patent",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Dataset(inner) => FlatClassProxy {
                class: "dataset",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Standard(inner) => FlatClassProxy {
                class: "standard",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Software(inner) => FlatClassProxy {
                class: "software",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Event(inner) => FlatClassProxy {
                class: "event",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::AudioVisual(inner) => FlatClassProxy {
                class: "audio-visual",
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Unknown(data) => {
                let mut out = serializer.serialize_map(Some(data.fields.len() + 1))?;
                out.serialize_entry("class", &data.class)?;
                for (key, value) in &data.fields {
                    out.serialize_entry(key, value)?;
                }
                out.end()
            }
        }
    }
}

#[cfg(feature = "schema")]
impl JsonSchema for InputReference {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "InputReference".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let variants = [
            reference_schema_branch::<Monograph>(generator, "monograph"),
            reference_schema_branch::<CollectionComponent>(generator, "collection-component"),
            reference_schema_branch::<SerialComponent>(generator, "serial-component"),
            reference_schema_branch::<Collection>(generator, "collection"),
            reference_schema_branch::<Serial>(generator, "serial"),
            reference_schema_branch::<LegalCase>(generator, "legal-case"),
            reference_schema_branch::<Statute>(generator, "statute"),
            reference_schema_branch::<Treaty>(generator, "treaty"),
            reference_schema_branch::<Hearing>(generator, "hearing"),
            reference_schema_branch::<Regulation>(generator, "regulation"),
            reference_schema_branch::<Brief>(generator, "brief"),
            reference_schema_branch::<Classic>(generator, "classic"),
            reference_schema_branch::<Patent>(generator, "patent"),
            reference_schema_branch::<Dataset>(generator, "dataset"),
            reference_schema_branch::<Standard>(generator, "standard"),
            reference_schema_branch::<Software>(generator, "software"),
            reference_schema_branch::<Event>(generator, "event"),
            reference_schema_branch::<AudioVisualWork>(generator, "audio-visual"),
        ];

        schemars::json_schema!({
            "oneOf": variants,
            "unevaluatedProperties": false
        })
    }
}

#[cfg(feature = "schema")]
fn reference_schema_branch<T: JsonSchema>(
    generator: &mut schemars::SchemaGenerator,
    class: &'static str,
) -> JsonValue {
    let mut schema = T::json_schema(generator);
    let object = schema.ensure_object();
    if !object.get("properties").is_some_and(JsonValue::is_object) {
        object.insert("properties".to_string(), JsonValue::Object(JsonMap::new()));
    }
    let Some(properties) = object
        .get_mut("properties")
        .and_then(JsonValue::as_object_mut)
    else {
        return schema.to_value();
    };
    properties.insert(
        "class".to_string(),
        serde_json::json!({
            "type": "string",
            "const": class
        }),
    );

    if !object.get("required").is_some_and(JsonValue::is_array) {
        object.insert("required".to_string(), JsonValue::Array(Vec::new()));
    }
    let Some(required) = object.get_mut("required").and_then(JsonValue::as_array_mut) else {
        return schema.to_value();
    };
    if !required.iter().any(|value| value.as_str() == Some("class")) {
        required.push(JsonValue::String("class".to_string()));
    }

    schema.to_value()
}

fn deserialize_reference_body(
    class: &str,
    body: JsonMap<String, JsonValue>,
) -> Result<InputReference, serde_json::Error> {
    let value = JsonValue::Object(body);
    match class {
        "monograph" => InputReference::from_known(ClassExtension::Monograph, value),
        "collection-component" => {
            InputReference::from_known(ClassExtension::CollectionComponent, value)
        }
        "serial-component" => InputReference::from_known(ClassExtension::SerialComponent, value),
        "collection" => InputReference::from_known(ClassExtension::Collection, value),
        "serial" => InputReference::from_known(ClassExtension::Serial, value),
        "legal-case" => InputReference::from_known(ClassExtension::LegalCase, value),
        "statute" => InputReference::from_known(ClassExtension::Statute, value),
        "treaty" => InputReference::from_known(ClassExtension::Treaty, value),
        "hearing" => InputReference::from_known(ClassExtension::Hearing, value),
        "regulation" => InputReference::from_known(ClassExtension::Regulation, value),
        "brief" => InputReference::from_known(ClassExtension::Brief, value),
        "classic" => InputReference::from_known(ClassExtension::Classic, value),
        "patent" => InputReference::from_known(ClassExtension::Patent, value),
        "dataset" => InputReference::from_known(ClassExtension::Dataset, value),
        "standard" => InputReference::from_known(ClassExtension::Standard, value),
        "software" => InputReference::from_known(ClassExtension::Software, value),
        "event" => InputReference::from_known(ClassExtension::Event, value),
        "audio-visual" => InputReference::from_known(ClassExtension::AudioVisual, value),
        other => {
            let fields = if let JsonValue::Object(fields) = value {
                fields
            } else {
                JsonMap::new()
            };
            Ok(InputReference::Unknown(Box::new(UnknownClassData {
                class: other.to_string(),
                fields,
            })))
        }
    }
}

impl InputReference {
    /// Return the typed class discriminator.
    #[must_use]
    pub fn class(&self) -> ReferenceClass {
        match &self.extension {
            ClassExtension::Monograph(_) => ReferenceClass::Monograph,
            ClassExtension::CollectionComponent(_) => ReferenceClass::CollectionComponent,
            ClassExtension::SerialComponent(_) => ReferenceClass::SerialComponent,
            ClassExtension::Collection(_) => ReferenceClass::Collection,
            ClassExtension::Serial(_) => ReferenceClass::Serial,
            ClassExtension::LegalCase(_) => ReferenceClass::LegalCase,
            ClassExtension::Statute(_) => ReferenceClass::Statute,
            ClassExtension::Treaty(_) => ReferenceClass::Treaty,
            ClassExtension::Hearing(_) => ReferenceClass::Hearing,
            ClassExtension::Regulation(_) => ReferenceClass::Regulation,
            ClassExtension::Brief(_) => ReferenceClass::Brief,
            ClassExtension::Classic(_) => ReferenceClass::Classic,
            ClassExtension::Patent(_) => ReferenceClass::Patent,
            ClassExtension::Dataset(_) => ReferenceClass::Dataset,
            ClassExtension::Standard(_) => ReferenceClass::Standard,
            ClassExtension::Software(_) => ReferenceClass::Software,
            ClassExtension::Event(_) => ReferenceClass::Event,
            ClassExtension::AudioVisual(_) => ReferenceClass::AudioVisual,
            ClassExtension::Unknown(data) => ReferenceClass::Unknown(data.class.clone()),
        }
    }

    /// Return the active class-specific overlay.
    #[must_use]
    pub fn extension(&self) -> &ClassExtension {
        &self.extension
    }

    /// Return the mutable active class-specific overlay.
    #[must_use]
    pub fn extension_mut(&mut self) -> &mut ClassExtension {
        &mut self.extension
    }

    /// Return monograph data when this reference is a monograph.
    #[must_use]
    pub fn as_monograph(&self) -> Option<&Monograph> {
        match &self.extension {
            ClassExtension::Monograph(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return collection-component data when this reference is a collection component.
    #[must_use]
    pub fn as_collection_component(&self) -> Option<&CollectionComponent> {
        match &self.extension {
            ClassExtension::CollectionComponent(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return serial-component data when this reference is a serial component.
    #[must_use]
    pub fn as_serial_component(&self) -> Option<&SerialComponent> {
        match &self.extension {
            ClassExtension::SerialComponent(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return collection data when this reference is a collection.
    #[must_use]
    pub fn as_collection(&self) -> Option<&Collection> {
        match &self.extension {
            ClassExtension::Collection(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return serial data when this reference is a serial.
    #[must_use]
    pub fn as_serial(&self) -> Option<&Serial> {
        match &self.extension {
            ClassExtension::Serial(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return legal-case data when this reference is a legal case.
    #[must_use]
    pub fn as_legal_case(&self) -> Option<&LegalCase> {
        match &self.extension {
            ClassExtension::LegalCase(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return statute data when this reference is a statute.
    #[must_use]
    pub fn as_statute(&self) -> Option<&Statute> {
        match &self.extension {
            ClassExtension::Statute(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return treaty data when this reference is a treaty.
    #[must_use]
    pub fn as_treaty(&self) -> Option<&Treaty> {
        match &self.extension {
            ClassExtension::Treaty(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return hearing data when this reference is a hearing.
    #[must_use]
    pub fn as_hearing(&self) -> Option<&Hearing> {
        match &self.extension {
            ClassExtension::Hearing(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return regulation data when this reference is a regulation.
    #[must_use]
    pub fn as_regulation(&self) -> Option<&Regulation> {
        match &self.extension {
            ClassExtension::Regulation(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return brief data when this reference is a brief.
    #[must_use]
    pub fn as_brief(&self) -> Option<&Brief> {
        match &self.extension {
            ClassExtension::Brief(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return classic-work data when this reference is a classic.
    #[must_use]
    pub fn as_classic(&self) -> Option<&Classic> {
        match &self.extension {
            ClassExtension::Classic(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return patent data when this reference is a patent.
    #[must_use]
    pub fn as_patent(&self) -> Option<&Patent> {
        match &self.extension {
            ClassExtension::Patent(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return dataset data when this reference is a dataset.
    #[must_use]
    pub fn as_dataset(&self) -> Option<&Dataset> {
        match &self.extension {
            ClassExtension::Dataset(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return standard data when this reference is a standard.
    #[must_use]
    pub fn as_standard(&self) -> Option<&Standard> {
        match &self.extension {
            ClassExtension::Standard(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return software data when this reference is software.
    #[must_use]
    pub fn as_software(&self) -> Option<&Software> {
        match &self.extension {
            ClassExtension::Software(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return event data when this reference is an event.
    #[must_use]
    pub fn as_event(&self) -> Option<&Event> {
        match &self.extension {
            ClassExtension::Event(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return audio-visual data when this reference is audio-visual.
    #[must_use]
    pub fn as_audio_visual(&self) -> Option<&AudioVisualWork> {
        match &self.extension {
            ClassExtension::AudioVisual(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Return unknown-class data when this reference names an unknown class.
    #[must_use]
    pub fn unknown_class(&self) -> Option<&UnknownClassData> {
        match &self.extension {
            ClassExtension::Unknown(data) => Some(data),
            _ => None,
        }
    }

    fn numbered(&self) -> Option<&dyn HasNumbering> {
        match &self.extension {
            ClassExtension::Monograph(reference) => Some(reference.as_ref()),
            ClassExtension::Collection(reference) => Some(reference.as_ref()),
            ClassExtension::CollectionComponent(reference) => Some(reference.as_ref()),
            ClassExtension::SerialComponent(reference) => Some(reference.as_ref()),
            ClassExtension::Classic(reference) => Some(reference.as_ref()),
            ClassExtension::AudioVisual(reference) => Some(reference.as_ref()),
            _ => None,
        }
    }

    /// Internal helper to find a numbering by type.
    fn find_numbering(&self, numbering_type: NumberingType) -> Option<String> {
        self.numbered()
            .and_then(|reference| reference.find_numbering(numbering_type))
    }

    /// Return the numbering value for an arbitrary numbering kind.
    pub fn numbering_value(&self, numbering_type: &NumberingType) -> Option<String> {
        self.find_numbering(numbering_type.clone())
    }

    /// Return the reference ID.
    pub fn id(&self) -> Option<RefID> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.id.clone(),
            ClassExtension::CollectionComponent(r) => r.id.clone(),
            ClassExtension::SerialComponent(r) => r.id.clone(),
            ClassExtension::Collection(r) => r.id.clone(),
            ClassExtension::Serial(r) => r.id.clone(),
            ClassExtension::LegalCase(r) => r.id.clone(),
            ClassExtension::Statute(r) => r.id.clone(),
            ClassExtension::Treaty(r) => r.id.clone(),
            ClassExtension::Hearing(r) => r.id.clone(),
            ClassExtension::Regulation(r) => r.id.clone(),
            ClassExtension::Brief(r) => r.id.clone(),
            ClassExtension::Classic(r) => r.id.clone(),
            ClassExtension::Patent(r) => r.id.clone(),
            ClassExtension::Dataset(r) => r.id.clone(),
            ClassExtension::Standard(r) => r.id.clone(),
            ClassExtension::Software(r) => r.id.clone(),
            ClassExtension::Event(r) => r.id.clone(),
            ClassExtension::AudioVisual(r) => r.id.clone(),
            ClassExtension::Unknown(data) => data
                .fields
                .get("id")
                .and_then(JsonValue::as_str)
                .map(|id| RefID(id.to_string())),
        }
    }

    /// Return the author.
    pub fn author(&self) -> Option<Contributor> {
        use crate::reference::contributor::ContributorRole as DataRole;

        match &self.extension {
            ClassExtension::Monograph(r) => {
                collect_contributors_by_role(&r.contributors, &DataRole::Author)
                    .or_else(|| r.author.clone())
            }
            ClassExtension::CollectionComponent(r) => {
                collect_contributors_by_role(&r.contributors, &DataRole::Author)
                    .or_else(|| r.author.clone())
            }
            ClassExtension::SerialComponent(r) => {
                let explicit_author =
                    collect_contributors_by_role(&r.contributors, &DataRole::Author)
                        .or_else(|| r.author.clone());

                explicit_author.or_else(|| {
                    let av_like = r
                        .medium
                        .as_deref()
                        .map(|value| {
                            let lowered = value.to_ascii_lowercase();
                            lowered.contains("podcast")
                                || lowered.contains("tv")
                                || lowered.contains("film")
                                || lowered.contains("video")
                        })
                        .unwrap_or(false)
                        || r.genre
                            .as_deref()
                            .map(|value| {
                                let lowered = value.to_ascii_lowercase();
                                lowered.contains("broadcast") || lowered.contains("film")
                            })
                            .unwrap_or(false);

                    if av_like {
                        collect_contributors_by_role(&r.contributors, &DataRole::Producer).or_else(
                            || collect_contributors_by_role(&r.contributors, &DataRole::Host),
                        )
                    } else {
                        None
                    }
                })
            }
            ClassExtension::Treaty(r) => r.author.clone(),
            ClassExtension::Brief(r) => r.author.clone(),
            ClassExtension::Classic(r) => r.author.clone(),
            ClassExtension::Patent(r) => r.author.clone(),
            ClassExtension::Dataset(r) => r.author.clone(),
            ClassExtension::Software(r) => r.author.clone(),
            ClassExtension::Event(r) => {
                collect_contributors_by_role(&r.contributors, &DataRole::Performer)
                    .or_else(|| {
                        collect_contributors_by_role(
                            &r.contributors,
                            &DataRole::Custom("organizer".to_string()),
                        )
                    })
                    .or_else(|| collect_contributors_by_role(&r.contributors, &DataRole::Author))
            }
            ClassExtension::AudioVisual(r) => {
                let explicit_author =
                    collect_contributors_by_role(&r.core.contributors, &DataRole::Author);

                match r.r#type {
                    AudioVisualType::Film | AudioVisualType::Episode => {
                        explicit_author.or_else(|| {
                            collect_contributors_by_role(&r.core.contributors, &DataRole::Director)
                        })
                    }
                    AudioVisualType::Recording => explicit_author.or_else(|| {
                        collect_contributors_by_role(&r.core.contributors, &DataRole::Composer)
                            .or_else(|| {
                                collect_contributors_by_role(
                                    &r.core.contributors,
                                    &DataRole::Performer,
                                )
                            })
                    }),
                    AudioVisualType::Broadcast => explicit_author
                        .or_else(|| {
                            collect_contributors_by_role(&r.core.contributors, &DataRole::Director)
                        })
                        .or_else(|| {
                            collect_contributors_by_role(&r.core.contributors, &DataRole::Producer)
                        }),
                }
            }
            _ => None,
        }
    }

    pub fn editor(&self) -> Option<Contributor> {
        match &self.extension {
            ClassExtension::Monograph(r) => {
                collect_contributors_by_role(&r.contributors, &ContributorRole::Editor)
                    .or_else(|| r.editor.clone())
            }
            ClassExtension::Collection(r) => {
                collect_contributors_by_role(&r.contributors, &ContributorRole::Editor)
                    .or_else(|| r.editor.clone())
            }
            ClassExtension::CollectionComponent(r) => r
                .container
                .as_ref()
                .and_then(|c| match c {
                    WorkRelation::Embedded(p) => p.editor(),
                    WorkRelation::Id(_) => None,
                })
                .or_else(|| {
                    collect_contributors_by_role(&r.contributors, &ContributorRole::Editor)
                }),
            ClassExtension::SerialComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.editor(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Serial(r) => {
                collect_contributors_by_role(&r.contributors, &ContributorRole::Editor)
                    .or_else(|| r.editor.clone())
            }
            ClassExtension::Classic(r) => r.editor.clone(),
            ClassExtension::AudioVisual(_) => None,
            _ => None,
        }
    }

    /// Return the translator.
    pub fn translator(&self) -> Option<Contributor> {
        match &self.extension {
            ClassExtension::Monograph(r) => {
                collect_contributors_by_role(&r.contributors, &ContributorRole::Translator)
                    .or_else(|| r.translator.clone())
            }
            ClassExtension::CollectionComponent(r) => {
                collect_contributors_by_role(&r.contributors, &ContributorRole::Translator)
                    .or_else(|| r.translator.clone())
            }
            ClassExtension::SerialComponent(r) => {
                collect_contributors_by_role(&r.contributors, &ContributorRole::Translator)
                    .or_else(|| r.translator.clone())
            }
            ClassExtension::Collection(r) => {
                collect_contributors_by_role(&r.contributors, &ContributorRole::Translator)
                    .or_else(|| r.translator.clone())
            }
            ClassExtension::Classic(r) => r.translator.clone(),
            ClassExtension::AudioVisual(r) => {
                collect_contributors_by_role(&r.core.contributors, &ContributorRole::Translator)
            }
            _ => None,
        }
    }

    /// Return the publisher.
    pub fn publisher(&self) -> Option<Publisher> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.publisher.clone(),
            ClassExtension::CollectionComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::SerialComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Collection(r) => r.publisher.clone(),
            ClassExtension::Serial(r) => r.publisher.clone(),
            ClassExtension::Classic(r) => r.publisher.clone(),
            ClassExtension::Dataset(r) => r.publisher.clone(),
            ClassExtension::Standard(r) => r.publisher.clone(),
            ClassExtension::Software(r) => r.publisher.clone(),
            ClassExtension::AudioVisual(r) => r.publisher.clone(),
            _ => None,
        }
    }

    /// Returns contributors matching `role` for any reference class that
    /// carries a contributors list.
    ///
    /// Returns `None` if no contributors with the given role exist.
    /// Returns a single `Contributor` directly, or folds multiple into a `ContributorList`.
    pub fn contributor(&self, role: ContributorRole) -> Option<Contributor> {
        let entries = self.all_contributor_entries();
        collect_contributors_by_role(entries, &role)
    }

    /// Return all contributor entries matching the requested role.
    pub fn contributor_entries(&self, role: &ContributorRole) -> Vec<&ContributorEntry> {
        self.all_contributor_entries()
            .iter()
            .filter(|entry| &entry.role == role)
            .collect()
    }

    fn all_contributor_entries(&self) -> &[ContributorEntry] {
        match &self.extension {
            ClassExtension::Monograph(r) => &r.contributors,
            ClassExtension::Collection(r) => &r.contributors,
            ClassExtension::CollectionComponent(r) => &r.contributors,
            ClassExtension::Serial(r) => &r.contributors,
            ClassExtension::SerialComponent(r) => &r.contributors,
            ClassExtension::Event(r) => &r.contributors,
            ClassExtension::AudioVisual(r) => &r.core.contributors,
            _ => &[],
        }
    }

    /// Return the title.
    pub fn title(&self) -> Option<Title> {
        match &self.extension {
            ClassExtension::Monograph(r) => match (&r.title, &r.short_title) {
                (Some(Title::Single(long)), Some(short)) => {
                    Some(Title::Shorthand(short.clone(), long.clone()))
                }
                _ => r.title.clone(),
            },
            ClassExtension::CollectionComponent(r) => r.title.clone(),
            ClassExtension::SerialComponent(r) => r.title.clone(),
            ClassExtension::Collection(r) => match (&r.title, &r.short_title) {
                (Some(Title::Single(long)), Some(short)) => {
                    Some(Title::Shorthand(short.clone(), long.clone()))
                }
                _ => r.title.clone(),
            },
            ClassExtension::Serial(r) => match (&r.title, &r.short_title) {
                (Some(Title::Single(long)), Some(short)) => {
                    Some(Title::Shorthand(short.clone(), long.clone()))
                }
                _ => r.title.clone(),
            },
            ClassExtension::LegalCase(r) => r.title.clone(),
            ClassExtension::Statute(r) => r.title.clone(),
            ClassExtension::Treaty(r) => r.title.clone(),
            ClassExtension::Hearing(r) => r.title.clone(),
            ClassExtension::Regulation(r) => r.title.clone(),
            ClassExtension::Brief(r) => r.title.clone(),
            ClassExtension::Classic(r) => r.title.clone(),
            ClassExtension::Patent(r) => r.title.clone(),
            ClassExtension::Dataset(r) => r.title.clone(),
            ClassExtension::Standard(r) => r.title.clone(),
            ClassExtension::Software(r) => r.title.clone(),
            ClassExtension::Event(r) => r.title.clone(),
            ClassExtension::AudioVisual(r) => match (&r.core.title, &r.core.short_title) {
                (Some(Title::Single(long)), Some(short)) => {
                    Some(Title::Shorthand(short.clone(), long.clone()))
                }
                _ => r.core.title.clone(),
            },
            ClassExtension::Unknown(data) => data
                .fields
                .get("title")
                .and_then(JsonValue::as_str)
                .map(|title| Title::Single(title.to_string())),
        }
    }

    fn non_empty_date(date: EdtfString) -> Option<EdtfString> {
        if date.is_empty() { None } else { Some(date) }
    }

    /// Return the creation or origination date.
    pub fn created(&self) -> Option<EdtfString> {
        match &self.extension {
            ClassExtension::Monograph(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::CollectionComponent(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::SerialComponent(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Collection(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Serial(_) => None,
            ClassExtension::LegalCase(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Statute(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Treaty(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Hearing(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Regulation(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Brief(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Classic(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Patent(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Dataset(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Standard(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Software(r) => Self::non_empty_date(r.created.clone()),
            ClassExtension::Event(_) => None,
            ClassExtension::AudioVisual(r) => Self::non_empty_date(r.core.created.clone()),
            ClassExtension::Unknown(_) => None,
        }
    }

    /// Return the explicit publication or release date.
    pub fn issued(&self) -> Option<EdtfString> {
        match &self.extension {
            ClassExtension::Monograph(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::CollectionComponent(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::SerialComponent(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Collection(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Serial(_) => None,
            ClassExtension::LegalCase(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Statute(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Treaty(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Hearing(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Regulation(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Brief(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Classic(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Patent(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Dataset(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Standard(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Software(r) => Self::non_empty_date(r.issued.clone()),
            ClassExtension::Event(r) => r.date.clone(),
            ClassExtension::AudioVisual(r) => Self::non_empty_date(r.core.issued.clone()),
            ClassExtension::Unknown(_) => None,
        }
    }

    /// Return the effective issued date used for compatibility layers.
    pub fn csl_issued_date(&self) -> Option<EdtfString> {
        self.issued().or_else(|| self.created())
    }

    /// Return the DOI.
    pub fn doi(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.doi.clone(),
            ClassExtension::CollectionComponent(r) => r.doi.clone(),
            ClassExtension::SerialComponent(r) => r.doi.clone(),
            ClassExtension::LegalCase(r) => r.doi.clone(),
            ClassExtension::Dataset(r) => r.doi.clone(),
            ClassExtension::Software(r) => r.doi.clone(),
            ClassExtension::AudioVisual(_) => None,
            _ => None,
        }
    }

    /// Return the ADS bibcode.
    pub fn ads_bibcode(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.ads_bibcode.clone(),
            ClassExtension::SerialComponent(r) => r.ads_bibcode.clone(),
            ClassExtension::AudioVisual(_) => None,
            _ => None,
        }
    }

    /// Return the note.
    pub fn note(&self) -> Option<RichText> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.note.clone(),
            ClassExtension::CollectionComponent(r) => r.note.clone(),
            ClassExtension::SerialComponent(r) => r.note.clone(),
            ClassExtension::Collection(r) => r.note.clone(),
            ClassExtension::Serial(r) => r.note.clone(),
            ClassExtension::LegalCase(r) => r.note.clone(),
            ClassExtension::Statute(r) => r.note.clone(),
            ClassExtension::Treaty(r) => r.note.clone(),
            ClassExtension::Standard(r) => r.note.clone(),
            ClassExtension::Event(r) => r.note.clone(),
            ClassExtension::AudioVisual(r) => r.note.clone(),
            _ => None,
        }
    }

    /// Return the URL.
    pub fn url(&self) -> Option<Url> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.url.clone(),
            ClassExtension::CollectionComponent(r) => r.url.clone(),
            ClassExtension::SerialComponent(r) => r.url.clone(),
            ClassExtension::Collection(r) => r.url.clone(),
            ClassExtension::Serial(r) => r.url.clone(),
            ClassExtension::LegalCase(r) => r.url.clone(),
            ClassExtension::Statute(r) => r.url.clone(),
            ClassExtension::Treaty(r) => r.url.clone(),
            ClassExtension::Hearing(r) => r.url.clone(),
            ClassExtension::Regulation(r) => r.url.clone(),
            ClassExtension::Brief(r) => r.url.clone(),
            ClassExtension::Classic(r) => r.url.clone(),
            ClassExtension::Patent(r) => r.url.clone(),
            ClassExtension::Dataset(r) => r.url.clone(),
            ClassExtension::Standard(r) => r.url.clone(),
            ClassExtension::Software(r) => r.url.clone(),
            ClassExtension::Event(r) => r.url.clone(),
            ClassExtension::AudioVisual(r) => r.url.clone(),
            ClassExtension::Unknown(_) => None,
        }
    }

    /// Return the publisher place.
    pub fn publisher_place(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r
                .publisher
                .as_ref()
                .and_then(|p| p.place.clone().map(Into::into))
                .or_else(|| {
                    r.container.as_ref().and_then(|c| match c {
                        WorkRelation::Embedded(p) => p.publisher_place(),
                        _ => None,
                    })
                }),
            ClassExtension::CollectionComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                _ => None,
            }),
            ClassExtension::SerialComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                _ => None,
            }),
            ClassExtension::Collection(r) => r
                .publisher
                .as_ref()
                .and_then(|p| p.place.clone().map(Into::into))
                .or_else(|| {
                    r.container.as_ref().and_then(|c| match c {
                        WorkRelation::Embedded(p) => p.publisher_place(),
                        _ => None,
                    })
                }),
            ClassExtension::Serial(_) => None,
            ClassExtension::Classic(r) => r
                .publisher
                .as_ref()
                .and_then(|p| p.place.clone().map(Into::into)),
            ClassExtension::Dataset(r) => r
                .publisher
                .as_ref()
                .and_then(|p| p.place.clone().map(Into::into)),
            ClassExtension::Standard(r) => r
                .publisher
                .as_ref()
                .and_then(|p| p.place.clone().map(Into::into)),
            ClassExtension::Software(r) => r
                .publisher
                .as_ref()
                .and_then(|p| p.place.clone().map(Into::into)),
            ClassExtension::Event(r) => r.location.clone(),
            ClassExtension::AudioVisual(r) => r
                .publisher
                .as_ref()
                .and_then(|p| p.place.clone().map(Into::into)),
            _ => None,
        }
    }

    /// Return the publisher as a string.
    pub fn publisher_str(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r
                .publisher
                .as_ref()
                .map(|p| p.name.to_string())
                .or_else(|| {
                    r.container.as_ref().and_then(|c| match c {
                        WorkRelation::Embedded(p) => p.publisher_str(),
                        _ => None,
                    })
                }),
            ClassExtension::CollectionComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str(),
                _ => None,
            }),
            ClassExtension::SerialComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str(),
                _ => None,
            }),
            ClassExtension::Collection(r) => r
                .publisher
                .as_ref()
                .map(|p| p.name.to_string())
                .or_else(|| {
                    r.container.as_ref().and_then(|c| match c {
                        WorkRelation::Embedded(p) => p.publisher_str(),
                        _ => None,
                    })
                }),
            ClassExtension::Serial(r) => r.publisher.as_ref().map(|p| p.name.to_string()),
            ClassExtension::Classic(r) => r.publisher.as_ref().map(|p| p.name.to_string()),
            ClassExtension::Dataset(r) => r.publisher.as_ref().map(|p| p.name.to_string()),
            ClassExtension::Standard(r) => r.publisher.as_ref().map(|p| p.name.to_string()),
            ClassExtension::Software(r) => r.publisher.as_ref().map(|p| p.name.to_string()),
            ClassExtension::Event(r) => r.network.clone(),
            ClassExtension::AudioVisual(r) => r.publisher.as_ref().map(|p| p.name.to_string()),
            _ => None,
        }
    }

    /// Normalize genre/medium values to canonical kebab-case (defensive fallback for legacy producers).
    ///
    /// Converts to ASCII lowercase and replaces whitespace/underscores with dashes.
    fn normalize_genre_medium(s: &str) -> String {
        let lower = s.to_ascii_lowercase();
        lower
            .split(|c: char| c.is_whitespace() || c == '_')
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Return the genre/type as string, normalized to canonical kebab-case.
    pub fn genre(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => {
                r.genre.as_ref().map(|g| Self::normalize_genre_medium(g))
            }
            ClassExtension::CollectionComponent(r) => {
                r.genre.as_ref().map(|g| Self::normalize_genre_medium(g))
            }
            ClassExtension::SerialComponent(r) => {
                r.genre.as_ref().map(|g| Self::normalize_genre_medium(g))
            }
            ClassExtension::Event(r) => r.genre.as_ref().map(|g| Self::normalize_genre_medium(g)),
            ClassExtension::AudioVisual(r) => r
                .core
                .genre
                .as_ref()
                .map(|g| Self::normalize_genre_medium(g)),
            _ => None,
        }
    }

    /// Return the archive or repository name.
    pub fn archive(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.archive.clone(),
            _ => None,
        }
    }

    /// Return the archive shelfmark or repository location.
    pub fn archive_location(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r
                .archive_info
                .as_ref()
                .and_then(|info| info.location.clone())
                .or_else(|| r.archive_location.clone()),
            ClassExtension::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.location.clone())
            }
            ClassExtension::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.location.clone())
            }
            _ => None,
        }
    }

    /// Return the archive name from structured ArchiveInfo.
    pub fn archive_name(&self) -> Option<MultilingualString> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.name.clone()),
            ClassExtension::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.name.clone())
            }
            ClassExtension::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.name.clone())
            }
            _ => None,
        }
    }

    /// Return the archive geographic place from structured ArchiveInfo.
    pub fn archive_place(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r
                .archive_info
                .as_ref()
                .and_then(|i| i.place.clone().map(Into::into)),
            ClassExtension::CollectionComponent(r) => r
                .archive_info
                .as_ref()
                .and_then(|i| i.place.clone().map(Into::into)),
            ClassExtension::SerialComponent(r) => r
                .archive_info
                .as_ref()
                .and_then(|i| i.place.clone().map(Into::into)),
            _ => None,
        }
    }

    /// Return the archive collection name from structured ArchiveInfo.
    pub fn archive_collection(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => {
                r.archive_info.as_ref().and_then(|i| i.collection.clone())
            }
            ClassExtension::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.collection.clone())
            }
            ClassExtension::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.collection.clone())
            }
            _ => None,
        }
    }

    /// Return the archive collection identifier from structured ArchiveInfo.
    pub fn archive_collection_id(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r
                .archive_info
                .as_ref()
                .and_then(|i| i.collection_id.clone()),
            ClassExtension::CollectionComponent(r) => r
                .archive_info
                .as_ref()
                .and_then(|i| i.collection_id.clone()),
            ClassExtension::SerialComponent(r) => r
                .archive_info
                .as_ref()
                .and_then(|i| i.collection_id.clone()),
            _ => None,
        }
    }

    /// Return the archive series from structured ArchiveInfo.
    pub fn archive_series(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.series.clone()),
            ClassExtension::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.series.clone())
            }
            ClassExtension::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.series.clone())
            }
            _ => None,
        }
    }

    /// Return the archive box number from structured ArchiveInfo.
    pub fn archive_box(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.r#box.clone()),
            ClassExtension::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.r#box.clone())
            }
            ClassExtension::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.r#box.clone())
            }
            _ => None,
        }
    }

    /// Return the archive folder from structured ArchiveInfo.
    pub fn archive_folder(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.folder.clone()),
            ClassExtension::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.folder.clone())
            }
            ClassExtension::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.folder.clone())
            }
            _ => None,
        }
    }

    /// Return the archive item identifier from structured ArchiveInfo.
    pub fn archive_item(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.item.clone()),
            ClassExtension::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.item.clone())
            }
            ClassExtension::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.item.clone())
            }
            _ => None,
        }
    }

    /// Return the archive URL from structured ArchiveInfo.
    pub fn archive_url(&self) -> Option<Url> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.archive_info.as_ref().and_then(|i| i.url.clone()),
            ClassExtension::CollectionComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.url.clone())
            }
            ClassExtension::SerialComponent(r) => {
                r.archive_info.as_ref().and_then(|i| i.url.clone())
            }
            _ => None,
        }
    }

    /// Return the publication status.
    pub fn status(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.status.clone(),
            ClassExtension::CollectionComponent(r) => r.status.clone(),
            ClassExtension::SerialComponent(r) => r.status.clone(),
            ClassExtension::Standard(r) => r.status.clone(),
            _ => None,
        }
    }

    /// Return the eprint identifier.
    pub fn eprint_id(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.eprint.as_ref().map(|e| e.id.clone()),
            ClassExtension::CollectionComponent(r) => r.eprint.as_ref().map(|e| e.id.clone()),
            ClassExtension::SerialComponent(r) => r.eprint.as_ref().map(|e| e.id.clone()),
            _ => None,
        }
    }

    /// Return the eprint server name.
    pub fn eprint_server(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.eprint.as_ref().map(|e| e.server.clone()),
            ClassExtension::CollectionComponent(r) => r.eprint.as_ref().map(|e| e.server.clone()),
            ClassExtension::SerialComponent(r) => r.eprint.as_ref().map(|e| e.server.clone()),
            _ => None,
        }
    }

    /// Return the eprint subject class.
    pub fn eprint_class(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.eprint.as_ref().and_then(|e| e.class.clone()),
            ClassExtension::CollectionComponent(r) => {
                r.eprint.as_ref().and_then(|e| e.class.clone())
            }
            ClassExtension::SerialComponent(r) => r.eprint.as_ref().and_then(|e| e.class.clone()),
            _ => None,
        }
    }

    /// Return the medium, normalized to canonical kebab-case.
    pub fn medium(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => {
                r.medium.as_ref().map(|m| Self::normalize_genre_medium(m))
            }
            ClassExtension::CollectionComponent(r) => {
                r.medium.as_ref().map(|m| Self::normalize_genre_medium(m))
            }
            ClassExtension::SerialComponent(r) => {
                r.medium.as_ref().map(|m| Self::normalize_genre_medium(m))
            }
            ClassExtension::AudioVisual(r) => {
                r.medium.as_ref().map(|m| Self::normalize_genre_medium(m))
            }
            _ => None,
        }
    }

    /// Return the version.
    pub fn version(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Dataset(r) => r.version.clone(),
            ClassExtension::Software(r) => r.version.clone(),
            _ => None,
        }
    }

    /// Return the abstract.
    pub fn abstract_text(&self) -> Option<RichText> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.abstract_text.clone(),
            ClassExtension::CollectionComponent(r) => r.abstract_text.clone(),
            ClassExtension::SerialComponent(r) => r.abstract_text.clone(),
            _ => None,
        }
    }

    /// Return the container-style title for parent works, reporters, or codes.
    pub fn container_title(&self) -> Option<Title> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title().or_else(|| p.container_title()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::CollectionComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title().or_else(|| p.container_title()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::SerialComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title().or_else(|| p.container_title()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Serial(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title().or_else(|| p.container_title()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::LegalCase(r) => r.reporter.clone().map(Title::Single),
            ClassExtension::Statute(r) => r.code.clone().map(Title::Single),
            ClassExtension::Treaty(r) => r.reporter.clone().map(Title::Single),
            ClassExtension::Event(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title().or_else(|| p.container_title()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::AudioVisual(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title().or_else(|| p.container_title()),
                WorkRelation::Id(_) => None,
            }),
            _ => None,
        }
    }

    /// Return the volume.
    pub fn volume(&self) -> Option<NumOrStr> {
        match &self.extension {
            ClassExtension::Monograph(r) => r
                .volume
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Volume))
                .map(NumOrStr::Str),
            ClassExtension::Collection(r) => r
                .volume
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Volume))
                .map(NumOrStr::Str),
            ClassExtension::CollectionComponent(r) => r
                .volume
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Volume))
                .map(NumOrStr::Str),
            ClassExtension::SerialComponent(r) => r
                .volume
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Volume))
                .map(NumOrStr::Str),
            ClassExtension::Classic(r) => r
                .volume
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Volume))
                .map(NumOrStr::Str),
            ClassExtension::LegalCase(r) => r.volume.clone().map(NumOrStr::Str),
            ClassExtension::Statute(r) => r.volume.clone().map(NumOrStr::Str),
            ClassExtension::Treaty(r) => r.volume.clone().map(NumOrStr::Str),
            ClassExtension::Regulation(r) => r.volume.clone().map(NumOrStr::Str),
            _ => self
                .find_numbering(NumberingType::Volume)
                .map(NumOrStr::Str),
        }
    }

    /// Return the collection number (series number).
    pub fn collection_number(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::CollectionComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.collection_number(),
                WorkRelation::Id(_) => None,
            }),
            _ => self.find_numbering(NumberingType::Volume),
        }
    }

    /// Return the issue.
    pub fn issue(&self) -> Option<NumOrStr> {
        match &self.extension {
            ClassExtension::Monograph(r) => r
                .issue
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Issue))
                .map(NumOrStr::Str),
            ClassExtension::Collection(r) => r
                .issue
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Issue))
                .map(NumOrStr::Str),
            ClassExtension::CollectionComponent(r) => r
                .issue
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Issue))
                .map(NumOrStr::Str),
            ClassExtension::SerialComponent(r) => r
                .issue
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Issue))
                .map(NumOrStr::Str),
            ClassExtension::Classic(r) => r
                .issue
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Issue))
                .map(NumOrStr::Str),
            _ => self.find_numbering(NumberingType::Issue).map(NumOrStr::Str),
        }
    }

    /// Return the pages.
    pub fn pages(&self) -> Option<NumOrStr> {
        match &self.extension {
            ClassExtension::CollectionComponent(r) => r.pages.clone(),
            ClassExtension::SerialComponent(r) => r.pages.clone().map(NumOrStr::Str),
            ClassExtension::LegalCase(r) => r.page.clone().map(NumOrStr::Str),
            ClassExtension::Statute(r) => r.page.clone().map(NumOrStr::Str),
            ClassExtension::Treaty(r) => r.page.clone().map(NumOrStr::Str),
            _ => None,
        }
    }

    /// Return the authority (court, legislative body, standards org, etc.).
    pub fn authority(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::LegalCase(r) => r.authority.clone(),
            ClassExtension::Statute(r) => r.authority.clone(),
            ClassExtension::Hearing(r) => r.authority.clone(),
            ClassExtension::Regulation(r) => r.authority.clone(),
            ClassExtension::Brief(r) => r.authority.clone(),
            ClassExtension::Patent(r) => r.authority.clone(),
            ClassExtension::Standard(r) => r.authority.clone(),
            _ => None,
        }
    }

    /// Return the reporter (legal reporter series).
    pub fn reporter(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::LegalCase(r) => r.reporter.clone(),
            ClassExtension::Treaty(r) => r.reporter.clone(),
            _ => None,
        }
    }

    /// Return the code (legal code abbreviation).
    pub fn code(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Statute(r) => r.code.clone(),
            ClassExtension::Regulation(r) => r.code.clone(),
            _ => None,
        }
    }

    /// Return the section (legal section number).
    pub fn section(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Statute(r) => r.section.clone(),
            ClassExtension::Regulation(r) => r.section.clone(),
            ClassExtension::Classic(_) => self.find_numbering(NumberingType::Section),
            _ => None,
        }
    }

    /// Return the generic document number.
    pub fn number(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r
                .number
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Number)),
            ClassExtension::Statute(r) => r.number.clone(),
            ClassExtension::Collection(r) => r
                .number
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Number)),
            ClassExtension::CollectionComponent(r) => r
                .number
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Number)),
            ClassExtension::SerialComponent(r) => r
                .number
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Number)),
            ClassExtension::Classic(r) => r
                .number
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Number)),
            _ => self.find_numbering(NumberingType::Number),
        }
    }

    /// Return the report identifier.
    pub fn report_number(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(_) => self.find_numbering(NumberingType::Report),
            _ => None,
        }
    }

    /// Return the edition.
    pub fn edition(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r
                .edition
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Edition)),
            ClassExtension::Collection(r) => r
                .edition
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Edition)),
            ClassExtension::CollectionComponent(r) => r
                .edition
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Edition)),
            ClassExtension::SerialComponent(r) => r
                .edition
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Edition)),
            ClassExtension::Classic(r) => r
                .edition
                .clone()
                .or_else(|| self.find_numbering(NumberingType::Edition)),
            _ => self.find_numbering(NumberingType::Edition),
        }
    }

    /// Return the accessed date.
    pub fn accessed(&self) -> Option<EdtfString> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.accessed.clone(),
            ClassExtension::CollectionComponent(r) => r.accessed.clone(),
            ClassExtension::SerialComponent(r) => r.accessed.clone(),
            ClassExtension::Collection(r) => r.accessed.clone(),
            ClassExtension::Serial(r) => r.accessed.clone(),
            ClassExtension::LegalCase(r) => r.accessed.clone(),
            ClassExtension::Statute(r) => r.accessed.clone(),
            ClassExtension::Treaty(r) => r.accessed.clone(),
            ClassExtension::Hearing(r) => r.accessed.clone(),
            ClassExtension::Regulation(r) => r.accessed.clone(),
            ClassExtension::Brief(r) => r.accessed.clone(),
            ClassExtension::Classic(r) => r.accessed.clone(),
            ClassExtension::Patent(r) => r.accessed.clone(),
            ClassExtension::Dataset(r) => r.accessed.clone(),
            ClassExtension::Standard(r) => r.accessed.clone(),
            ClassExtension::Software(r) => r.accessed.clone(),
            ClassExtension::Event(r) => r.accessed.clone(),
            ClassExtension::AudioVisual(r) => r.accessed.clone(),
            ClassExtension::Unknown(_) => None,
        }
    }

    /// Return the original publication date.
    pub fn original_date(&self) -> Option<EdtfString> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::CollectionComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::SerialComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::LegalCase(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Statute(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Treaty(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Hearing(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Regulation(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Brief(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Classic(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Patent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Dataset(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Standard(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Software(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Event(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::AudioVisual(r) => r.core.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.csl_issued_date(),
                WorkRelation::Id(_) => None,
            }),
            _ => None,
        }
    }

    /// Return the original title.
    pub fn original_title(&self) -> Option<Title> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::CollectionComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::SerialComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::LegalCase(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Statute(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Treaty(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Hearing(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Regulation(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Brief(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Classic(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Patent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Dataset(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Standard(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Software(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Event(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::AudioVisual(r) => r.core.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.title(),
                WorkRelation::Id(_) => None,
            }),
            _ => None,
        }
    }

    /// Return the original publisher as a string.
    pub fn original_publisher_str(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::CollectionComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::SerialComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::LegalCase(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Statute(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Treaty(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Hearing(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Regulation(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Brief(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Classic(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Patent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Dataset(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Standard(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Software(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Event(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::AudioVisual(r) => r.core.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_str().filter(|value| !value.is_empty()),
                WorkRelation::Id(_) => None,
            }),
            _ => None,
        }
    }

    /// Return the original publisher place.
    pub fn original_publisher_place(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::CollectionComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::SerialComponent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::LegalCase(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Statute(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Treaty(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Hearing(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Regulation(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Brief(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Classic(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Patent(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Dataset(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Standard(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Software(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Event(r) => r.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::AudioVisual(r) => r.core.original.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(p) => p.publisher_place(),
                WorkRelation::Id(_) => None,
            }),
            _ => None,
        }
    }

    /// Return the ISBN.
    pub fn isbn(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.isbn.clone(),
            _ => None,
        }
    }

    /// Return the ISSN.
    pub fn issn(&self) -> Option<String> {
        match &self.extension {
            ClassExtension::SerialComponent(r) => r.container.as_ref().and_then(|c| match c {
                WorkRelation::Embedded(s) => s.issn(),
                WorkRelation::Id(_) => None,
            }),
            ClassExtension::Serial(r) => r.issn.clone(),
            _ => None,
        }
    }

    /// Return the Keywords.
    pub fn keywords(&self) -> Option<Vec<String>> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.keywords.clone(),
            ClassExtension::CollectionComponent(r) => r.keywords.clone(),
            ClassExtension::SerialComponent(r) => r.keywords.clone(),
            ClassExtension::Collection(r) => r.keywords.clone(),
            ClassExtension::Serial(_) => None,
            ClassExtension::LegalCase(r) => r.keywords.clone(),
            ClassExtension::Statute(r) => r.keywords.clone(),
            ClassExtension::Treaty(r) => r.keywords.clone(),
            ClassExtension::Hearing(r) => r.keywords.clone(),
            ClassExtension::Regulation(r) => r.keywords.clone(),
            ClassExtension::Brief(r) => r.keywords.clone(),
            ClassExtension::Classic(r) => r.keywords.clone(),
            ClassExtension::Patent(r) => r.keywords.clone(),
            ClassExtension::Dataset(r) => r.keywords.clone(),
            ClassExtension::Standard(r) => r.keywords.clone(),
            ClassExtension::Software(r) => r.keywords.clone(),
            ClassExtension::Event(_) => None,
            ClassExtension::AudioVisual(_) => None,
            ClassExtension::Unknown(_) => None,
        }
    }

    /// Return the language.
    pub fn language(&self) -> Option<LangID> {
        match &self.extension {
            ClassExtension::Monograph(r) => r.language.clone(),
            ClassExtension::CollectionComponent(r) => r.language.clone(),
            ClassExtension::SerialComponent(r) => r.language.clone(),
            ClassExtension::Collection(r) => r.language.clone(),
            ClassExtension::Serial(r) => r.language.clone(),
            ClassExtension::LegalCase(r) => r.language.clone(),
            ClassExtension::Statute(r) => r.language.clone(),
            ClassExtension::Treaty(r) => r.language.clone(),
            ClassExtension::Hearing(r) => r.language.clone(),
            ClassExtension::Regulation(r) => r.language.clone(),
            ClassExtension::Brief(r) => r.language.clone(),
            ClassExtension::Classic(r) => r.language.clone(),
            ClassExtension::Patent(r) => r.language.clone(),
            ClassExtension::Dataset(r) => r.language.clone(),
            ClassExtension::Standard(r) => r.language.clone(),
            ClassExtension::Software(r) => r.language.clone(),
            ClassExtension::Event(r) => r.language.clone(),
            ClassExtension::AudioVisual(r) => r.core.language.clone(),
            ClassExtension::Unknown(_) => None,
        }
    }

    /// Return field-level language overrides.
    pub fn field_languages(&self) -> &FieldLanguageMap {
        match &self.extension {
            ClassExtension::Monograph(r) => &r.field_languages,
            ClassExtension::CollectionComponent(r) => &r.field_languages,
            ClassExtension::SerialComponent(r) => &r.field_languages,
            ClassExtension::Collection(r) => &r.field_languages,
            ClassExtension::Serial(r) => &r.field_languages,
            ClassExtension::LegalCase(r) => &r.field_languages,
            ClassExtension::Statute(r) => &r.field_languages,
            ClassExtension::Treaty(r) => &r.field_languages,
            ClassExtension::Hearing(r) => &r.field_languages,
            ClassExtension::Regulation(r) => &r.field_languages,
            ClassExtension::Brief(r) => &r.field_languages,
            ClassExtension::Classic(r) => &r.field_languages,
            ClassExtension::Patent(r) => &r.field_languages,
            ClassExtension::Dataset(r) => &r.field_languages,
            ClassExtension::Standard(r) => &r.field_languages,
            ClassExtension::Software(r) => &r.field_languages,
            ClassExtension::Event(r) => &r.field_languages,
            ClassExtension::AudioVisual(r) => &r.field_languages,
            ClassExtension::Unknown(_) => &EMPTY_FIELD_LANGUAGES,
        }
    }

    /// Set the reference ID on the class-specific extension.
    ///
    /// For unknown-class references the id is stored as a `JsonValue::String`
    /// inside `UnknownClassData::fields["id"]`. The wire schema requires
    /// `id: string`, so round-trip is lossless for valid inputs.
    pub fn set_id(&mut self, id: impl Into<RefID>) {
        let id = id.into();
        match &mut self.extension {
            ClassExtension::Monograph(monograph) => monograph.id = Some(id.clone()),
            ClassExtension::CollectionComponent(component) => component.id = Some(id.clone()),
            ClassExtension::SerialComponent(component) => component.id = Some(id.clone()),
            ClassExtension::Collection(collection) => collection.id = Some(id.clone()),
            ClassExtension::Serial(serial) => serial.id = Some(id.clone()),
            ClassExtension::LegalCase(r) => r.id = Some(id.clone()),
            ClassExtension::Statute(r) => r.id = Some(id.clone()),
            ClassExtension::Treaty(r) => r.id = Some(id.clone()),
            ClassExtension::Hearing(r) => r.id = Some(id.clone()),
            ClassExtension::Regulation(r) => r.id = Some(id.clone()),
            ClassExtension::Brief(r) => r.id = Some(id.clone()),
            ClassExtension::Classic(r) => r.id = Some(id.clone()),
            ClassExtension::Patent(r) => r.id = Some(id.clone()),
            ClassExtension::Dataset(r) => r.id = Some(id.clone()),
            ClassExtension::Standard(r) => r.id = Some(id.clone()),
            ClassExtension::Software(r) => r.id = Some(id.clone()),
            ClassExtension::Event(r) => r.id = Some(id.clone()),
            ClassExtension::AudioVisual(r) => r.id = Some(id.clone()),
            ClassExtension::Unknown(data) => {
                data.fields
                    .insert("id".to_string(), JsonValue::String(id.to_string()));
            }
        }
    }

    /// Return the reference type as a string (CSL-compatible).
    #[allow(
        clippy::too_many_lines,
        reason = "Enum dispatch for reference types requires extensive branching"
    )]
    pub fn ref_type(&self) -> String {
        match &self.extension {
            ClassExtension::Monograph(r) => match r.r#type {
                MonographType::Book => {
                    if r.medium
                        .as_deref()
                        .is_some_and(|m| m.to_ascii_lowercase().contains("interview"))
                    {
                        "interview".to_string()
                    } else {
                        "book".to_string()
                    }
                }
                MonographType::Manual => "manual".to_string(),
                MonographType::Report => "report".to_string(),
                MonographType::Thesis => "thesis".to_string(),
                MonographType::Webpage => "webpage".to_string(),
                MonographType::Post => "post".to_string(),
                MonographType::Interview => "interview".to_string(),
                MonographType::Manuscript => "manuscript".to_string(),
                MonographType::Preprint => "preprint".to_string(),
                MonographType::PersonalCommunication => "personal-communication".to_string(),
                MonographType::Document => {
                    if let Some(genre) = r.genre.as_deref()
                        && matches!(genre, "bill-proceeding" | "bill-record")
                    {
                        return genre.to_string();
                    }
                    if self.genre().as_deref() == Some("conference-paper") {
                        return "paper-conference".to_string();
                    }
                    if r.medium
                        .as_deref()
                        .is_some_and(|m| m.to_ascii_lowercase().contains("interview"))
                    {
                        "interview".to_string()
                    } else {
                        "document".to_string()
                    }
                }
            },
            ClassExtension::CollectionComponent(r) => match r.r#type {
                MonographComponentType::Chapter => match r.genre.as_deref() {
                    Some("entry-dictionary") => "entry-dictionary".to_string(),
                    Some("entry-encyclopedia") => "entry-encyclopedia".to_string(),
                    _ => "chapter".to_string(),
                },
                MonographComponentType::Document => "paper-conference".to_string(),
            },
            ClassExtension::SerialComponent(r) => {
                if r.genre.as_deref() == Some("entry-encyclopedia") {
                    return "entry-encyclopedia".to_string();
                }

                let container_type = r.container.as_ref().and_then(|c| match c {
                    WorkRelation::Embedded(p) => Some(p.ref_type()),
                    _ => None,
                });

                match container_type.as_deref() {
                    Some("article-journal") => "article-journal".to_string(),
                    Some("article-magazine") => "article-magazine".to_string(),
                    Some("article-newspaper") => "article-newspaper".to_string(),
                    Some("broadcast") => {
                        if r.genre
                            .as_deref()
                            .is_some_and(|g| g.to_ascii_lowercase().contains("film"))
                        {
                            "motion-picture".to_string()
                        } else {
                            "broadcast".to_string()
                        }
                    }
                    _ => "article-journal".to_string(),
                }
            }
            ClassExtension::Collection(r) => match r.r#type {
                CollectionType::EditedBook => "book".to_string(),
                _ => "collection".to_string(),
            },
            ClassExtension::Serial(r) => match r.r#type {
                SerialType::AcademicJournal => "article-journal".to_string(),
                SerialType::Magazine => "article-magazine".to_string(),
                SerialType::Newspaper => "article-newspaper".to_string(),
                SerialType::BroadcastProgram => "broadcast".to_string(),
                _ => "serial".to_string(),
            },
            ClassExtension::LegalCase(_) => "legal-case".to_string(),
            ClassExtension::Statute(_) => "statute".to_string(),
            ClassExtension::Treaty(_) => "treaty".to_string(),
            ClassExtension::Hearing(_) => "hearing".to_string(),
            ClassExtension::Regulation(_) => "regulation".to_string(),
            ClassExtension::Brief(_) => "brief".to_string(),
            ClassExtension::Classic(_) => "classic".to_string(),
            ClassExtension::Patent(_) => "patent".to_string(),
            ClassExtension::Dataset(_) => "dataset".to_string(),
            ClassExtension::Standard(_) => "standard".to_string(),
            ClassExtension::Software(_) => "software".to_string(),
            ClassExtension::Event(r) => {
                match r
                    .genre
                    .as_deref()
                    .map(|g| g.to_ascii_lowercase())
                    .as_deref()
                {
                    Some(g) if g.contains("conference") || g.contains("paper") => {
                        "paper-conference".to_string()
                    }
                    Some(g) if g.contains("broadcast") => "broadcast".to_string(),
                    Some(g) if g.contains("talk") || g.contains("speech") => "speech".to_string(),
                    _ => "event".to_string(),
                }
            }
            ClassExtension::AudioVisual(r) => match r.r#type {
                AudioVisualType::Film => "motion-picture".to_string(),
                AudioVisualType::Episode => "broadcast".to_string(),
                AudioVisualType::Recording => "song".to_string(),
                AudioVisualType::Broadcast => "broadcast".to_string(),
            },
            ClassExtension::Unknown(data) => {
                // Unknown classes round-trip but cannot route to a known
                // CSL ref-type; the engine has no template branch for the
                // raw class string, so rendering will fall through to the
                // default path (typically empty output).
                //
                // TODO(csl26-1bdr): Layer 5 `CompatibilityWarning` plumbing
                // will surface this as a soft-degrade warning rather than
                // silent fall-through. Until then we return the raw class
                // string so debug builds and logs can identify the value.
                debug_assert!(
                    !ReferenceClass::KNOWN.contains(&data.class.as_str()),
                    "ClassExtension::Unknown should never wrap a known class string (got `{}`)",
                    data.class
                );
                data.class.clone()
            }
        }
    }
}

/// Collects contributors with a given role from a slice of entries.
fn collect_contributors_by_role(
    entries: &[ContributorEntry],
    role: &ContributorRole,
) -> Option<Contributor> {
    use crate::reference::contributor::ContributorList;
    let matching: Vec<&Contributor> = entries
        .iter()
        .filter(|e| &e.role == role)
        .map(|e| &e.contributor)
        .collect();
    match matching.len() {
        0 => None,
        1 =>
        {
            #[allow(clippy::indexing_slicing, reason = "matching.len() == 1")]
            Some(matching[0].clone())
        }
        _ => Some(Contributor::ContributorList(ContributorList(
            matching.into_iter().cloned().collect(),
        ))),
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod discriminator_tests {
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
    fn public_discriminator_rejects_unknown_field_on_known_class() {
        // when an unknown-for-this-class field is present on a known class,
        // then deserialization must fail with a serde-canonical "unknown field"
        // error that names the offending field — not merely a generic schema error.
        let err = parse_reference(
            r#"{
                "class": "legal-case",
                "title": "Smith v. Jones",
                "monograph-type": "book"
            }"#,
        )
        .unwrap_err()
        .to_string();

        assert!(
            err.contains("`monograph-type`"),
            "error must quote the offending field name in backticks, got: {err}"
        );
        assert!(
            err.contains("unknown field") || err.contains("did not match"),
            "error must be a canonical unknown-field/match-failure error, got: {err}"
        );
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
            parse_reference(r#"{ "class": "dance-performance", "id": "orig", "title": "P" }"#)
                .unwrap();

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

        let known_valid =
            r#"{ "class": "monograph", "type": "book", "title": "B", "issued": "2024" }"#;
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
        assert!(
            parse_reference(wrong_class_field).is_err(),
            "known-invalid corpus row must be rejected by the dispatcher"
        );
        assert!(
            schema_text.contains("\"const\":\"legal-case\"")
                && schema_text.contains("\"authority\"")
                && schema_text.contains("\"type\""),
            "schema must expose both relevant branches so unevaluatedProperties can reject cross-class leakage"
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
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod normalize_tests {
    use super::InputReference;

    fn norm(s: &str) -> String {
        InputReference::normalize_genre_medium(s)
    }

    #[test]
    fn test_normalize_genre_medium() {
        assert_eq!(norm("Technical report"), "technical-report");
        assert_eq!(norm("PhD thesis"), "phd-thesis");
        assert_eq!(norm("Short film"), "short-film");
        assert_eq!(norm("video-interview"), "video-interview");
        assert_eq!(norm("film"), "film");
        assert_eq!(norm("Annual report"), "annual-report");
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod numbering_tests {
    use super::{InputReference, NumOrStr, NumberingType};

    fn parse_reference(json: &str) -> InputReference {
        serde_json::from_str(json).expect("reference should parse")
    }

    #[test]
    fn shorthand_numbering_accessors_cover_all_numbered_reference_variants() {
        let cases = [
            (
                "monograph",
                r#"{
                    "class": "monograph",
                    "type": "book",
                    "title": "Book",
                    "issued": "2024",
                    "volume": "1",
                    "issue": "2",
                    "edition": "3",
                    "number": "4"
                }"#,
            ),
            (
                "collection",
                r#"{
                    "class": "collection",
                    "type": "anthology",
                    "title": "Collection",
                    "issued": "2024",
                    "volume": "1",
                    "issue": "2",
                    "edition": "3",
                    "number": "4"
                }"#,
            ),
            (
                "collection-component",
                r#"{
                    "class": "collection-component",
                    "type": "chapter",
                    "title": "Chapter",
                    "issued": "2024",
                    "volume": "1",
                    "issue": "2",
                    "edition": "3",
                    "number": "4"
                }"#,
            ),
            (
                "serial-component",
                r#"{
                    "class": "serial-component",
                    "type": "article",
                    "title": "Article",
                    "issued": "2024",
                    "volume": "1",
                    "issue": "2",
                    "edition": "3",
                    "number": "4"
                }"#,
            ),
            (
                "classic",
                r#"{
                    "class": "classic",
                    "title": "Classic",
                    "issued": "2024",
                    "volume": "1",
                    "issue": "2",
                    "edition": "3",
                    "number": "4"
                }"#,
            ),
        ];

        for (label, json) in cases {
            let reference = parse_reference(json);

            assert_eq!(
                reference.volume(),
                Some(NumOrStr::Str("1".to_string())),
                "{label} should resolve volume"
            );
            assert_eq!(
                reference.issue(),
                Some(NumOrStr::Str("2".to_string())),
                "{label} should resolve issue"
            );
            assert_eq!(
                reference.edition(),
                Some("3".to_string()),
                "{label} should resolve edition"
            );
            assert_eq!(
                reference.number(),
                Some("4".to_string()),
                "{label} should resolve number"
            );
            assert_eq!(
                reference.report_number(),
                None,
                "{label} should not resolve report number"
            );
        }
    }

    #[test]
    fn report_number_accessor_stays_separate_from_generic_number() {
        let reference = parse_reference(
            r#"{
                "class": "monograph",
                "type": "report",
                "title": "Report",
                "issued": "2024",
                "numbering": [
                    { "type": "report", "value": "TR-42" }
                ]
            }"#,
        );

        assert_eq!(reference.number(), None);
        assert_eq!(reference.report_number(), Some("TR-42".to_string()));
    }

    #[test]
    fn numbering_value_accessor_resolves_custom_numbering_without_changing_builtin_accessors() {
        let reference = parse_reference(
            r#"{
                "class": "monograph",
                "type": "book",
                "title": "Score",
                "issued": "2024",
                "numbering": [
                    { "type": "movement", "value": "II" }
                ]
            }"#,
        );

        assert_eq!(
            reference.numbering_value(&NumberingType::Custom("movement".to_string())),
            Some("II".to_string())
        );
        assert_eq!(reference.number(), None);
        assert_eq!(reference.report_number(), None);
    }

    #[test]
    fn numbering_value_accessor_normalizes_manual_custom_numbering_keys() {
        let reference = parse_reference(
            r#"{
                "class": "monograph",
                "type": "book",
                "title": "Score",
                "issued": "2024",
                "numbering": [
                    { "type": "movement", "value": "II" }
                ]
            }"#,
        );

        assert_eq!(
            reference.numbering_value(&NumberingType::Custom("Movement".to_string())),
            Some("II".to_string())
        );
    }

    #[test]
    fn collection_component_collection_number_bubbles_from_embedded_container() {
        let reference = parse_reference(
            r#"{
                "class": "collection-component",
                "type": "chapter",
                "title": "Chapter",
                "issued": "2024",
                "container": {
                    "class": "collection",
                    "type": "edited-book",
                    "title": "Series",
                    "issued": "2024",
                    "volume": "7"
                }
            }"#,
        );

        assert_eq!(reference.collection_number(), Some("7".to_string()));
    }

    #[test]
    fn collection_component_entry_encyclopedia_genre_preserves_ref_type() {
        let reference = parse_reference(
            r#"{
                "class": "collection-component",
                "type": "chapter",
                "title": "Renaissance Art and Culture",
                "genre": "entry-encyclopedia",
                "issued": "2022",
                "container": {
                    "class": "collection",
                    "type": "edited-book",
                    "title": "Encyclopedia of World History",
                    "issued": "2022"
                }
            }"#,
        );

        assert_eq!(reference.ref_type(), "entry-encyclopedia");
    }
}
