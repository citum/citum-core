/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
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
use citum_engine::reference::{Bibliography, Citation};
use citum_schema::Style;

/// Parse a Citum YAML style string, returning a structured error on failure.
fn parse_style(style_yaml: &str) -> Result<Style, String> {
    serde_yaml::from_str(style_yaml).map_err(|e| format!("Style parse error: {e}"))
}

/// Parse a Citum native JSON bibliography string (`{id: Reference, ...}`).
fn parse_bibliography(refs_json: &str) -> Result<Bibliography, String> {
    serde_json::from_str(refs_json).map_err(|e| format!("Bibliography parse error: {e}"))
}

/// Render a single citation to plain text.
///
/// # Arguments
///
/// - `style_yaml` — Citum style as a YAML string
/// - `refs_json` — bibliography as a JSON object (`{id: Reference}`) or
///   CSL-JSON array
/// - `citation_json` — a single [`Citation`] as JSON
///
/// # Errors
///
/// Returns an error string if any input fails to parse or rendering fails.
///
/// # Example (Rust)
///
/// ```ignore
/// let result = citum_bindings::render_citation(style_yaml, refs_json, cite_json);
/// ```
#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "renderCitation"))]
pub fn render_citation(
    style_yaml: &str,
    refs_json: &str,
    citation_json: &str,
) -> Result<String, String> {
    let style = parse_style(style_yaml)?;
    let bib = parse_bibliography(refs_json)?;
    let citation: Citation =
        serde_json::from_str(citation_json).map_err(|e| format!("Citation parse error: {e}"))?;
    let processor = Processor::new(style, bib);
    processor
        .process_citation(&citation)
        .map_err(|e| format!("Render error: {e}"))
}

/// Render a full bibliography to plain text.
///
/// # Arguments
///
/// - `style_yaml` — Citum style as a YAML string
/// - `refs_json` — bibliography as a JSON object or CSL-JSON array
///
/// # Errors
///
/// Returns an error string if any input fails to parse or rendering fails.
#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "renderBibliography"))]
pub fn render_bibliography(style_yaml: &str, refs_json: &str) -> Result<String, String> {
    let style = parse_style(style_yaml)?;
    let bib = parse_bibliography(refs_json)?;
    let processor = Processor::new(style, bib);
    Ok(processor.render_bibliography())
}

/// Validate a Citum style string.
///
/// Checks that the style parses correctly against the Citum schema. Returns
/// `Ok(())` when valid, or `Err` with a list of error messages when not.
///
/// # Errors
///
/// Returns an error string when the YAML cannot be parsed into a valid Citum
/// style.
///
/// # Example (JS/WASM)
///
/// ```js
/// try {
///   validateStyle(yamlString);
/// } catch (e) {
///   console.error("Style errors:", e.message);
/// }
/// ```
#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "validateStyle"))]
pub fn validate_style(style_yaml: &str) -> Result<(), String> {
    serde_yaml::from_str::<Style>(style_yaml)
        .map(|_| ())
        .map_err(|e| format!("Style parse error: {e}"))
}
