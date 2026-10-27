/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! `schema` subcommand: emit JSON schemas for Citum data types and RPC methods.

use super::CliResult;
use crate::args::{SchemaArgs, SchemaType};
use citum_io::biblatex::{biblatex_entry_type_descriptors, biblatex_field_descriptors};
use citum_schema::InputBibliography;
use citum_schema::Style;
use citum_schema::locale::RawLocale;
use citum_schema_data::AbbreviationMap;
use citum_schema_data::reference::conversion::CSL_TYPE_MAP;
use citum_server::rpc::{
    FormatDocumentParams, RenderBibliographyParams, RenderCitationParams, ValidateStyleParams,
};
use schemars::schema_for;
use std::fs;

/// Types that carry a flatten-captured `unknown_fields` map for forward-compat
/// parse tolerance. The Rust loaders accept extra keys (SoftDegrade contract,
/// see `docs/specs/FORWARD_COMPATIBILITY.md`), so producers no longer get the
/// `additionalProperties: false` typo catch from `deny_unknown_fields`. We
/// inject `unevaluatedProperties: false` on the generated schema for each so
/// editor validation still flags unknown keys at authoring time.
const TOLERANT_TYPE_NAMES: &[&str] = &[
    "ArticleJournalBibliographyConfig",
    "AudioVisualWork",
    "BibliographyConfig",
    "BibliographyOptions",
    "BibliographySortPartitioning",
    "BibliographySpec",
    "Brief",
    "CitationOptions",
    "CitationSpec",
    "Classic",
    "Collection",
    "CollectionComponent",
    "CompoundNumericConfig",
    "Config",
    "ContributorConfig",
    "Dataset",
    "DateConfig",
    "Event",
    "Hearing",
    "IntegralNameMemoryConfig",
    "LegalCase",
    "LocalizedTemplateSpec",
    "LocatorConfig",
    "LocatorKindConfig",
    "LocatorPattern",
    "Monograph",
    "NoteConfig",
    "Patent",
    "Regulation",
    "Serial",
    "SerialComponent",
    "Software",
    "Standard",
    "Statute",
    "Style",
    "Substitute",
    "TitleRendering",
    "TitlesConfig",
    "Treaty",
];

/// Stamp `unevaluatedProperties: false` on each tolerant object schema so
/// producer-side validation rejects typos that the Rust loader would silently
/// capture into `unknown_fields`.
fn stamp_unevaluated_properties(schema: &mut serde_json::Value) {
    fn stamp_object(obj: &mut serde_json::Map<String, serde_json::Value>) {
        if obj.get("type").and_then(|v| v.as_str()) != Some("object") {
            return;
        }
        if obj.contains_key("unevaluatedProperties") {
            return;
        }
        if obj.get("additionalProperties") == Some(&serde_json::Value::Bool(false)) {
            return;
        }
        obj.insert(
            "unevaluatedProperties".to_string(),
            serde_json::Value::Bool(false),
        );
    }

    if let Some(root_obj) = schema.as_object_mut() {
        let root_is_tolerant = root_obj
            .get("title")
            .and_then(|v| v.as_str())
            .is_some_and(|t| TOLERANT_TYPE_NAMES.contains(&t));
        if root_is_tolerant {
            stamp_object(root_obj);
        }
        if let Some(defs) = root_obj.get_mut("$defs").and_then(|v| v.as_object_mut()) {
            for name in TOLERANT_TYPE_NAMES {
                if let Some(def_obj) = defs.get_mut(*name).and_then(|v| v.as_object_mut()) {
                    stamp_object(def_obj);
                }
            }
            // InputReference is a discriminated union: each `oneOf` branch is a
            // tolerant reference-type struct inlined by schemars. Stamp each so
            // producer-side validation rejects typos on reference payloads.
            if let Some(ir) = defs
                .get_mut("InputReference")
                .and_then(|v| v.as_object_mut())
                && let Some(one_of) = ir.get_mut("oneOf").and_then(|v| v.as_array_mut())
            {
                for branch in one_of {
                    if let Some(branch_obj) = branch.as_object_mut() {
                        branch_obj.remove("additionalProperties");
                        stamp_object(branch_obj);
                    }
                }
            }
        }
    }
}

fn build_server_schema() -> Result<serde_json::Value, serde_json::Error> {
    use serde_json::{Map, Value};
    let methods: &[(&str, Value)] = &[
        (
            "render_citation",
            serde_json::to_value(schema_for!(RenderCitationParams))?,
        ),
        (
            "render_bibliography",
            serde_json::to_value(schema_for!(RenderBibliographyParams))?,
        ),
        (
            "validate_style",
            serde_json::to_value(schema_for!(ValidateStyleParams))?,
        ),
        (
            "format_document",
            serde_json::to_value(schema_for!(FormatDocumentParams))?,
        ),
    ];
    let mut merged_defs: Map<String, Value> = Map::new();
    let mut method_schemas: Vec<(&str, Value)> = Vec::new();
    for (name, schema) in methods {
        let mut schema = schema.clone();
        if let Some(obj) = schema.as_object_mut() {
            obj.remove("$schema");
            if let Some(Value::Object(defs_map)) = obj.remove("$defs") {
                merged_defs.extend(defs_map);
            }
        }
        method_schemas.push((name, schema));
    }
    let mut doc = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Citum Server RPC",
        "description": "Parameter schemas for all Citum JSON-RPC methods.",
    });
    if let Some(obj) = doc.as_object_mut() {
        if !merged_defs.is_empty() {
            obj.insert("$defs".to_string(), Value::Object(merged_defs));
        }
        for (name, schema) in method_schemas {
            obj.insert(name.to_string(), schema);
        }
    }
    annotate_server_schema(&mut doc);
    Ok(doc)
}

/// Add descriptions for inline properties that `schemars` cannot infer from
/// tagged or custom-serialized Rust types.
fn annotate_server_schema(schema: &mut serde_json::Value) {
    let Some(definitions) = schema
        .as_object_mut()
        .and_then(|object| object.get_mut("$defs"))
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };

    annotate_style_input(definitions);
    annotate_sort_key(definitions);
    annotate_wrap_config(definitions);
    annotate_delimiter(definitions);
    annotate_message_arg_source(definitions);
}

fn set_property_description(object: &mut serde_json::Value, property: &str, description: &str) {
    if let Some(properties) = object
        .as_object_mut()
        .and_then(|object| object.get_mut("properties"))
        .and_then(serde_json::Value::as_object_mut)
        && let Some(field) = properties
            .get_mut(property)
            .and_then(serde_json::Value::as_object_mut)
        && !field.contains_key("description")
    {
        field.insert(
            "description".to_string(),
            serde_json::Value::String(description.to_string()),
        );
    }
}

fn annotate_style_input(definitions: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(style_input) = definitions
        .get_mut("StyleInput")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|definition| definition.get_mut("oneOf"))
        .and_then(serde_json::Value::as_array_mut)
    {
        for variant in style_input {
            let kind = variant
                .as_object()
                .and_then(|variant| variant.get("properties"))
                .and_then(serde_json::Value::as_object)
                .and_then(|properties| properties.get("kind"))
                .and_then(serde_json::Value::as_object)
                .and_then(|kind| kind.get("const"))
                .and_then(serde_json::Value::as_str);
            let (kind_description, value_description) = match kind {
                Some("id") => (
                    "Style source selector; `id` resolves through the server's style registry chain.",
                    "Style identifier to resolve, such as `apa-7th`.",
                ),
                Some("uri") => (
                    "Style source selector; `uri` resolves a remote style through the server's resolver chain.",
                    "HTTP(S) URI for the remote YAML style.",
                ),
                Some("path") => (
                    "Style source selector; `path` reads a local YAML style file.",
                    "Filesystem path to the YAML style, resolved by the server.",
                ),
                Some("yaml") => (
                    "Style source selector; `yaml` parses an inline YAML style definition.",
                    "Inline YAML style definition.",
                ),
                _ => continue,
            };
            set_property_description(variant, "kind", kind_description);
            set_property_description(variant, "value", value_description);
        }
    }
}

fn annotate_sort_key(definitions: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(sort_key) = definitions
        .get_mut("SortKey")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|definition| definition.get_mut("oneOf"))
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|variants| variants.last_mut())
    {
        set_property_description(
            sort_key,
            "field",
            "Custom reference field name used as a bibliography sort key.",
        );
    }
}

fn annotate_wrap_config(definitions: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(wrap_config) = definitions
        .get_mut("WrapConfig")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|definition| definition.get_mut("oneOf"))
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|variants| variants.get_mut(1))
    {
        set_property_description(
            wrap_config,
            "inner-prefix",
            "Optional text inserted after the opening punctuation and before the rendered content.",
        );
        set_property_description(
            wrap_config,
            "inner-suffix",
            "Optional text inserted after the rendered content and before the closing punctuation.",
        );
    }
}

fn annotate_delimiter(definitions: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(delimiter) = definitions
        .get_mut("DelimiterPunctuation")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|definition| definition.get_mut("oneOf"))
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|variants| variants.get_mut(1))
    {
        set_property_description(
            delimiter,
            "mark",
            "Semantic punctuation mark to render with locale-aware punctuation rules.",
        );
    }
}

fn annotate_message_arg_source(definitions: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(literal) = definitions
        .get_mut("MessageArgSource")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|definition| definition.get_mut("anyOf"))
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|variants| variants.first_mut())
    {
        set_property_description(
            literal,
            "literal",
            "Literal string passed unchanged to the locale message argument.",
        );
    }
}

/// Build the declarative BibLaTeX/CSL-JSON ↔ Citum mapping data dump.
///
/// Not a JSON Schema (no `schema_for!` involved) — a plain serialization of
/// the mapping consts in `citum-refs::formats::biblatex::tables` and
/// `citum-schema-data::reference::conversion`, consumed by
/// `scripts/build-data-model-reference.js` to generate
/// `docs/reference/BIBLATEX_MAPPING.md` and
/// `docs/reference/generated/CSL_JSON_MAPPING.md`.
fn build_type_map_schema() -> Result<serde_json::Value, serde_json::Error> {
    Ok(serde_json::json!({
        "biblatex_entry_types": biblatex_entry_type_descriptors(),
        "biblatex_fields": biblatex_field_descriptors(),
        "csl_type_map": CSL_TYPE_MAP,
    }))
}

pub(super) fn run_schema(args: SchemaArgs) -> CliResult {
    if let Some(dir) = args.out_dir {
        fs::create_dir_all(&dir)?;
        let server_schema = build_server_schema()?;
        let mut schemas: Vec<(&str, serde_json::Value)> = vec![
            ("style", serde_json::to_value(schema_for!(Style))?),
            ("bib", serde_json::to_value(schema_for!(InputBibliography))?),
            ("locale", serde_json::to_value(schema_for!(RawLocale))?),
            (
                "citation",
                serde_json::to_value(schema_for!(citum_schema::Citations))?,
            ),
            (
                "registry",
                serde_json::to_value(schema_for!(citum_schema::StyleRegistry))?,
            ),
            (
                "abbrev-map",
                serde_json::to_value(schema_for!(AbbreviationMap))?,
            ),
            ("server", server_schema),
            ("type-map", build_type_map_schema()?),
        ];
        for (_, schema) in &mut schemas {
            stamp_unevaluated_properties(schema);
        }
        for (name, schema) in &schemas {
            let filename = format!("{name}.json");
            let path = dir.join(&filename);
            fs::write(&path, serde_json::to_string_pretty(&schema)?)?;
        }
        println!("Schemas exported to {}", dir.display());
        return Ok(());
    }

    if let Some(t) = args.r#type {
        let mut schema: serde_json::Value = match t {
            SchemaType::Style => serde_json::to_value(schema_for!(Style))?,
            SchemaType::Bib => serde_json::to_value(schema_for!(InputBibliography))?,
            SchemaType::Locale => serde_json::to_value(schema_for!(RawLocale))?,
            SchemaType::Citation => serde_json::to_value(schema_for!(citum_schema::Citations))?,
            SchemaType::Registry => serde_json::to_value(schema_for!(citum_schema::StyleRegistry))?,
            SchemaType::AbbrevMap => serde_json::to_value(schema_for!(AbbreviationMap))?,
            SchemaType::Server => build_server_schema()?,
            SchemaType::TypeMap => build_type_map_schema()?,
        };
        stamp_unevaluated_properties(&mut schema);
        println!("{}", serde_json::to_string_pretty(&schema)?);
        return Ok(());
    }

    Err(
        "Specify a schema type (style, bib, locale, citation, registry, abbrev-map, server, type-map) or --out-dir"
            .into(),
    )
}
