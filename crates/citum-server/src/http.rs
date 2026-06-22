/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Default HTTP transport for the JSON-RPC server.
//!
//! This module is compiled when the default-on `http` feature is enabled.
//!
//! The HTTP server exposes `POST /rpc` for JSON-RPC requests, `GET /rpc` as a
//! method hint, and `GET /rpc/methods` as a static descriptor list. When the
//! `schema` feature is enabled, `GET /rpc/schema` also exposes the schema
//! mirror used by the HTTP API.
//!
//! Responses follow the same JSON-RPC shape as the stdio transport, including
//! the document-level `format_document` result with `formatted_citations`,
//! `bibliography`, and `warnings`.
//!
//! The server binds to `127.0.0.1` and is intended for local use. If you need
//! to expose it beyond the local machine, put it behind authentication and
//! transport security.
//!
//! ## Example
//!
//! ```text
//! cargo run -q -p citum-server -- --http --port 9000
//!
//! curl -s http://localhost:9000/rpc \
//!   -H 'Content-Type: application/json' \
//!   -d '{"id":2,"method":"format_document","params":{"style":{"kind":"path","value":"styles/embedded/apa-7th.yaml"},"output_format":"html","refs":{"smith2010":{"id":"smith2010","class":"monograph","type":"book","title":"Nationalism: Theory, Ideology, History","author":[{"family":"Smith","given":"Anthony D."}],"issued":"2010","publisher":{"name":"Polity"}}},"citations":[{"id":"cite-1","items":[{"id":"smith2010","locator":{"label":"page","value":"10"}}]}],"document_options":{"show_semantics":true}}}'
//! ```

use crate::rpc::{RpcDispatcher, RpcRequest, error_response};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// Maximum accepted HTTP JSON-RPC request size.
pub const DEFAULT_HTTP_BODY_LIMIT_BYTES: usize = 8 * 1024 * 1024;

/// HTTP handler for JSON-RPC requests.
/// Dispatches to the same RPC logic as stdin/stdout.
async fn rpc_handler(
    State(dispatcher): State<Arc<Mutex<RpcDispatcher>>>,
    Json(payload): Json<RpcRequest>,
) -> impl IntoResponse {
    let result = match dispatcher.lock() {
        Ok(mut dispatcher) => dispatcher
            .dispatch(payload.clone())
            .map_err(|(id, error)| (StatusCode::BAD_REQUEST, error_response(id, error))),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({
                "id": payload.id,
                "error": "RPC dispatcher mutex poisoned"
            }),
        )),
    };
    match result {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err((status, error)) => (status, Json(error)),
    }
}

/// GET /rpc — returns 405 with a JSON hint about POST usage.
///
/// Includes `Allow: POST` as required by RFC 9110 §15.5.6.
async fn rpc_get_hint() -> impl IntoResponse {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        [(header::ALLOW, "POST")],
        Json(json!({
            "error": "POST required",
            "hint": "Send a JSON-RPC envelope via POST. See GET /rpc/methods for available methods."
        })),
    )
}

/// GET /rpc/methods — returns a static descriptor list of all supported methods.
async fn rpc_methods() -> impl IntoResponse {
    let methods = vec![
        json!({
            "method": "render_citation",
            "description": "Render a single citation.",
            "required": ["style_path", "refs", "citation"],
            "optional": ["output_format", "inject_ast_indices"]
        }),
        json!({
            "method": "render_bibliography",
            "description": "Render a complete bibliography.",
            "required": ["style_path", "refs"],
            "optional": ["output_format", "inject_ast_indices"]
        }),
        json!({
            "method": "validate_style",
            "description": "Validate a Citum YAML style file.",
            "required": ["style_path"],
            "optional": []
        }),
        json!({
            "method": "format_document",
            "description": "Format all citations and bibliography in a document.",
            "required": ["style", "refs", "citations"],
            "optional": ["output_format", "locale", "document_options"]
        }),
    ];

    #[cfg(feature = "session")]
    let methods = {
        let mut methods = methods;
        methods.extend([
            json!({
                "method": "open_session",
                "description": "Open a stateful document session.",
                "required": ["style"],
                "optional": ["output_format", "locale", "document_options"]
            }),
            json!({
                "method": "put_references",
                "description": "Replace the full reference set for a session.",
                "required": ["session_id", "refs"],
                "optional": []
            }),
            json!({
                "method": "insert_citations_batch",
                "description": "Replace the full ordered citation list for a session.",
                "required": ["session_id", "citations"],
                "optional": []
            }),
            json!({
                "method": "insert_citation",
                "description": "Insert one citation into a session.",
                "required": ["session_id", "citation"],
                "optional": ["position"]
            }),
            json!({
                "method": "update_citation",
                "description": "Update one citation in a session.",
                "required": ["session_id", "citation_id", "citation"],
                "optional": ["position"]
            }),
            json!({
                "method": "delete_citation",
                "description": "Delete one citation from a session.",
                "required": ["session_id", "citation_id"],
                "optional": []
            }),
            json!({
                "method": "preview_citation",
                "description": "Render a citation preview without mutating session state.",
                "required": ["session_id", "items"],
                "optional": ["position"]
            }),
            json!({
                "method": "get_citations",
                "description": "Return current formatted citations for a session.",
                "required": ["session_id"],
                "optional": []
            }),
            json!({
                "method": "get_bibliography",
                "description": "Return current bibliography for a session.",
                "required": ["session_id"],
                "optional": []
            }),
            json!({
                "method": "close_session",
                "description": "Close and free a session.",
                "required": ["session_id"],
                "optional": []
            }),
        ]);
        methods
    };

    Json(json!(methods))
}

#[cfg(feature = "schema")]
async fn rpc_schema() -> impl IntoResponse {
    #[cfg(feature = "session")]
    use crate::rpc::{
        DeleteCitationParams, InsertCitationParams, InsertCitationsBatchParams, OpenSessionParams,
        PreviewCitationParams, PutReferencesParams, SessionIdParams, SetNociteParams,
        UpdateCitationParams,
    };
    use crate::rpc::{
        FormatDocumentParams, RenderBibliographyParams, RenderCitationParams, ValidateStyleParams,
    };
    use schemars::schema_for;

    let mut schema = serde_json::json!({
        "render_citation": schema_for!(RenderCitationParams),
        "render_bibliography": schema_for!(RenderBibliographyParams),
        "validate_style": schema_for!(ValidateStyleParams),
        "format_document": schema_for!(FormatDocumentParams),
    });
    #[cfg(feature = "session")]
    {
        if let Some(schema) = schema.as_object_mut() {
            schema.insert(
                "open_session".to_string(),
                json!(schema_for!(OpenSessionParams)),
            );
            schema.insert(
                "put_references".to_string(),
                json!(schema_for!(PutReferencesParams)),
            );
            schema.insert(
                "set_nocite".to_string(),
                json!(schema_for!(SetNociteParams)),
            );
            schema.insert(
                "insert_citations_batch".to_string(),
                json!(schema_for!(InsertCitationsBatchParams)),
            );
            schema.insert(
                "insert_citation".to_string(),
                json!(schema_for!(InsertCitationParams)),
            );
            schema.insert(
                "update_citation".to_string(),
                json!(schema_for!(UpdateCitationParams)),
            );
            schema.insert(
                "delete_citation".to_string(),
                json!(schema_for!(DeleteCitationParams)),
            );
            schema.insert(
                "preview_citation".to_string(),
                json!(schema_for!(PreviewCitationParams)),
            );
            schema.insert(
                "get_citations".to_string(),
                json!(schema_for!(SessionIdParams)),
            );
            schema.insert(
                "get_bibliography".to_string(),
                json!(schema_for!(SessionIdParams)),
            );
            schema.insert(
                "close_session".to_string(),
                json!(schema_for!(SessionIdParams)),
            );
        }
    }
    Json(schema)
}

/// Build the HTTP router for JSON-RPC requests.
pub fn app() -> Router {
    let dispatcher = Arc::new(Mutex::new(RpcDispatcher::new_http()));
    let router = Router::new()
        .route("/rpc", post(rpc_handler))
        .route("/rpc", get(rpc_get_hint))
        .route("/rpc/methods", get(rpc_methods))
        .layer(DefaultBodyLimit::max(DEFAULT_HTTP_BODY_LIMIT_BYTES))
        .with_state(dispatcher);

    #[cfg(feature = "schema")]
    let router = router.route("/rpc/schema", get(rpc_schema));

    router
}

/// Start the HTTP server on the given port.
///
/// # Errors
///
/// Returns an error when the socket cannot be bound or the HTTP server exits
/// with a transport-level failure.
pub async fn run_http(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    eprintln!("Citum server listening on http://{addr}");

    axum::serve(listener, app()).await?;

    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::{DEFAULT_HTTP_BODY_LIMIT_BYTES, app, rpc_handler};
    use crate::rpc::RpcDispatcher;
    use axum::{
        Json,
        body::{Body, to_bytes},
        extract::State,
        http::{Request, StatusCode},
        response::IntoResponse,
    };
    use serde_json::json;
    use std::panic;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    /// Absolute path to the APA style.
    /// `CARGO_MANIFEST_DIR` is the crate root; workspace root is two levels up.
    fn apa_style_path() -> String {
        format!(
            "{}/../../styles/embedded/apa-7th.yaml",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    /// Minimal bibliography: one book (Hawking 1988) in native Citum schema format.
    fn hawking_refs() -> serde_json::Value {
        json!({
            "ITEM-2": {
                "id": "ITEM-2",
                "class": "monograph",
                "type": "book",
                "title": "A Brief History of Time",
                "author": [{"family": "Hawking", "given": "Stephen"}],
                "issued": "1988"
            }
        })
    }

    async fn response_body_json(response: axum::response::Response<Body>) -> serde_json::Value {
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        serde_json::from_slice(&body).expect("response body should be valid JSON")
    }

    fn test_dispatcher() -> State<Arc<Mutex<RpcDispatcher>>> {
        State(Arc::new(Mutex::new(RpcDispatcher::new_http())))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rpc_handler_poisoned_dispatcher_returns_internal_server_error() {
        let dispatcher = Arc::new(Mutex::new(RpcDispatcher::new_http()));
        let poisoned = Arc::clone(&dispatcher);
        let _ = panic::catch_unwind(move || {
            let _guard = poisoned
                .lock()
                .expect("dispatcher lock should be available");
            panic!("poison dispatcher mutex");
        });
        let payload = serde_json::from_value(json!({
            "id": 25,
            "method": "validate_style",
            "params": {
                "style_path": apa_style_path()
            }
        }))
        .expect("payload should deserialize");

        let response = rpc_handler(State(dispatcher), Json(payload))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = response_body_json(response).await;
        assert_eq!(body["id"], 25);
        assert_eq!(body["error"], "RPC dispatcher mutex poisoned");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rpc_handler_render_citation_returns_ok() {
        let payload = serde_json::from_value(json!({
            "id": 1,
            "method": "render_citation",
            "params": {
                "style_path": apa_style_path(),
                "refs": hawking_refs(),
                "citation": {
                    "id": "cite-1",
                    "items": [{"id": "ITEM-2"}]
                }
            }
        }))
        .expect("payload should deserialize");

        let response = rpc_handler(test_dispatcher(), Json(payload))
            .await
            .into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = response_body_json(response).await;
        let result = body["result"].as_str().expect("result should be a string");
        assert!(
            result.contains("Hawking") || result.contains("1988"),
            "citation should reference the work: {result}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rpc_handler_render_bibliography_html_returns_ok() {
        let payload = serde_json::from_value(json!({
            "id": 4,
            "method": "render_bibliography",
            "params": {
                "style_path": apa_style_path(),
                "refs": hawking_refs(),
                "output_format": "html"
            }
        }))
        .expect("payload should deserialize");

        let response = rpc_handler(test_dispatcher(), Json(payload))
            .await
            .into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = response_body_json(response).await;
        assert_eq!(body["result"]["format"], "html");
        let content = body["result"]["content"]
            .as_str()
            .expect("content should be a string");
        assert!(
            content.contains("citum-bibliography"),
            "html bibliography should include wrapper markup"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rpc_handler_unknown_method_returns_bad_request() {
        let payload = serde_json::from_value(json!({
            "id": 2,
            "method": "frobnicate",
            "params": {}
        }))
        .expect("payload should deserialize");

        let response = rpc_handler(test_dispatcher(), Json(payload))
            .await
            .into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

        let body = response_body_json(response).await;
        assert_eq!(body["id"], 2);
        assert!(
            body["error"]
                .as_str()
                .expect("error should be a string")
                .contains("unknown method")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rpc_handler_missing_field_returns_bad_request() {
        let payload = serde_json::from_value(json!({
            "id": 3,
            "method": "render_bibliography",
            "params": {}
        }))
        .expect("payload should deserialize");

        let response = rpc_handler(test_dispatcher(), Json(payload))
            .await
            .into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

        let body = response_body_json(response).await;
        assert_eq!(body["id"], 3);
        assert!(
            body["error"]
                .as_str()
                .expect("error should be a string")
                .contains("style_path")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn app_rejects_oversized_http_request_body() {
        let oversized = "x".repeat(DEFAULT_HTTP_BODY_LIMIT_BYTES + 1);
        let request = Request::builder()
            .method("POST")
            .uri("/rpc")
            .header("content-type", "application/json")
            .body(Body::from(oversized))
            .expect("request should build");

        let response = app().oneshot(request).await.expect("request should run");

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_rpc_returns_405_with_hint_and_allow_header() {
        let request = Request::builder()
            .method("GET")
            .uri("/rpc")
            .body(Body::empty())
            .expect("request should build");

        let response = app().oneshot(request).await.expect("request should run");
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(
            response
                .headers()
                .get("allow")
                .and_then(|v| v.to_str().ok()),
            Some("POST"),
        );

        let body = response_body_json(response).await;
        assert!(body["hint"].as_str().unwrap_or("").contains("POST"));
    }

    #[cfg(feature = "schema")]
    #[tokio::test(flavor = "current_thread")]
    async fn get_rpc_schema_returns_method_schemas() {
        let request = Request::builder()
            .method("GET")
            .uri("/rpc/schema")
            .body(Body::empty())
            .expect("request should build");

        let response = app().oneshot(request).await.expect("request should run");
        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_json(response).await;
        assert!(
            body["render_citation"].is_object(),
            "render_citation schema missing"
        );
        assert!(
            body["render_bibliography"].is_object(),
            "render_bibliography schema missing"
        );
        assert!(
            body["validate_style"].is_object(),
            "validate_style schema missing"
        );
        assert!(
            body["format_document"].is_object(),
            "format_document schema missing"
        );
        #[cfg(feature = "session")]
        {
            assert!(
                body["open_session"]["properties"]["style"].is_object(),
                "open_session schema should describe style params"
            );
            assert!(
                body["get_citations"]["properties"]["session_id"].is_object(),
                "get_citations schema should describe session_id params"
            );
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_rpc_methods_returns_all_four_methods() {
        let request = Request::builder()
            .method("GET")
            .uri("/rpc/methods")
            .body(Body::empty())
            .expect("request should build");

        let response = app().oneshot(request).await.expect("request should run");
        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_json(response).await;
        let methods: Vec<&str> = body
            .as_array()
            .expect("should be array")
            .iter()
            .filter_map(|m| m["method"].as_str())
            .collect();
        assert!(methods.contains(&"render_citation"));
        assert!(methods.contains(&"render_bibliography"));
        assert!(methods.contains(&"validate_style"));
        assert!(methods.contains(&"format_document"));

        let format_document = body
            .as_array()
            .expect("should be array")
            .iter()
            .find(|method| method["method"] == "format_document")
            .expect("format_document descriptor should exist");
        assert_eq!(
            format_document["optional"],
            serde_json::json!(["output_format", "locale", "document_options"])
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rpc_handler_format_document_returns_citations_and_bibliography() {
        let payload = serde_json::from_value(json!({
            "id": 70,
            "method": "format_document",
            "params": {
                "style": { "kind": "path", "value": apa_style_path() },
                "output_format": "plain",
                "refs": hawking_refs(),
                "citations": [{ "id": "cite-1", "items": [{ "id": "ITEM-2" }] }]
            }
        }))
        .expect("payload should deserialize");

        let response = rpc_handler(test_dispatcher(), Json(payload))
            .await
            .into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = response_body_json(response).await;
        let formatted = body["result"]["formatted_citations"]
            .as_array()
            .expect("formatted_citations should be an array");
        assert_eq!(
            formatted.len(),
            1,
            "one citation submitted, one should be returned"
        );
        assert_eq!(
            formatted[0]["id"], "cite-1",
            "citation id should be preserved in output"
        );
        assert!(
            body["result"]["bibliography"].is_object(),
            "result should include a bibliography object: {body:?}"
        );
    }
}
