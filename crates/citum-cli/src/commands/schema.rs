/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! `schema` subcommand: emit JSON schemas for Citum data types and RPC methods.

use super::CliResult;
use crate::args::{SchemaArgs, SchemaType};
use citum_schema::InputBibliography;
use citum_schema::Style;
use citum_schema::locale::RawLocale;
use citum_schema_data::AbbreviationMap;
use citum_server::rpc::{
    FormatDocumentParams, RenderBibliographyParams, RenderCitationParams, ValidateStyleParams,
};
use schemars::schema_for;
use std::fs;

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
    Ok(doc)
}

pub(super) fn run_schema(args: SchemaArgs) -> CliResult {
    if let Some(dir) = args.out_dir {
        fs::create_dir_all(&dir)?;
        let server_schema = build_server_schema()?;
        let schemas: &[(&str, serde_json::Value)] = &[
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
        ];
        for (name, schema) in schemas {
            let filename = format!("{name}.json");
            let path = dir.join(&filename);
            fs::write(&path, serde_json::to_string_pretty(&schema)?)?;
        }
        println!("Schemas exported to {}", dir.display());
        return Ok(());
    }

    if let Some(t) = args.r#type {
        let schema: serde_json::Value = match t {
            SchemaType::Style => serde_json::to_value(schema_for!(Style))?,
            SchemaType::Bib => serde_json::to_value(schema_for!(InputBibliography))?,
            SchemaType::Locale => serde_json::to_value(schema_for!(RawLocale))?,
            SchemaType::Citation => serde_json::to_value(schema_for!(citum_schema::Citations))?,
            SchemaType::Registry => serde_json::to_value(schema_for!(citum_schema::StyleRegistry))?,
            SchemaType::AbbrevMap => serde_json::to_value(schema_for!(AbbreviationMap))?,
            SchemaType::Server => build_server_schema()?,
        };
        println!("{}", serde_json::to_string_pretty(&schema)?);
        return Ok(());
    }

    Err(
        "Specify a schema type (style, bib, locale, citation, registry, abbrev-map, server) or --out-dir"
            .into(),
    )
}
