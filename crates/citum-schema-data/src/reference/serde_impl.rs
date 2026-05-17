/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! `Deserialize`/`Serialize`/`JsonSchema` impls for [`InputReference`].
//!
//! Owns the flat-with-discriminator wire format: a `class` string keys
//! dispatch to the typed [`super::ClassExtension`] payload.

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map as JsonMap, Value as JsonValue};

#[cfg(feature = "schema")]
use super::types::legal::{Brief, Hearing, LegalCase, Regulation, Statute, Treaty};
#[cfg(feature = "schema")]
use super::types::specialized::{
    AudioVisualWork, Classic, Dataset, Event, Patent, Software, Standard,
};
#[cfg(feature = "schema")]
use super::types::structural::{
    Collection, CollectionComponent, Monograph, Serial, SerialComponent,
};
use super::{ClassExtension, InputReference, ReferenceClass, UnknownClassData};

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
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::CollectionComponent(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::SerialComponent(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Collection(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Serial(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::LegalCase(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Statute(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Treaty(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Hearing(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Regulation(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Brief(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Classic(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Patent(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Dataset(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Standard(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Software(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::Event(inner) => FlatClassProxy {
                class: self.extension.class_name(),
                inner: inner.as_ref(),
            }
            .serialize(serializer),
            ClassExtension::AudioVisual(inner) => FlatClassProxy {
                class: self.extension.class_name(),
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
            reference_schema_branch::<Monograph>(generator, ReferenceClass::Monograph.name()),
            reference_schema_branch::<CollectionComponent>(
                generator,
                ReferenceClass::CollectionComponent.name(),
            ),
            reference_schema_branch::<SerialComponent>(
                generator,
                ReferenceClass::SerialComponent.name(),
            ),
            reference_schema_branch::<Collection>(generator, ReferenceClass::Collection.name()),
            reference_schema_branch::<Serial>(generator, ReferenceClass::Serial.name()),
            reference_schema_branch::<LegalCase>(generator, ReferenceClass::LegalCase.name()),
            reference_schema_branch::<Statute>(generator, ReferenceClass::Statute.name()),
            reference_schema_branch::<Treaty>(generator, ReferenceClass::Treaty.name()),
            reference_schema_branch::<Hearing>(generator, ReferenceClass::Hearing.name()),
            reference_schema_branch::<Regulation>(generator, ReferenceClass::Regulation.name()),
            reference_schema_branch::<Brief>(generator, ReferenceClass::Brief.name()),
            reference_schema_branch::<Classic>(generator, ReferenceClass::Classic.name()),
            reference_schema_branch::<Patent>(generator, ReferenceClass::Patent.name()),
            reference_schema_branch::<Dataset>(generator, ReferenceClass::Dataset.name()),
            reference_schema_branch::<Standard>(generator, ReferenceClass::Standard.name()),
            reference_schema_branch::<Software>(generator, ReferenceClass::Software.name()),
            reference_schema_branch::<Event>(generator, ReferenceClass::Event.name()),
            reference_schema_branch::<AudioVisualWork>(
                generator,
                ReferenceClass::AudioVisual.name(),
            ),
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
    class: &str,
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
    match ReferenceClass::from_known_name(class) {
        Some(ReferenceClass::Monograph) => {
            InputReference::from_known(ClassExtension::Monograph, value)
        }
        Some(ReferenceClass::CollectionComponent) => {
            InputReference::from_known(ClassExtension::CollectionComponent, value)
        }
        Some(ReferenceClass::SerialComponent) => {
            InputReference::from_known(ClassExtension::SerialComponent, value)
        }
        Some(ReferenceClass::Collection) => {
            InputReference::from_known(ClassExtension::Collection, value)
        }
        Some(ReferenceClass::Serial) => InputReference::from_known(ClassExtension::Serial, value),
        Some(ReferenceClass::LegalCase) => {
            InputReference::from_known(ClassExtension::LegalCase, value)
        }
        Some(ReferenceClass::Statute) => InputReference::from_known(ClassExtension::Statute, value),
        Some(ReferenceClass::Treaty) => InputReference::from_known(ClassExtension::Treaty, value),
        Some(ReferenceClass::Hearing) => InputReference::from_known(ClassExtension::Hearing, value),
        Some(ReferenceClass::Regulation) => {
            InputReference::from_known(ClassExtension::Regulation, value)
        }
        Some(ReferenceClass::Brief) => InputReference::from_known(ClassExtension::Brief, value),
        Some(ReferenceClass::Classic) => InputReference::from_known(ClassExtension::Classic, value),
        Some(ReferenceClass::Patent) => InputReference::from_known(ClassExtension::Patent, value),
        Some(ReferenceClass::Dataset) => InputReference::from_known(ClassExtension::Dataset, value),
        Some(ReferenceClass::Standard) => {
            InputReference::from_known(ClassExtension::Standard, value)
        }
        Some(ReferenceClass::Software) => {
            InputReference::from_known(ClassExtension::Software, value)
        }
        Some(ReferenceClass::Event) => InputReference::from_known(ClassExtension::Event, value),
        Some(ReferenceClass::AudioVisual) => {
            InputReference::from_known(ClassExtension::AudioVisual, value)
        }
        Some(ReferenceClass::Unknown(_)) | None => {
            let fields = if let JsonValue::Object(fields) = value {
                fields
            } else {
                JsonMap::new()
            };
            Ok(InputReference::Unknown(Box::new(UnknownClassData {
                class: class.to_string(),
                fields,
            })))
        }
    }
}
