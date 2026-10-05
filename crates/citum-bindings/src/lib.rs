/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Citum cross-language bindings.
//!
//! Exposes a minimal, stable public API over `citum-engine` for use from
//! JavaScript/WASM and other languages via cdylib. No internal engine types
//! cross the public boundary — all inputs and outputs are plain strings.
//!
//! # WASM
//!
//! Enable the `wasm` feature to compile with `wasm-bindgen` annotations:
//!
//! ```toml
//! citum-bindings = { features = ["wasm"] }
//! ```
//!
//! # API
//!
//! Three functions cover the common citation management use cases:
//!
//! - [`render_citation`] — render a single citation
//! - [`render_bibliography`] — render a full bibliography
//! - [`validate_style`] — check a style for parse/schema errors

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use citum_engine::processor::Processor;
use citum_engine::render::html::Html as HtmlRenderer;
use citum_engine::{Citation, Reference, StyleInput, resolve_embedded_locale};
#[cfg(feature = "wasm")]
use citum_engine::{
    CitationOccurrence, CitationOccurrenceItem, DocumentSession, OutputFormatKind, RefsInput,
};
use citum_schema::{BibliographySpec, CitationSpec, Style};
use indexmap::IndexMap;
use serde_json::Value;

/// Load and fully resolve a style from inline YAML or an exact builtin selector.
fn load_style(style_input: &str) -> Result<Style, String> {
    let raw: serde_yaml::Value =
        serde_yaml::from_str(style_input).map_err(|e| format!("Style parse error: {e}"))?;
    let id_key = serde_yaml::Value::String("id".to_string());

    let style = if let Some(mapping) = raw.as_mapping() {
        if let Some(id) = mapping.get(&id_key) {
            if mapping.len() != 1 {
                return Err(
                    "Invalid style selector: top-level `id` must be the only field; use `info.id` for style metadata"
                        .to_string(),
                );
            }
            let id = id.as_str().ok_or_else(|| {
                "Invalid style selector: top-level `id` must be a string".to_string()
            })?;
            load_builtin_style(id)?
        } else {
            Style::from_yaml_str(style_input).map_err(|e| format!("Style parse error: {e}"))?
        }
    } else {
        Style::from_yaml_str(style_input).map_err(|e| format!("Style parse error: {e}"))?
    };

    resolve_style(style)
}

fn resolve_style(mut style: Style) -> Result<Style, String> {
    style = style
        .try_into_resolved()
        .map_err(|e| format!("Style resolution error: {e}"))?;
    style.extends = None;
    style.extends_pin = None;
    Ok(style)
}

fn load_builtin_style(id: &str) -> Result<Style, String> {
    let registry = citum_schema::embedded::default_registry();
    let entry = registry
        .resolve(id)
        .ok_or_else(|| builtin_style_error(id))?;
    let builtin = entry
        .builtin
        .as_deref()
        .ok_or_else(|| builtin_style_error(id))?;
    citum_schema::embedded::get_embedded_style(builtin)
        .ok_or_else(|| builtin_style_error(id))?
        .map_err(|e| format!("Bundled style '{id}' is invalid: {e}"))
}

fn builtin_style_error(id: &str) -> String {
    format!(
        "Style ID '{id}' is not bundled with @citum/engine; fetch version-pinned style YAML and pass the YAML content instead"
    )
}

fn materialize_template_references(style: &mut Style) {
    if let Some(citation) = style.citation.as_mut() {
        materialize_citation_template_references(citation);
    }
    if let Some(bibliography) = style.bibliography.as_mut() {
        materialize_bibliography_template_reference(bibliography);
    }
}

fn materialize_citation_template_references(citation: &mut CitationSpec) {
    if citation.template.is_none() {
        citation.template = citation.resolve_template().map(Into::into);
    }
    citation.template_ref = None;
    for child in [
        citation.integral.as_deref_mut(),
        citation.non_integral.as_deref_mut(),
        citation.subsequent.as_deref_mut(),
        citation.ibid.as_deref_mut(),
    ]
    .into_iter()
    .flatten()
    {
        materialize_citation_template_references(child);
    }
}

fn materialize_bibliography_template_reference(bibliography: &mut BibliographySpec) {
    if bibliography.template.is_none() {
        bibliography.template = bibliography.resolve_template().map(Into::into);
    }
    bibliography.template_ref = None;
}

fn processor_with_style_locale(
    style: Style,
    references: IndexMap<String, Reference>,
) -> Result<Processor, String> {
    let (locale, warnings) = resolve_embedded_locale(&style, None);
    if !warnings.is_empty() {
        let messages = warnings
            .into_iter()
            .map(|warning| warning.message)
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("Locale resolution error: {messages}"));
    }
    Ok(Processor::with_locale(style, references, locale))
}

/// Parse a single reference `Value`, upgrading legacy CSL-JSON to the Citum schema when possible.
fn parse_single_reference_value(
    key: &str,
    val: Value,
) -> Result<citum_schema::reference::InputReference, String> {
    if let Ok(r) = serde_json::from_value::<citum_schema::reference::InputReference>(val.clone()) {
        return Ok(r);
    }

    #[cfg(feature = "legacy-convert")]
    {
        if let Ok(legacy) = serde_json::from_value::<csl_legacy::csl_json::Reference>(val) {
            let r: citum_schema::reference::InputReference = legacy.into();
            return Ok(r);
        }
    }

    #[cfg(not(feature = "legacy-convert"))]
    {
        Err(format!("Failed to parse reference '{key}'"))
    }

    #[cfg(feature = "legacy-convert")]
    {
        Err(format!(
            "Failed to parse reference '{key}' as InputReference or CSL-JSON"
        ))
    }
}

/// Parse references JSON, accepting either an object map or an array, upgrading
/// legacy CSL-JSON to the Citum schema when possible.
fn parse_references(refs_json: &str) -> Result<IndexMap<String, Reference>, String> {
    let json_value: Value =
        serde_json::from_str(refs_json).map_err(|e| format!("Invalid JSON for references: {e}"))?;

    let mut mapped: IndexMap<String, Reference> = IndexMap::new();

    match json_value {
        Value::Object(obj) => {
            for (key, val) in obj {
                let r = parse_single_reference_value(&key, val)?;
                mapped.insert(key, r);
            }
        }
        Value::Array(arr) => {
            for (idx, val) in arr.into_iter().enumerate() {
                let Some(obj) = val.as_object() else {
                    return Err(format!(
                        "Failed to parse reference at index {idx}: expected an object"
                    ));
                };

                let id = match obj.get("id").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => {
                        return Err(format!(
                            "Failed to parse reference at index {}: missing string 'id' field",
                            idx
                        ));
                    }
                };

                // Use the entire element value for parsing, keyed by its `id`.
                let r = parse_single_reference_value(&id, Value::Object(obj.clone()))?;
                mapped.insert(id.clone(), r);
            }
        }
        _ => {
            return Err(
                "Invalid JSON for references: expected an object map or an array of objects"
                    .to_string(),
            );
        }
    }
    Ok(mapped)
}

/// Extract the `info` block from a YAML style string as JSON.
///
/// # Errors
///
/// Returns a string error if the YAML fails to parse or the info block cannot
/// be serialized to JSON.
#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "getStyleMetadata"))]
pub fn get_style_metadata(style_yaml: &str) -> Result<String, String> {
    let style = load_style(style_yaml)?;
    serde_json::to_string(&style.info).map_err(|e| format!("Serialization error: {e}"))
}

/// Materialize all template presets in a style and return the updated YAML.
///
/// # Errors
///
/// Returns a string error if the input YAML fails to parse or the materialized
/// style cannot be serialized back to YAML.
#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "materializeStyle"))]
pub fn materialize_style(style_yaml: &str) -> Result<String, String> {
    let mut style = load_style(style_yaml)?;
    materialize_template_references(&mut style);
    use serde_yaml;
    serde_yaml::to_string(&style).map_err(|e| format!("YAML serialization error: {e}"))
}

/// Render a single citation to HTML.
///
/// - `style_yaml` — Citum style as YAML
/// - `refs_json` — bibliography as JSON object (`{id: Reference}`) or CSL-JSON array
/// - `citation_json` — a single [`Citation`] as JSON
/// - `mode` — optional mode override (e.g. `"Integral"`)
///
/// # Errors
///
/// Returns a string error on style/reference/citation parse failure, invalid
/// mode string, or engine rendering error.
#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "renderCitation"))]
pub fn render_citation(
    style_yaml: &str,
    refs_json: &str,
    citation_json: &str,
    mode: Option<String>,
) -> Result<String, String> {
    let style = load_style(style_yaml)?;
    if style.citation.is_none() {
        return Err(
            "Style has no citation section; renderCitation cannot produce output".to_string(),
        );
    }
    let refs = parse_references(refs_json)?;
    let mut citation: Citation =
        serde_json::from_str(citation_json).map_err(|e| format!("Citation parse error: {e}"))?;
    if let Some(m) = mode {
        let m_enum =
            serde_json::from_str::<citum_schema::citation::CitationMode>(&format!("\"{m}\""))
                .map_err(|e| format!("Invalid citation mode '{m}': {e}"))?;
        citation.mode = m_enum;
    }
    let processor = processor_with_style_locale(style, refs)?;
    let mut run = processor.begin_run();
    processor
        .process_citation_with_format::<HtmlRenderer>(&citation, &mut run)
        .map_err(|e| format!("Render error: {e}"))
}

/// Render a full bibliography to HTML.
///
/// - `style_yaml` — Citum style as YAML
/// - `refs_json` — bibliography as JSON object or CSL-JSON array
///
/// # Errors
///
/// Returns a string error on style or reference parse failure.
#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "renderBibliography"))]
pub fn render_bibliography(style_yaml: &str, refs_json: &str) -> Result<String, String> {
    let style = load_style(style_yaml)?;
    if style.bibliography.is_none() {
        return Err(
            "Style has no bibliography section; renderBibliography cannot produce output"
                .to_string(),
        );
    }
    let refs = parse_references(refs_json)?;
    let processor = processor_with_style_locale(style, refs)?;
    Ok(processor.render_bibliography_with_format_standalone::<HtmlRenderer>())
}

/// Validate a Citum style string.
///
/// # Errors
///
/// Returns a string error describing the parse or schema validation failure.
#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "validateStyle"))]
pub fn validate_style(style_yaml: &str) -> Result<(), String> {
    load_style(style_yaml).map(|_| ())
}

/// Format a complete document's citations and bibliography in one call.
///
/// Takes a JSON-encoded `FormatDocumentRequest` and returns a JSON-encoded
/// `FormatDocumentResult`. Bundled style IDs and aliases resolve offline.
/// URI and path inputs are unavailable in WASM; callers should fetch or read
/// version-pinned YAML and pass it as `StyleInput::Yaml`.
///
/// # Errors
///
/// Returns a string error on request JSON parse failure, style resolution failure,
/// or engine rendering error.
#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "formatDocument"))]
pub fn format_document(request_json: &str) -> Result<String, String> {
    let request: citum_engine::FormatDocumentRequest =
        serde_json::from_str(request_json).map_err(|e| format!("Invalid request JSON: {}", e))?;
    let style = match &request.style {
        StyleInput::Id(id) => resolve_style(load_builtin_style(id)?)?,
        StyleInput::Yaml(yaml) => load_style(yaml)?,
        StyleInput::Uri(uri) => {
            return Err(format!(
                "Style URI '{uri}' cannot be fetched by @citum/engine; fetch version-pinned YAML and pass it with kind 'yaml'"
            ));
        }
        StyleInput::Path(path) => {
            return Err(format!(
                "Style path '{path}' is unavailable in WASM; read the file and pass its contents with kind 'yaml'"
            ));
        }
    };
    let result = citum_engine::format_document_with_style(style, request)
        .map_err(|e| format!("Format error: {}", e))?;
    serde_json::to_string(&result).map_err(|e| format!("Result serialization failed: {}", e))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = "DocumentSession")]
/// WASM wrapper around the engine's stateful document session.
pub struct WasmDocumentSession {
    inner: DocumentSession,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(js_class = "DocumentSession")]
impl WasmDocumentSession {
    /// Create a stateful document session from style YAML or a bundled selector and optional refs JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if the style YAML or refs JSON cannot be parsed.
    #[wasm_bindgen(constructor)]
    pub fn new(style_yaml: &str, refs_json: Option<String>) -> Result<WasmDocumentSession, String> {
        let style = load_style(style_yaml)?;
        let mut inner = DocumentSession::new(
            style,
            StyleInput::Yaml(style_yaml.to_string()),
            None,
            OutputFormatKind::Html,
            None,
        );
        if let Some(refs_json) = refs_json {
            inner
                .put_references(refs_input_from_json(&refs_json)?)
                .map_err(|e| e.to_string())?;
        }
        Ok(Self { inner })
    }

    /// Replace the session's full reference set from a JSON object.
    ///
    /// # Errors
    ///
    /// Returns an error if the refs JSON cannot be parsed.
    pub fn put_references(&mut self, refs_json: &str) -> Result<(), String> {
        self.inner
            .put_references(refs_input_from_json(refs_json)?)
            .map_err(|e| e.to_string())
    }

    /// Replace the full ordered citation list and return a session mutation result JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if citations JSON is invalid, rendering fails, or the
    /// result cannot be serialized.
    pub fn insert_citations_batch(&mut self, citations_json: &str) -> Result<String, String> {
        let citations: Vec<CitationOccurrence> = serde_json::from_str(citations_json)
            .map_err(|e| format!("Invalid citations JSON: {e}"))?;
        let result = self
            .inner
            .insert_citations_batch(citations)
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&result).map_err(|e| format!("Result serialization failed: {e}"))
    }

    /// Insert one citation and return a session mutation result JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if request JSON is invalid, rendering fails, or the
    /// result cannot be serialized.
    pub fn insert_citation(
        &mut self,
        citation_json: &str,
        position_json: Option<String>,
    ) -> Result<String, String> {
        let citation = parse_json(citation_json, "citation")?;
        let position = parse_optional_json(position_json, "position")?;
        let result = self
            .inner
            .insert_citation(citation, position)
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&result).map_err(|e| format!("Result serialization failed: {e}"))
    }

    /// Update one citation and return a session mutation result JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if request JSON is invalid, the citation is missing,
    /// rendering fails, or the result cannot be serialized.
    pub fn update_citation(
        &mut self,
        citation_id: &str,
        citation_json: &str,
        position_json: Option<String>,
    ) -> Result<String, String> {
        let citation = parse_json(citation_json, "citation")?;
        let position = parse_optional_json(position_json, "position")?;
        let result = self
            .inner
            .update_citation(citation_id, citation, position)
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&result).map_err(|e| format!("Result serialization failed: {e}"))
    }

    /// Delete one citation and return a session mutation result JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the citation is missing, rendering fails, or the
    /// result cannot be serialized.
    pub fn delete_citation(&mut self, citation_id: &str) -> Result<String, String> {
        let result = self
            .inner
            .delete_citation(citation_id)
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&result).map_err(|e| format!("Result serialization failed: {e}"))
    }

    /// Render a citation preview without mutating session state.
    ///
    /// # Errors
    ///
    /// Returns an error if request JSON is invalid, rendering fails, or the
    /// result cannot be serialized.
    pub fn preview_citation(
        &self,
        items_json: &str,
        position_json: Option<String>,
    ) -> Result<String, String> {
        let items: Vec<CitationOccurrenceItem> = parse_json(items_json, "items")?;
        let position = parse_optional_json(position_json, "position")?;
        let result = self
            .inner
            .preview_citation(items, None, position)
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&result).map_err(|e| format!("Result serialization failed: {e}"))
    }

    /// Return the current formatted citations as a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the result cannot be serialized.
    pub fn get_citations(&self) -> Result<String, String> {
        serde_json::to_string(&serde_json::json!({
            "formatted_citations": self.inner.get_citations()
        }))
        .map_err(|e| format!("Result serialization failed: {e}"))
    }

    /// Return the current bibliography as a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the result cannot be serialized.
    pub fn get_bibliography(&self) -> Result<String, String> {
        serde_json::to_string(&serde_json::json!({
            "bibliography": self.inner.get_bibliography()
        }))
        .map_err(|e| format!("Result serialization failed: {e}"))
    }

    /// Explicitly dispose of the session object.
    pub fn dispose(self) {}
}

#[cfg(feature = "wasm")]
fn refs_input_from_json(refs_json: &str) -> Result<RefsInput, String> {
    let refs = serde_json::from_str(refs_json).map_err(|e| format!("Invalid refs JSON: {e}"))?;
    Ok(RefsInput::Json(refs))
}

#[cfg(feature = "wasm")]
fn parse_json<T>(json: &str, label: &str) -> Result<T, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    serde_json::from_str(json).map_err(|e| format!("Invalid {label} JSON: {e}"))
}

#[cfg(feature = "wasm")]
fn parse_optional_json<T>(json: Option<String>, label: &str) -> Result<Option<T>, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    json.map(|value| parse_json(&value, label)).transpose()
}

/// Export all Citum schema types as TypeScript type definitions to a file.
///
/// Writes a `.d.ts` file to `out_path` containing type definitions for all
/// annotated public schema types (references, citations, bibliography input).
///
/// Typically called via `citum bindings --out-dir <dir>`, which writes
/// to `<out-dir>/citum.d.ts`.
///
/// # Errors
///
/// Returns a [`specta_typescript::Error`] if the output file cannot be written
/// or a type registration is invalid.
#[cfg(feature = "typescript")]
#[cfg(not(target_arch = "wasm32"))]
pub fn export_typescript(out_path: &std::path::Path) -> Result<(), specta_typescript::Error> {
    use citum_schema::citation::{
        CitationItem, CitationLocator, CitationMode, LocatorType, LocatorValue,
    };
    use citum_schema::reference::{Contributor, InputReference};
    use citum_schema::{Citation, InputBibliography};
    use specta::TypeCollection;
    use specta_typescript::Typescript;

    let types = TypeCollection::default()
        .register::<InputReference>()
        .register::<InputBibliography>()
        .register::<Citation>()
        .register::<CitationItem>()
        .register::<CitationLocator>()
        .register::<CitationMode>()
        .register::<LocatorType>()
        .register::<LocatorValue>()
        .register::<Contributor>();

    Typescript::default().export_to(out_path, &types)
}
