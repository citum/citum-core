/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

use crate::error::ServerError;
use citum_engine::{Bibliography, Citation, Processor};
use citum_schema::Style;
use serde::Deserialize;
use serde_json::{Value, json};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

/// JSON-RPC request envelope.
#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    pub id: Value,
    pub method: String,
    pub params: Value,
}

/// Bibliography rendering response (array of strings).
type BibliographyResult = Vec<String>;

/// Main RPC dispatcher that processes a single request.
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

    // Load the style.
    let style = load_style(style_path)?;

    // Deserialize references and citation from JSON.
    let bibliography: Bibliography = serde_json::from_value(refs.clone())
        .map_err(|e| ServerError::BibliographyError(e.to_string()))?;

    let citation: Citation = serde_json::from_value(citation_obj.clone())
        .map_err(|e| ServerError::CitationError(e.to_string()))?;

    // Create processor and render.
    let processor = Processor::new(style, bibliography);

    let result = processor
        .process_citation(&citation)
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

    // Load the style.
    let style = load_style(style_path)?;

    // Deserialize bibliography from JSON.
    let bibliography: Bibliography = serde_json::from_value(refs.clone())
        .map_err(|e| ServerError::BibliographyError(e.to_string()))?;

    // Create processor and render bibliography.
    let processor = Processor::new(style, bibliography);

    let bib_string = processor.render_bibliography();

    // Split by newlines to return as array.
    let result: BibliographyResult = bib_string
        .lines()
        .filter(|line| !line.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(json!({
        "id": id,
        "result": result
    }))
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

    serde_yaml::from_str::<Style>(&content).map_err(|e| ServerError::StyleValidation(e.to_string()))
}

/// Run the JSON-RPC server on stdin/stdout.
/// Reads newline-delimited JSON requests and writes newline-delimited JSON responses.
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
        writeln!(stdout, "{}", response)?;
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
