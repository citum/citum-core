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
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

/// JSON-RPC request envelope.
#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    /// The request identifier echoed back in success and error responses.
    pub id: Value,
    /// The JSON-RPC method name to dispatch.
    pub method: String,
    /// The method-specific parameter object.
    pub params: Value,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum OutputFormat {
    #[default]
    Plain,
    Html,
    Djot,
    Latex,
    Typst,
}

impl OutputFormat {
    fn parse(params: &Value) -> Result<Self, ServerError> {
        match params.get("output_format") {
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|_| ServerError::UnsupportedOutputFormat(value.to_string())),
            None => Ok(Self::default()),
        }
    }
}

fn parse_inject_ast_indices(params: &Value) -> bool {
    params
        .get("inject_ast_indices")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

#[derive(Debug, Serialize)]
struct BibliographyResult {
    format: OutputFormat,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    entries: Option<Vec<String>>,
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
        _ => Err((Some(req.id), format!("unknown method: {}", req.method))),
    }
}

/// Render a single citation.
fn render_citation(params: &Value, id: Value) -> Result<Value, ServerError> {
    let style_path = params
        .get("style_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ServerError::MissingField("style_path".to_string()))?;

    let refs = params
        .get("refs")
        .ok_or_else(|| ServerError::MissingField("refs".to_string()))?;

    let citation_obj = params
        .get("citation")
        .ok_or_else(|| ServerError::MissingField("citation".to_string()))?;
    let output_format = OutputFormat::parse(params)?;
    let inject_ast_indices = parse_inject_ast_indices(params);

    // Load the style.
    let style = load_style(style_path)?;

    // Deserialize references and citation from JSON.
    let bibliography: Bibliography = serde_json::from_value(refs.clone())
        .map_err(|e| ServerError::BibliographyError(e.to_string()))?;

    let citation: Citation = serde_json::from_value(citation_obj.clone())
        .map_err(|e| ServerError::CitationError(e.to_string()))?;

    // Create processor and render.
    let mut processor = Processor::new(style, bibliography);
    processor.set_inject_ast_indices(inject_ast_indices);

    let result = render_citation_with_format(&processor, &citation, output_format)
        .map_err(|e| ServerError::CitationError(e.to_string()))?;

    Ok(json!({
        "id": id,
        "result": result
    }))
}

/// Render a bibliography.
fn render_bibliography(params: &Value, id: Value) -> Result<Value, ServerError> {
    let style_path = params
        .get("style_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ServerError::MissingField("style_path".to_string()))?;

    let refs = params
        .get("refs")
        .ok_or_else(|| ServerError::MissingField("refs".to_string()))?;
    let output_format = OutputFormat::parse(params)?;
    let inject_ast_indices = parse_inject_ast_indices(params);

    // Load the style.
    let style = load_style(style_path)?;

    // Deserialize bibliography from JSON.
    let bibliography: Bibliography = serde_json::from_value(refs.clone())
        .map_err(|e| ServerError::BibliographyError(e.to_string()))?;

    // Create processor and render bibliography.
    let mut processor = Processor::new(style, bibliography);
    processor.set_inject_ast_indices(inject_ast_indices);

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
    let style_path = params
        .get("style_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ServerError::MissingField("style_path".to_string()))?;

    match load_style(style_path) {
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

/// Load a style from a YAML file.
fn load_style(style_path: &str) -> Result<Style, ServerError> {
    let path = Path::new(style_path);

    // Check if file exists.
    if !path.exists() {
        return Err(ServerError::StyleNotFound(style_path.to_string()));
    }

    let content =
        fs::read_to_string(path).map_err(|_| ServerError::StyleNotFound(style_path.to_string()))?;

    Style::from_yaml_str(&content).map_err(|e| ServerError::StyleValidation(e.to_string()))
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
