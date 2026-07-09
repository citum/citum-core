/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! JSON-RPC request handling for the stdio transport.
//!
//! This module defines the request envelope shared by the stdio entrypoint
//! and the HTTP handler, plus the dispatcher that maps method names to the
//! renderer and validator operations.
//!
//! ## JSON-RPC envelope
//!
//! Requests use `{ "id", "method", "params" }`. Successful responses echo
//! the request ID and include `result`; failures echo the request ID when
//! available and include `error`.
//!
//! See the crate-level documentation for the shared method table exposed by
//! both transports.
//!
//! ## `format_document` result
//!
//! `format_document` returns one document-level result object with three
//! top-level fields:
//!
//! - `formatted_citations`: one rendered citation object per input citation.
//! - `bibliography`: the rendered bibliography object, including `format`,
//!   `content`, and `entries`.
//! - `warnings`: non-fatal diagnostics produced while evaluating the style.
//!
//! Clients should read bibliography output from `result.bibliography`, not
//! from `result.formatted_citations`.
//!
//! ## Stdio example
//!
//! From the Citum repository root:
//!
//! ```text
//! printf '%s\n' '{"id":1,"method":"render_citation","params":{"style_path":"styles/embedded/apa-7th.yaml","refs":{"hawking1988":{"id":"hawking1988","class":"monograph","type":"book","title":"A Brief History of Time","author":[{"family":"Hawking","given":"Stephen"}],"issued":"1988"}},"citation":{"id":"cite-1","items":[{"id":"hawking1988"}]}}}' \
//!   | cargo run -q -p citum-server
//! ```

use crate::error::ServerError;
use citum_engine::{
    Bibliography, BibliographyBlockRequest, Citation, DocumentOptions, Processor, StyleInput,
    render::{djot::Djot, html::Html, latex::Latex, plain::PlainText, typst::Typst},
};
#[cfg(feature = "session")]
use citum_engine::{
    CitationInsertPosition, CitationOccurrence, CitationOccurrenceItem, DocumentSession,
    OpenSessionResult, OutputFormatKind, RefsInput, apply_style_overrides,
};
use citum_schema::Style;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[cfg(all(feature = "session", feature = "http"))]
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
#[cfg(all(feature = "session", feature = "http"))]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(all(feature = "session", feature = "http"))]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    pub style: StyleInput,
    /// Optional partial-style overlay (YAML or JSON) merged over the resolved base
    /// style for this request only. Uses the same null-aware, typed-merge semantics
    /// as `extends` inheritance. The base style is never mutated.
    pub style_overrides: Option<String>,
    /// Optional BCP 47 locale override.
    pub locale: Option<String>,
    /// Output format (plain, html, djot, latex, typst). Defaults to plain.
    pub output_format: Option<OutputFormat>,
    /// Bibliography input as `RefsInput`: path (YAML/JSON/CBOR or `.bib`), inline YAML,
    /// inline JSON, inline BibLaTeX (`{"kind":"biblatex","value":"@book{…}"}`) or legacy bare map.
    pub refs: serde_json::Value,
    /// Ordered citations as they appear in the document.
    pub citations: serde_json::Value,
    /// Optional bibliography blocks to render in document order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bibliography_blocks: Vec<BibliographyBlockRequest>,
    /// Optional document-level configuration.
    pub document_options: Option<DocumentOptions>,
    /// Reference IDs to include in the bibliography without an in-text citation.
    ///
    /// Each ID must be present in `refs`. Unknown IDs produce a `nocite_missing_ref`
    /// warning and are otherwise ignored.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nocite: Vec<String>,
}

/// Parameters for the `open_session` method.
#[cfg(feature = "session")]
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct OpenSessionParams {
    /// Style identifier, path, URI, or inline YAML.
    pub style: StyleInput,
    /// Optional partial-style overlay (YAML or JSON) merged over the resolved base
    /// style for this session. Uses the same null-aware, typed-merge semantics as
    /// `extends` inheritance. The base style is never mutated.
    pub style_overrides: Option<String>,
    /// Optional BCP 47 locale override.
    pub locale: Option<String>,
    /// Output format (plain, html, djot, latex, typst). Defaults to plain.
    pub output_format: Option<OutputFormatKind>,
    /// Optional document-level configuration.
    pub document_options: Option<DocumentOptions>,
}

/// Parameters for the `put_references` method.
#[cfg(feature = "session")]
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct PutReferencesParams {
    /// Session identifier returned by `open_session`.
    pub session_id: Option<String>,
    /// Full reference input for the session.
    pub refs: RefsInput,
}

/// Parameters for the `set_nocite` method.
#[cfg(feature = "session")]
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct SetNociteParams {
    /// Session identifier returned by `open_session`.
    pub session_id: Option<String>,
    /// Reference IDs to include in the bibliography without an in-text citation.
    ///
    /// Each ID must be present in the session's loaded references. Unknown IDs
    /// produce a `nocite_missing_ref` warning and are otherwise ignored.
    #[serde(default)]
    pub nocite: Vec<String>,
}

/// Parameters for the `insert_citations_batch` method.
#[cfg(feature = "session")]
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct InsertCitationsBatchParams {
    /// Session identifier returned by `open_session`.
    pub session_id: Option<String>,
    /// Complete ordered citation list.
    pub citations: Vec<CitationOccurrence>,
}

/// Parameters for the `insert_citation` method.
#[cfg(feature = "session")]
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct InsertCitationParams {
    /// Session identifier returned by `open_session`.
    pub session_id: Option<String>,
    /// Citation to insert.
    pub citation: CitationOccurrence,
    /// Optional neighbour-ID position context.
    pub position: Option<CitationInsertPosition>,
}

/// Parameters for the `update_citation` method.
#[cfg(feature = "session")]
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct UpdateCitationParams {
    /// Session identifier returned by `open_session`.
    pub session_id: Option<String>,
    /// Existing citation ID to update.
    pub citation_id: String,
    /// Replacement citation data.
    pub citation: CitationOccurrence,
    /// Optional neighbour-ID position context.
    pub position: Option<CitationInsertPosition>,
}

/// Parameters for the `delete_citation` method.
#[cfg(feature = "session")]
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct DeleteCitationParams {
    /// Session identifier returned by `open_session`.
    pub session_id: Option<String>,
    /// Existing citation ID to delete.
    pub citation_id: String,
}

/// Parameters for the `preview_citation` method.
#[cfg(feature = "session")]
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct PreviewCitationParams {
    /// Session identifier returned by `open_session`.
    pub session_id: Option<String>,
    /// Citation items to preview.
    pub items: Vec<CitationOccurrenceItem>,
    /// Citation mode for the preview (integral / non-integral).
    pub mode: Option<citum_schema::data::citation::CitationMode>,
    /// Optional neighbour-ID position context.
    pub position: Option<CitationInsertPosition>,
}

/// Parameters for methods that only need a session ID.
#[cfg(feature = "session")]
#[derive(Debug, Deserialize)]
#[cfg_attr(
    any(feature = "schema", feature = "schema-types"),
    derive(schemars::JsonSchema)
)]
pub struct SessionIdParams {
    /// Session identifier returned by `open_session`.
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct BibliographyResult {
    format: OutputFormat,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    entries: Option<Vec<String>>,
}

#[derive(Debug)]
#[cfg(all(feature = "session", feature = "http"))]
struct StoredSession {
    session: DocumentSession,
    last_access: SystemTime,
}

#[cfg(all(feature = "session", feature = "http"))]
const HTTP_SESSION_TTL_SECS: u64 = 30 * 60;

#[cfg(feature = "session")]
#[derive(Debug)]
enum SessionMode {
    Stdio {
        session: Box<Option<DocumentSession>>,
    },
    #[cfg(all(feature = "session", feature = "http"))]
    Http {
        sessions: HashMap<String, StoredSession>,
        next_session_id: AtomicU64,
        ttl: Duration,
    },
}

/// Stateful RPC dispatcher used by stdio and HTTP transports.
#[derive(Debug)]
pub struct RpcDispatcher {
    #[cfg(feature = "session")]
    session_mode: SessionMode,
}

/// Error returned by the stateful RPC dispatcher.
#[derive(Debug)]
pub enum RpcDispatchError {
    /// Legacy string-valued error response.
    Message(String),
    /// Complete JSON response for methods with structured top-level errors.
    Response(Box<Value>),
}

impl RpcDispatcher {
    /// Create a dispatcher with one implicit stdio session slot.
    pub fn new_stdio() -> Self {
        Self {
            #[cfg(feature = "session")]
            session_mode: SessionMode::Stdio {
                session: Box::new(None),
            },
        }
    }

    /// Create a dispatcher with an HTTP multi-session store.
    #[cfg(feature = "http")]
    pub fn new_http() -> Self {
        Self {
            #[cfg(feature = "session")]
            session_mode: SessionMode::Http {
                sessions: HashMap::new(),
                next_session_id: AtomicU64::new(1),
                ttl: Duration::from_secs(HTTP_SESSION_TTL_SECS),
            },
        }
    }

    /// Process one RPC request against this dispatcher's session state.
    ///
    /// # Errors
    ///
    /// Returns a dispatch error when the method is unknown or a method-specific
    /// validation, rendering, or session lookup fails.
    pub fn dispatch(
        &mut self,
        req: RpcRequest,
    ) -> Result<Value, (Option<Value>, RpcDispatchError)> {
        let id = req.id.clone();

        match req.method.as_str() {
            "render_citation" => render_citation(&req.params, id)
                .map_err(|e| (Some(req.id), RpcDispatchError::Message(e.to_string()))),
            "render_bibliography" => render_bibliography(&req.params, id)
                .map_err(|e| (Some(req.id), RpcDispatchError::Message(e.to_string()))),
            "validate_style" => validate_style(&req.params, id)
                .map_err(|e| (Some(req.id), RpcDispatchError::Message(e.to_string()))),
            "format_document" => format_document(&req.params, id)
                .map_err(|e| (Some(req.id), RpcDispatchError::Message(e.to_string()))),
            #[cfg(feature = "session")]
            "open_session" => self
                .open_session(&req.params, id)
                .map_err(|e| (Some(req.id), RpcDispatchError::Message(e.to_string()))),
            #[cfg(feature = "session")]
            "put_references" => self.put_references(&req.params, id, req.id.clone()),
            #[cfg(feature = "session")]
            "set_nocite" => self.set_nocite(&req.params, id, req.id.clone()),
            #[cfg(feature = "session")]
            "insert_citations_batch" => {
                self.insert_citations_batch(&req.params, id, req.id.clone())
            }
            #[cfg(feature = "session")]
            "insert_citation" => self.insert_citation(&req.params, id, req.id.clone()),
            #[cfg(feature = "session")]
            "update_citation" => self.update_citation(&req.params, id, req.id.clone()),
            #[cfg(feature = "session")]
            "delete_citation" => self.delete_citation(&req.params, id, req.id.clone()),
            #[cfg(feature = "session")]
            "preview_citation" => self.preview_citation(&req.params, id, req.id.clone()),
            #[cfg(feature = "session")]
            "get_citations" => self.get_citations(&req.params, id, req.id.clone()),
            #[cfg(feature = "session")]
            "get_bibliography" => self.get_bibliography(&req.params, id, req.id.clone()),
            #[cfg(feature = "session")]
            "close_session" => self.close_session(&req.params, id, req.id.clone()),
            _ => Err((
                Some(req.id),
                RpcDispatchError::Message(format!("unknown method: {}", req.method)),
            )),
        }
    }

    #[cfg(feature = "session")]
    fn open_session(&mut self, params: &Value, id: Value) -> Result<Value, ServerError> {
        let params: OpenSessionParams = serde_json::from_value(params.clone())
            .map_err(|e| ServerError::CitationError(format!("Invalid request JSON: {e}")))?;
        let mut style = resolve_style_input(&params.style)?;
        if let Some(src) = &params.style_overrides {
            apply_style_overrides(&mut style, src)
                .map_err(|e| ServerError::StyleValidation(e.to_string()))?;
        }
        let session = DocumentSession::new(
            style,
            params.style,
            params.locale,
            params.output_format.unwrap_or_default(),
            params.document_options,
        );
        let session_id = match &mut self.session_mode {
            SessionMode::Stdio { session: slot } => {
                **slot = Some(session);
                "default".to_string()
            }
            #[cfg(feature = "http")]
            SessionMode::Http {
                sessions,
                next_session_id,
                ..
            } => {
                let next = next_session_id.fetch_add(1, Ordering::Relaxed);
                let session_id = format!("s-{next:016x}");
                sessions.insert(
                    session_id.clone(),
                    StoredSession {
                        session,
                        last_access: SystemTime::now(),
                    },
                );
                session_id
            }
        };
        let result = OpenSessionResult { session_id };
        Ok(json!({ "id": id, "result": result }))
    }

    #[cfg(feature = "session")]
    fn put_references(
        &mut self,
        params: &Value,
        id: Value,
        request_id: Value,
    ) -> Result<Value, (Option<Value>, RpcDispatchError)> {
        let params: PutReferencesParams = parse_session_params(params, &request_id)?;
        let session = self.session_mut(params.session_id.as_deref(), &request_id)?;
        session
            .put_references(params.refs)
            .map_err(|e| (Some(request_id), RpcDispatchError::Message(e.to_string())))?;
        Ok(json!({ "id": id, "result": {} }))
    }

    #[cfg(feature = "session")]
    fn set_nocite(
        &mut self,
        params: &Value,
        id: Value,
        request_id: Value,
    ) -> Result<Value, (Option<Value>, RpcDispatchError)> {
        let params: SetNociteParams = parse_session_params(params, &request_id)?;
        let session = self.session_mut(params.session_id.as_deref(), &request_id)?;
        let result = session
            .set_nocite(params.nocite)
            .map_err(|e| (Some(request_id), RpcDispatchError::Message(e.to_string())))?;
        Ok(json!({ "id": id, "result": result }))
    }

    #[cfg(feature = "session")]
    fn insert_citations_batch(
        &mut self,
        params: &Value,
        id: Value,
        request_id: Value,
    ) -> Result<Value, (Option<Value>, RpcDispatchError)> {
        let params: InsertCitationsBatchParams = parse_session_params(params, &request_id)?;
        let session = self.session_mut(params.session_id.as_deref(), &request_id)?;
        let result = session
            .insert_citations_batch(params.citations)
            .map_err(session_method_error(&request_id))?;
        Ok(json!({ "id": id, "result": result }))
    }

    #[cfg(feature = "session")]
    fn insert_citation(
        &mut self,
        params: &Value,
        id: Value,
        request_id: Value,
    ) -> Result<Value, (Option<Value>, RpcDispatchError)> {
        let params: InsertCitationParams = parse_session_params(params, &request_id)?;
        let session = self.session_mut(params.session_id.as_deref(), &request_id)?;
        let result = session
            .insert_citation(params.citation, params.position)
            .map_err(session_method_error(&request_id))?;
        Ok(json!({ "id": id, "result": result }))
    }

    #[cfg(feature = "session")]
    fn update_citation(
        &mut self,
        params: &Value,
        id: Value,
        request_id: Value,
    ) -> Result<Value, (Option<Value>, RpcDispatchError)> {
        let params: UpdateCitationParams = parse_session_params(params, &request_id)?;
        let session = self.session_mut(params.session_id.as_deref(), &request_id)?;
        let result = session
            .update_citation(&params.citation_id, params.citation, params.position)
            .map_err(session_method_error(&request_id))?;
        Ok(json!({ "id": id, "result": result }))
    }

    #[cfg(feature = "session")]
    fn delete_citation(
        &mut self,
        params: &Value,
        id: Value,
        request_id: Value,
    ) -> Result<Value, (Option<Value>, RpcDispatchError)> {
        let params: DeleteCitationParams = parse_session_params(params, &request_id)?;
        let session = self.session_mut(params.session_id.as_deref(), &request_id)?;
        let result = session
            .delete_citation(&params.citation_id)
            .map_err(session_method_error(&request_id))?;
        Ok(json!({ "id": id, "result": result }))
    }

    #[cfg(feature = "session")]
    fn preview_citation(
        &mut self,
        params: &Value,
        id: Value,
        request_id: Value,
    ) -> Result<Value, (Option<Value>, RpcDispatchError)> {
        let params: PreviewCitationParams = parse_session_params(params, &request_id)?;
        let session = self.session_mut(params.session_id.as_deref(), &request_id)?;
        let result = session
            .preview_citation(params.items, params.mode, params.position)
            .map_err(session_method_error(&request_id))?;
        Ok(json!({ "id": id, "result": result }))
    }

    #[cfg(feature = "session")]
    fn get_citations(
        &mut self,
        params: &Value,
        id: Value,
        request_id: Value,
    ) -> Result<Value, (Option<Value>, RpcDispatchError)> {
        let params: SessionIdParams = parse_session_params(params, &request_id)?;
        let session = self.session_mut(params.session_id.as_deref(), &request_id)?;
        Ok(json!({
            "id": id,
            "result": { "formatted_citations": session.get_citations() }
        }))
    }

    #[cfg(feature = "session")]
    fn get_bibliography(
        &mut self,
        params: &Value,
        id: Value,
        request_id: Value,
    ) -> Result<Value, (Option<Value>, RpcDispatchError)> {
        let params: SessionIdParams = parse_session_params(params, &request_id)?;
        let session = self.session_mut(params.session_id.as_deref(), &request_id)?;
        Ok(json!({
            "id": id,
            "result": { "bibliography": session.get_bibliography() }
        }))
    }

    #[cfg(feature = "session")]
    fn close_session(
        &mut self,
        params: &Value,
        id: Value,
        request_id: Value,
    ) -> Result<Value, (Option<Value>, RpcDispatchError)> {
        let params: SessionIdParams = parse_session_params(params, &request_id)?;
        #[cfg(not(feature = "http"))]
        let _ = &params;
        match &mut self.session_mode {
            SessionMode::Stdio { session } => {
                **session = None;
            }
            #[cfg(feature = "http")]
            SessionMode::Http { sessions, .. } => {
                let session_id = params.session_id.as_deref().ok_or_else(|| {
                    (
                        Some(request_id.clone()),
                        RpcDispatchError::Message("missing required field: session_id".to_string()),
                    )
                })?;
                sessions.remove(session_id);
            }
        }
        Ok(json!({ "id": id, "result": {} }))
    }

    #[cfg(feature = "session")]
    fn session_mut(
        &mut self,
        session_id: Option<&str>,
        request_id: &Value,
    ) -> Result<&mut DocumentSession, (Option<Value>, RpcDispatchError)> {
        #[cfg(not(feature = "http"))]
        let _ = session_id;
        match &mut self.session_mode {
            SessionMode::Stdio { session } => session.as_mut().as_mut().ok_or_else(|| {
                (
                    Some(request_id.clone()),
                    RpcDispatchError::Message("session not open".to_string()),
                )
            }),
            #[cfg(feature = "http")]
            SessionMode::Http { sessions, ttl, .. } => {
                let session_id = session_id.ok_or_else(|| {
                    (
                        Some(request_id.clone()),
                        RpcDispatchError::Message("missing required field: session_id".to_string()),
                    )
                })?;
                if let Some(stored) = sessions.get(session_id)
                    && stored.last_access.elapsed().unwrap_or_default() > *ttl
                {
                    let expired_at = format_system_time(stored.last_access + *ttl);
                    sessions.remove(session_id);
                    return Err((
                        Some(request_id.clone()),
                        RpcDispatchError::Response(Box::new(json!({
                            "id": request_id,
                            "error": "session_expired",
                            "session_id": session_id,
                            "expired_at": expired_at
                        }))),
                    ));
                }
                let stored = sessions.get_mut(session_id).ok_or_else(|| {
                    (
                        Some(request_id.clone()),
                        RpcDispatchError::Message(format!("session not found: {session_id}")),
                    )
                })?;
                stored.last_access = SystemTime::now();
                Ok(&mut stored.session)
            }
        }
    }
}

impl Default for RpcDispatcher {
    fn default() -> Self {
        Self::new_stdio()
    }
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
    let mut dispatcher = RpcDispatcher::new_stdio();
    dispatcher.dispatch(req).map_err(|(id, error)| match error {
        RpcDispatchError::Message(message) => (id, message),
        RpcDispatchError::Response(value) => (id, value.to_string()),
    })
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
    let mut run = processor.begin_run();
    let result = render_citation_with_format(&processor, &citation, output_format, &mut run)
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
    run: &mut citum_engine::processor::RunState,
) -> Result<String, ServerError> {
    match format {
        OutputFormat::Plain => {
            Ok(processor.process_citation_with_format::<PlainText>(citation, run)?)
        }
        OutputFormat::Html => Ok(processor.process_citation_with_format::<Html>(citation, run)?),
        OutputFormat::Djot => Ok(processor.process_citation_with_format::<Djot>(citation, run)?),
        OutputFormat::Latex => Ok(processor.process_citation_with_format::<Latex>(citation, run)?),
        OutputFormat::Typst => Ok(processor.process_citation_with_format::<Typst>(citation, run)?),
    }
}

fn render_bibliography_with_format(
    processor: &Processor,
    format: OutputFormat,
) -> Result<String, ServerError> {
    match format {
        OutputFormat::Plain => {
            Ok(processor.render_bibliography_with_format_standalone::<PlainText>())
        }
        OutputFormat::Html => Ok(processor.render_bibliography_with_format_standalone::<Html>()),
        OutputFormat::Djot => Ok(processor.render_bibliography_with_format_standalone::<Djot>()),
        OutputFormat::Latex => Ok(processor.render_bibliography_with_format_standalone::<Latex>()),
        OutputFormat::Typst => Ok(processor.render_bibliography_with_format_standalone::<Typst>()),
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

#[cfg(feature = "session")]
fn resolve_style_input(style_input: &StyleInput) -> Result<Style, ServerError> {
    match style_input {
        StyleInput::Yaml(_) => style_input
            .resolve_local()
            .map_err(|e| ServerError::CitationError(e.to_string())),
        StyleInput::Id(s) | StyleInput::Uri(s) | StyleInput::Path(s) => load_style(s),
    }
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

#[cfg(feature = "session")]
fn parse_session_params<T>(
    params: &Value,
    request_id: &Value,
) -> Result<T, (Option<Value>, RpcDispatchError)>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(params.clone()).map_err(|e| {
        (
            Some(request_id.clone()),
            RpcDispatchError::Message(format!("Invalid request JSON: {e}")),
        )
    })
}

#[cfg(feature = "session")]
fn session_method_error(
    request_id: &Value,
) -> impl FnOnce(citum_engine::DocumentSessionError) -> (Option<Value>, RpcDispatchError) + '_ {
    |err| {
        (
            Some(request_id.clone()),
            RpcDispatchError::Message(err.to_string()),
        )
    }
}

#[cfg(all(feature = "session", feature = "http"))]
fn format_system_time(time: SystemTime) -> String {
    let seconds = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if let Ok(seconds) = i64::try_from(seconds)
        && let Ok(datetime) = time::OffsetDateTime::from_unix_timestamp(seconds)
        && let Ok(formatted) = datetime.format(&time::format_description::well_known::Rfc3339)
    {
        return formatted;
    }
    format!("unix:{seconds}")
}

/// Build a JSON-RPC error response from a dispatch error.
pub fn error_response(id: Option<Value>, error: RpcDispatchError) -> Value {
    match error {
        RpcDispatchError::Message(error) => json!({
            "id": id,
            "error": error
        }),
        RpcDispatchError::Response(response) => *response,
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
    let mut dispatcher = RpcDispatcher::new_stdio();

    let reader = stdin.lock();
    for line in reader.lines() {
        let line = line?;

        // Skip empty lines.
        if line.is_empty() {
            continue;
        }

        // Try to parse the request.
        let response = match serde_json::from_str::<RpcRequest>(&line) {
            Ok(req) => match dispatcher.dispatch(req.clone()) {
                Ok(result) => result,
                Err((id, error)) => error_response(id, error),
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
