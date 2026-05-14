/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use crate::error::ServerError;
use citum_engine::{
    Bibliography, Citation, Processor,
    render::{djot::Djot, html::Html, latex::Latex, plain::PlainText, typst::Typst},
};
use citum_schema::Style;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

/// JSON-RPC request envelope.
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct RpcRequest {
    /// The request identifier echoed back in success and error responses.
    pub id: Value,
    /// The JSON-RPC method name to dispatch.
    pub method: String,
    /// The method-specific parameter object.
    pub params: Value,
}

/// Output format for rendered citations and bibliographies.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub enum OutputFormat {
    /// Plain text output.
    #[default]
    Plain,
    /// HTML output.
    Html,
    /// Djot markup output.
    Djot,
    /// LaTeX output.
    Latex,
    /// Typst output.
    Typst,
}

/// Parameters for the `render_citation` method.
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct RenderCitationParams {
    /// Path to the Citum YAML style file.
    pub style_path: String,
    /// Bibliography (references) as a map of reference objects.
    pub refs: serde_json::Value,
    /// Citation object specifying which references to cite.
    pub citation: serde_json::Value,
    /// Output format for the rendered citation.
    pub output_format: Option<OutputFormat>,
    /// Debug: embed AST node indices in output.
    pub inject_ast_indices: Option<bool>,
}

/// Parameters for the `render_bibliography` method.
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct RenderBibliographyParams {
    /// Path to the Citum YAML style file.
    pub style_path: String,
    /// Bibliography (references) as a map of reference objects.
    pub refs: serde_json::Value,
    /// Output format for the rendered bibliography.
    pub output_format: Option<OutputFormat>,
    /// Debug: embed AST node indices in output.
    pub inject_ast_indices: Option<bool>,
}

/// Parameters for the `validate_style` method.
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct ValidateStyleParams {
    /// Path to the Citum YAML style file to validate.
    pub style_path: String,
}

/// Parameters for the `format_document` method (schema mirror of `FormatDocumentRequest`).
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct FormatDocumentParams {
    /// Style identifier, path, URI, or inline YAML.
    pub style: String,
    /// Optional BCP 47 locale override.
    pub locale: Option<String>,
    /// Output format (plain, html, djot, latex, typst). Defaults to plain.
    pub output_format: Option<OutputFormat>,
    /// Bibliography (references) as a map of reference objects.
    pub refs: serde_json::Value,
    /// Ordered citations as they appear in the document.
    pub citations: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct BibliographyResult {
    format: OutputFormat,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    entries: Option<Vec<String>>,
}

/// Return `MissingField` if `field` is absent from `params`.
fn require_field(params: &Value, field: &'static str) -> Result<(), ServerError> {
    if params.get(field).is_none() {
        return Err(ServerError::MissingField(field.into()));
    }
    Ok(())
}

/// Validate the optional `output_format` field before full deserialization.
fn validate_output_format(params: &Value) -> Result<(), ServerError> {
    if let Some(v) = params.get("output_format") {
        serde_json::from_value::<OutputFormat>(v.clone()).map_err(|_| {
            let raw = v.as_str().unwrap_or("unknown").to_string();
            ServerError::UnsupportedOutputFormat(raw.into())
        })?;
    }
    Ok(())
}

/// Main RPC dispatcher that processes a single request.
///
/// On success, this returns a JSON object containing the original request ID
/// and a method-specific `result` payload. On failure, it returns the request
/// ID when available plus a human-readable error string.
///
/// # Errors
///
/// Returns an error for unknown methods or when request-specific rendering or
/// validation steps fail.
pub fn dispatch(req: RpcRequest) -> Result<Value, (Option<Value>, String)> {
    let id = req.id.clone();

    match req.method.as_str() {
        "render_citation" => {
            render_citation(&req.params, id).map_err(|e| (Some(req.id), e.to_string()))
        }
        "render_bibliography" => {
            render_bibliography(&req.params, id).map_err(|e| (Some(req.id), e.to_string()))
        }
        "validate_style" => {
            validate_style(&req.params, id).map_err(|e| (Some(req.id), e.to_string()))
        }
        "format_document" => {
            format_document(&req.params, id).map_err(|e| (Some(req.id), e.to_string()))
        }
        _ => Err((Some(req.id), format!("unknown method: {}", req.method))),
    }
}

/// Render a single citation.
fn render_citation(params: &Value, id: Value) -> Result<Value, ServerError> {
    require_field(params, "style_path")?;
    require_field(params, "refs")?;
    require_field(params, "citation")?;
    validate_output_format(params)?;
    let params: RenderCitationParams = serde_json::from_value(params.clone())
        .map_err(|e| ServerError::CitationError(e.to_string()))?;

    // Load the style.
    let style = load_style(&params.style_path)?;

    // Deserialize references and citation from JSON.
    let bibliography: Bibliography = serde_json::from_value(params.refs.clone())
        .map_err(|e| ServerError::BibliographyError(e.to_string()))?;

    let citation: Citation = serde_json::from_value(params.citation.clone())
        .map_err(|e| ServerError::CitationError(e.to_string()))?;

    // Create processor and render.
    let mut processor = Processor::new(style, bibliography);
    let inject_ast_indices = params.inject_ast_indices.unwrap_or(false);
    processor.set_inject_ast_indices(inject_ast_indices);

    let output_format = params.output_format.unwrap_or_default();
    let result = render_citation_with_format(&processor, &citation, output_format)
        .map_err(|e| ServerError::CitationError(e.to_string()))?;

    Ok(json!({
        "id": id,
        "result": result
    }))
}

/// Render a bibliography.
fn render_bibliography(params: &Value, id: Value) -> Result<Value, ServerError> {
    require_field(params, "style_path")?;
    require_field(params, "refs")?;
    validate_output_format(params)?;
    let params: RenderBibliographyParams = serde_json::from_value(params.clone())
        .map_err(|e| ServerError::CitationError(e.to_string()))?;

    // Load the style.
    let style = load_style(&params.style_path)?;

    // Deserialize bibliography from JSON.
    let bibliography: Bibliography = serde_json::from_value(params.refs.clone())
        .map_err(|e| ServerError::BibliographyError(e.to_string()))?;

    // Create processor and render bibliography.
    let mut processor = Processor::new(style, bibliography);
    let inject_ast_indices = params.inject_ast_indices.unwrap_or(false);
    processor.set_inject_ast_indices(inject_ast_indices);

    let output_format = params.output_format.unwrap_or_default();
    let content = render_bibliography_with_format(&processor, output_format)?;
    let entries = matches!(output_format, OutputFormat::Plain).then(|| {
        content
            .lines()
            .filter(|line| !line.is_empty())
            .map(std::string::ToString::to_string)
            .collect()
    });
    let result = BibliographyResult {
        format: output_format,
        content,
        entries,
    };

    Ok(json!({
        "id": id,
        "result": result
    }))
}

fn render_citation_with_format(
    processor: &Processor,
    citation: &Citation,
    format: OutputFormat,
) -> Result<String, ServerError> {
    match format {
        OutputFormat::Plain => Ok(processor.process_citation_with_format::<PlainText>(citation)?),
        OutputFormat::Html => Ok(processor.process_citation_with_format::<Html>(citation)?),
        OutputFormat::Djot => Ok(processor.process_citation_with_format::<Djot>(citation)?),
        OutputFormat::Latex => Ok(processor.process_citation_with_format::<Latex>(citation)?),
        OutputFormat::Typst => Ok(processor.process_citation_with_format::<Typst>(citation)?),
    }
}

fn render_bibliography_with_format(
    processor: &Processor,
    format: OutputFormat,
) -> Result<String, ServerError> {
    match format {
        OutputFormat::Plain => Ok(processor.render_bibliography_with_format::<PlainText>()),
        OutputFormat::Html => Ok(processor.render_bibliography_with_format::<Html>()),
        OutputFormat::Djot => Ok(processor.render_bibliography_with_format::<Djot>()),
        OutputFormat::Latex => Ok(processor.render_bibliography_with_format::<Latex>()),
        OutputFormat::Typst => Ok(processor.render_bibliography_with_format::<Typst>()),
    }
}

/// Validate a style YAML file.
fn validate_style(params: &Value, id: Value) -> Result<Value, ServerError> {
    require_field(params, "style_path")?;
    let params: ValidateStyleParams = serde_json::from_value(params.clone())
        .map_err(|e| ServerError::CitationError(e.to_string()))?;

    match load_style(&params.style_path) {
        Ok(_) => Ok(json!({
            "id": id,
            "result": {
                "valid": true,
                "warnings": []
            }
        })),
        Err(e) => Ok(json!({
            "id": id,
            "result": {
                "valid": false,
                "warnings": [e.to_string()]
            }
        })),
    }
}

/// Format a complete document's citations and bibliography.
fn format_document(params: &Value, id: Value) -> Result<Value, ServerError> {
    let request: citum_engine::FormatDocumentRequest = serde_json::from_value(params.clone())
        .map_err(|e| ServerError::CitationError(format!("Invalid request JSON: {}", e)))?;

    let result = match &request.style {
        citum_engine::StyleInput::Yaml(_) => citum_engine::format_document(request)
            .map_err(|e| ServerError::CitationError(e.to_string()))?,
        citum_engine::StyleInput::Id(s)
        | citum_engine::StyleInput::Uri(s)
        | citum_engine::StyleInput::Path(s) => {
            let style = load_style(s)?;
            citum_engine::format_document_with_style(style, request)
                .map_err(|e| ServerError::CitationError(e.to_string()))?
        }
    };

    let result_json =
        serde_json::to_value(&result).map_err(|e| ServerError::CitationError(e.to_string()))?;

    Ok(json!({
        "id": id,
        "result": result_json
    }))
}

/// Load a style through the standard resolver chain.
///
/// The chain includes file, store, HTTP, git, and registry resolvers.
/// This server is intended for local use only; do not expose it to untrusted
/// clients, as `style_input` can trigger outbound network requests (SSRF risk).
fn load_style(style_input: &str) -> Result<Style, ServerError> {
    use citum_store::resolver::{ResolverError, StyleResolver};

    let chain = citum_store::build_standard_chain()
        .map_err(|e| ServerError::ResolverError(e.to_string()))?;

    match chain.resolve_style(style_input) {
        Ok(style) => {
            let mut resolved = style
                .try_into_resolved_with(Some(&chain))
                .map_err(|e| ServerError::StyleResolution(e.to_string()))?;
            resolved.extends = None;
            Ok(resolved)
        }
        Err(ResolverError::StyleNotFound(_)) => {
            Err(ServerError::StyleNotFound(style_input.to_string()))
        }
        Err(e) => Err(ServerError::ResolverError(e.to_string())),
    }
}

/// Run the JSON-RPC server on stdin/stdout.
/// Reads newline-delimited JSON requests and writes newline-delimited JSON responses.
///
/// # Errors
///
/// Returns an error when reading from stdin, writing to stdout, or flushing the
/// output stream fails.
pub fn run_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let reader = stdin.lock();
    for line in reader.lines() {
        let line = line?;

        // Skip empty lines.
        if line.is_empty() {
            continue;
        }

        // Try to parse the request.
        let response = match serde_json::from_str::<RpcRequest>(&line) {
            Ok(req) => match dispatch(req.clone()) {
                Ok(result) => result,
                Err((id, error)) => json!({
                    "id": id,
                    "error": error
                }),
            },
            Err(e) => {
                // Invalid JSON: send error without ID.
                json!({
                    "id": Value::Null,
                    "error": format!("invalid JSON: {}", e)
                })
            }
        };

        // Write response as newline-delimited JSON.
        writeln!(stdout, "{response}")?;
        stdout.flush()?;
    }

    Ok(())
}

// Helper to make RpcRequest cloneable for error reporting.
impl Clone for RpcRequest {
    fn clone(&self) -> Self {
        RpcRequest {
            id: self.id.clone(),
            method: self.method.clone(),
            params: self.params.clone(),
        }
    }
}
