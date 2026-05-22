/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Public `InputReference` container and unknown-class payload.

use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

#[cfg(feature = "bindings")]
use specta::Type;

use super::classes::ClassExtension;
use super::types::common::FieldLanguageMap;

/// Empty field-language map returned by accessors on unknown-class references.
///
/// `FieldLanguageMap` is a `HashMap`, whose `::new()` is not `const`, so a
/// `LazyLock` is required. The map is constructed once for the process and
/// reused by every unknown-class reference.
pub(crate) static EMPTY_FIELD_LANGUAGES: LazyLock<FieldLanguageMap> =
    LazyLock::new(FieldLanguageMap::new);

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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
pub struct UnknownClassData {
    /// Raw `class:` string from the input object.
    pub class: String,
    /// Non-shared fields captured verbatim for round-trip preservation.
    #[cfg_attr(feature = "bindings", specta(type = serde_json::Value))]
    pub fields: JsonMap<String, JsonValue>,
}
